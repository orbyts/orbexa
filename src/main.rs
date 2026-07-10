use std::{env, path::PathBuf, process::ExitCode};

use orbexa::{
    artifact::{load_manifest, load_page_artifact},
    config::{LoadedConfig, load_config, resolve_config_path, resolve_state_dir},
    lock::{load_lock, locked_page, resolve_lock_path, upsert_locked_page, write_lock},
    notion::{Block, NotionClient, Page},
    plan::{BootstrapDiscovery, DiscoveredObject, render_init_plan_with_discovery},
    registry::{
        load_registry, registry_from_workspace_page, registry_with_database, resolve_registry_path,
        write_registry,
    },
    state::{State, write_state},
};

fn main() -> ExitCode {
    match run(env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

fn run(args: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    match args.as_slice() {
        [] => {
            print_help();
            Ok(())
        }
        [flag] if flag == "--version" || flag == "-V" => {
            println!("orbexa {}", orbexa::VERSION);
            Ok(())
        }
        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }
        [command] if command == "check" => check(None),
        [command, config_flag, config_path] if command == "check" && config_flag == "--config" => {
            check(Some(PathBuf::from(config_path)))
        }
        [command, flag] if command == "init" && flag == "--dry-run" => init(None, true),
        [command, flag] if command == "init" && flag == "--recreate-database" => {
            recreate_database(false)
        }
        [command, flag, dry_run_flag]
            if command == "init"
                && flag == "--recreate-database"
                && dry_run_flag == "--dry-run" =>
        {
            recreate_database(true)
        }
        [command, dry_run_flag, flag]
            if command == "init"
                && dry_run_flag == "--dry-run"
                && flag == "--recreate-database" =>
        {
            recreate_database(true)
        }
        [command, config_flag, config_path, dry_run_flag]
            if command == "init" && config_flag == "--config" && dry_run_flag == "--dry-run" =>
        {
            init(Some(PathBuf::from(config_path)), true)
        }
        [command, dry_run_flag, config_flag, config_path]
            if command == "init" && dry_run_flag == "--dry-run" && config_flag == "--config" =>
        {
            init(Some(PathBuf::from(config_path)), true)
        }
        [command, config_flag, config_path] if command == "init" && config_flag == "--config" => {
            init(Some(PathBuf::from(config_path)), false)
        }
        [command] if command == "init" => init(None, false),
        [command, input] if command == "apply" => apply(PathBuf::from(input), false),
        [command, input, flag] if command == "apply" && flag == "--dry-run" => {
            apply(PathBuf::from(input), true)
        }
        _ => {
            print_help();
            Err("invalid arguments".into())
        }
    }
}

fn check(explicit_config_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(explicit_config_path)?;
    let token = notion_token()?;

    let client = NotionClient::new(token, loaded.config.notion.api_version.clone());
    let parent = client.retrieve_page(&loaded.config.notion.parent_page_id)?;

    println!("Orbexa check");
    println!();
    println!("Config:");
    println!("  {}", loaded.path.display());
    println!();
    println!("Notion:");
    println!("  parent page id: {}", parent.id);
    println!(
        "  parent title:   {}",
        parent.title().unwrap_or_else(|| "<untitled>".into())
    );
    println!("  api version:    {}", loaded.config.notion.api_version);
    println!();
    println!("Workspace target:");
    println!("  page:        {}", loaded.config.workspace.page_name);
    println!("  database:    {}", loaded.config.workspace.database_name);
    println!(
        "  data source: {}",
        loaded.config.workspace.data_sources.documents.name
    );

    Ok(())
}

fn recreate_database(dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(None)?;
    let registry_path = resolve_registry_path(&loaded.config)?;
    let loaded_registry = load_registry(&registry_path)?
        .ok_or("workspace registry is missing; run `orbexa init` first")?;

    let token = notion_token()?;
    let client = NotionClient::new(token, loaded.config.notion.api_version.clone());

    let workspace_page =
        client.retrieve_page(&loaded_registry.registry.notion.workspace_page_id)?;

    if workspace_page.in_trash {
        return Err(format!(
            "workspace page `{}` is in trash and cannot be used. Restore it in Notion or create a new workspace explicitly.",
            loaded_registry.registry.notion.workspace_page_id
        )
        .into());
    }

    let lock_path = resolve_lock_path(&loaded_registry.registry.name)?;
    let mut lock = load_lock(&lock_path)?;

    println!("Orbexa recreate database");
    println!();
    println!("Registry:");
    println!("  {}", loaded_registry.path.display());
    println!("Workspace page:");
    println!(
        "  {} {}",
        loaded_registry.registry.notion.workspace_page_name,
        loaded_registry.registry.notion.workspace_page_id
    );
    println!();
    println!("Current registered database:");
    println!(
        "  {} {}",
        loaded_registry.registry.notion.database.name, loaded_registry.registry.notion.database.id
    );
    println!();

    if dry_run {
        println!("Would create:");
        println!("  Database     {}", loaded.config.workspace.database_name);
        println!(
            "  Data source  {}",
            loaded.config.workspace.data_sources.documents.name
        );
        println!();
        println!("Would update registry:");
        println!("  {}", loaded_registry.path.display());
        println!();
        println!("Would clear stale page locks:");
        for page in &lock.pages {
            if page.workspace == loaded_registry.registry.name.to_lowercase()
                || page.workspace == loaded_registry.registry.name
            {
                println!("  {}", page.codexa_id);
            }
        }
        return Ok(());
    }

    let database = client.create_database(
        &loaded_registry.registry.notion.workspace_page_id,
        &loaded.config.workspace.database_name,
        &loaded.config.workspace.data_sources.documents.name,
    )?;

    let data_source = database
        .data_source_named(&loaded.config.workspace.data_sources.documents.name)
        .or_else(|| database.data_sources.first())
        .ok_or("created database did not return a data source")?;

    let registry = registry_with_database(
        loaded_registry.registry.clone(),
        database.id.clone(),
        data_source.id.clone(),
    );
    let written_registry_path = write_registry(&registry_path, &registry)?;

    let before = lock.pages.len();
    lock.pages.retain(|page| {
        page.workspace != loaded_registry.registry.name.to_lowercase()
            && page.workspace != loaded_registry.registry.name
    });
    let cleared = before - lock.pages.len();
    let written_lock_path = write_lock(&lock_path, &lock)?;

    println!("Created:");
    println!(
        "  Database {} {}",
        loaded.config.workspace.database_name, database.id
    );
    println!("  Data source {} {}", data_source.name, data_source.id);
    if let Some(url) = &database.url {
        println!("  URL {url}");
    }
    println!();
    println!("Updated registry:");
    println!("  {}", written_registry_path.display());
    println!();
    println!("Cleared stale page locks:");
    println!("  {cleared}");
    println!("Wrote lock:");
    println!("  {}", written_lock_path.display());

    Ok(())
}

fn apply(input_dir: PathBuf, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loaded_config = load_orbexa_config(None)?;
    let registry_path = resolve_registry_path(&loaded_config.config)?;
    let loaded_registry = load_registry(&registry_path)?
        .ok_or("workspace registry is missing; run `orbexa init` first")?;

    if loaded_registry.registry.notion.database.id.is_empty()
        || loaded_registry
            .registry
            .notion
            .data_sources
            .documents
            .id
            .is_empty()
    {
        return Err(
            "workspace registry is missing database/data source IDs; run `orbexa init` first"
                .into(),
        );
    }

    let manifest = load_manifest(&input_dir)?;
    let lock_path = resolve_lock_path(&loaded_registry.registry.name)?;
    let mut lock = load_lock(&lock_path)?;

    println!("Orbexa apply");
    println!();
    println!("Input:");
    println!("  {}", input_dir.display());
    println!("Manifest:");
    println!("  {}", manifest.path.display());
    println!("Registry:");
    println!("  {}", loaded_registry.path.display());
    println!("Lock:");
    println!("  {}", lock_path.display());
    println!();

    let token = notion_token()?;
    let client = NotionClient::new(token, loaded_config.config.notion.api_version.clone());

    let registered_database_id = &loaded_registry.registry.notion.database.id;
    let registered_data_source_id = &loaded_registry.registry.notion.data_sources.documents.id;

    let database = client.retrieve_database(registered_database_id).map_err(|error| {
        format!(
            "registered Notion database `{}` could not be read. It may have been deleted manually. \
Registry: {}\n\
Run an explicit repair/recreate command before applying artifacts. Original error: {error}",
            registered_database_id,
            loaded_registry.path.display()
        )
    })?;

    if database.in_trash {
        return Err(format!(
            "registered Notion database `{}` is in trash and cannot be used. \
Registry: {}\n\
Run an explicit repair/recreate command before applying artifacts.",
            registered_database_id,
            loaded_registry.path.display()
        )
        .into());
    }

    let has_registered_data_source = database
        .data_sources
        .iter()
        .any(|data_source| data_source.id == *registered_data_source_id);

    if !has_registered_data_source {
        return Err(format!(
            "registered Notion data source `{}` was not found in database `{}`. It may have been deleted or replaced manually. \
Registry: {}\n\
Run an explicit repair/recreate command before applying artifacts.",
            registered_data_source_id,
            registered_database_id,
            loaded_registry.path.display()
        )
        .into());
    }

    for page_entry in &manifest.manifest.pages {
        let loaded_page = load_page_artifact(&input_dir, &page_entry.path)?;
        let artifact = loaded_page.artifact;

        if artifact.target.workspace != loaded_registry.registry.name.to_lowercase()
            && artifact.target.workspace != loaded_registry.registry.name
        {
            return Err(format!(
                "artifact `{}` targets workspace `{}`, but loaded registry is `{}`",
                artifact.document.id, artifact.target.workspace, loaded_registry.registry.name
            )
            .into());
        }

        if artifact.target.data_source
            != loaded_registry.registry.notion.data_sources.documents.kind
            && artifact.target.data_source != "documents"
        {
            return Err(format!(
                "artifact `{}` targets unsupported data source `{}`",
                artifact.document.id, artifact.target.data_source
            )
            .into());
        }

        if let Some(existing) = locked_page(&lock, &artifact.document.id) {
            let locked_page = client
                .retrieve_page(&existing.notion_page_id)
                .map_err(|error| {
                    format!(
                        "locked Notion page `{}` for Codexa document `{}` could not be read. \
It may have been deleted manually.\n\
Lock: {}\n\
Run an explicit repair/recreate command before applying artifacts. Original error: {error}",
                        existing.notion_page_id,
                        artifact.document.id,
                        lock_path.display()
                    )
                })?;

            if locked_page.in_trash {
                return Err(format!(
                    "locked Notion page `{}` for Codexa document `{}` is in trash and cannot be used.\n\
Lock: {}\n\
Run an explicit repair/recreate command before applying artifacts.",
                    existing.notion_page_id,
                    artifact.document.id,
                    lock_path.display()
                )
                .into());
            }

            println!("Skip:");
            println!(
                "  {} already synced to {}",
                artifact.document.id, existing.notion_page_id
            );
            println!(
                "  Notion title: {}",
                locked_page.title().unwrap_or_else(|| "<untitled>".into())
            );
            continue;
        }

        if dry_run {
            println!("Would create:");
            println!("  {} → Notion page", artifact.document.id);
            println!("  Title: {}", artifact.document.title);
            println!("  Description: {}", artifact.document.description);
            println!("  Root: {}", artifact.navigation.root);
            println!("  Product: {}", artifact.navigation.product);
            println!("  Kind: {}", artifact.document.kind);
            println!("  Tags: {}", artifact.document.tags.join(", "));
            println!("  Status: {}", artifact.document.status);
            println!("  Visibility: {}", artifact.document.visibility);
            println!();
            continue;
        }

        let page_appearance = loaded_config
            .config
            .workspace
            .data_sources
            .documents
            .appearance
            .as_ref()
            .unwrap_or(&loaded_config.config.workspace.appearance);

        let page = client.create_document_page(
            &loaded_registry.registry.notion.data_sources.documents.id,
            &artifact.document.title,
            &artifact.document.description,
            &artifact.navigation.root,
            &artifact.navigation.product,
            &artifact.document.kind,
            &artifact.document.tags,
            &artifact.document.status,
            &artifact.document.visibility,
            &artifact.content.markdown,
            &page_appearance.icon,
            &page_appearance.cover,
        )?;

        upsert_locked_page(&mut lock, &artifact, page.id.clone(), page.url.clone());

        println!("Created:");
        println!("  {} {}", artifact.document.id, page.id);
        if let Some(url) = &page.url {
            println!("  URL {url}");
        }
        println!();
    }

    if !dry_run {
        let written_lock_path = write_lock(&lock_path, &lock)?;
        println!("Wrote:");
        println!("  {}", written_lock_path.display());
    }

    Ok(())
}

fn init(
    explicit_config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(explicit_config_path)?;
    let state_dir = resolve_state_dir()?;
    let registry_path = resolve_registry_path(&loaded.config)?;
    let token = notion_token()?;

    let client = NotionClient::new(token, loaded.config.notion.api_version.clone());

    if let Some(loaded_registry) = load_registry(&registry_path)? {
        let workspace_page =
            client.retrieve_page(&loaded_registry.registry.notion.workspace_page_id)?;

        if !loaded_registry.registry.notion.database.id.is_empty() {
            println!("Orbexa init");
            println!();
            println!("Already initialized:");
            println!("  Registry: {}", loaded_registry.path.display());
            println!(
                "  Workspace page: {}",
                loaded_registry.registry.notion.workspace_page_name
            );
            println!(
                "  Page ID: {}",
                loaded_registry.registry.notion.workspace_page_id
            );
            println!(
                "  Notion title: {}",
                workspace_page
                    .title()
                    .unwrap_or_else(|| "<untitled>".into())
            );
            println!(
                "  Database: {} {}",
                loaded_registry.registry.notion.database.name,
                loaded_registry.registry.notion.database.id
            );
            println!(
                "  Data source: {} {}",
                loaded_registry.registry.notion.data_sources.documents.name,
                loaded_registry.registry.notion.data_sources.documents.id
            );
            if let Some(url) = &loaded_registry.registry.notion.workspace_page_url {
                println!("  URL: {url}");
            }
            println!();
            println!("No changes made.");
            return Ok(());
        }

        if dry_run {
            println!("Orbexa init plan");
            println!();
            println!("Already has workspace page:");
            println!("  Registry: {}", loaded_registry.path.display());
            println!(
                "  Workspace page: {} {}",
                loaded_registry.registry.notion.workspace_page_name,
                loaded_registry.registry.notion.workspace_page_id
            );
            println!();
            println!("Would create:");
            println!("  Database     {}", loaded.config.workspace.database_name);
            println!(
                "  Data source  {}",
                loaded.config.workspace.data_sources.documents.name
            );
            println!();
            println!("Would update registry:");
            println!("  {}", loaded_registry.path.display());
            return Ok(());
        }

        let database = client.create_database(
            &loaded_registry.registry.notion.workspace_page_id,
            &loaded.config.workspace.database_name,
            &loaded.config.workspace.data_sources.documents.name,
        )?;

        let data_source = database
            .data_source_named(&loaded.config.workspace.data_sources.documents.name)
            .or_else(|| database.data_sources.first())
            .ok_or("created database did not return a data source")?;

        let registry = registry_with_database(
            loaded_registry.registry,
            database.id.clone(),
            data_source.id.clone(),
        );
        let written_registry_path = write_registry(&registry_path, &registry)?;

        println!("Orbexa init");
        println!();
        println!("Created:");
        println!(
            "  Database {} {}",
            loaded.config.workspace.database_name, database.id
        );
        println!("  Data source {} {}", data_source.name, data_source.id);
        if let Some(url) = &database.url {
            println!("  URL {url}");
        }
        println!();
        println!("Updated registry:");
        println!("  {}", written_registry_path.display());
        return Ok(());
    }

    let parent = client.retrieve_page(&loaded.config.notion.parent_page_id)?;
    let children = client.retrieve_block_children(&loaded.config.notion.parent_page_id)?;
    let discovery = build_discovery(&loaded, &parent, &children);

    if dry_run {
        print!(
            "{}",
            render_init_plan_with_discovery(
                &loaded.path,
                &state_dir,
                &loaded.config,
                Some(&discovery),
            )
        );
        println!();
        println!("Would write registry:");
        println!("  {}", registry_path.display());
        return Ok(());
    }

    if !discovery.matching_workspace_pages.is_empty() {
        return Err(format!(
            "workspace page `{}` already exists under parent page; explicit adoption is not implemented yet",
            loaded.config.workspace.page_name
        )
        .into());
    }

    let workspace_page = client.create_child_page(
        &loaded.config.notion.parent_page_id,
        &loaded.config.workspace.page_name,
        &loaded.config.workspace.appearance.icon,
        &loaded.config.workspace.appearance.cover,
    )?;

    let state = State::workspace_page(
        loaded.config.notion.parent_page_id.clone(),
        workspace_page.id.clone(),
        loaded.config.workspace.page_name.clone(),
        workspace_page.url.clone(),
    );
    let state_path = write_state(&state_dir, &state)?;

    let registry = registry_from_workspace_page(
        &loaded.config,
        workspace_page.id.clone(),
        workspace_page.url.clone(),
    );
    let written_registry_path = write_registry(&registry_path, &registry)?;

    println!("Orbexa init");
    println!();
    println!("Created:");
    println!(
        "  Page {} {}",
        loaded.config.workspace.page_name, workspace_page.id
    );
    if let Some(url) = &workspace_page.url {
        println!("  URL  {url}");
    }
    println!();
    println!("Wrote:");
    println!("  {}", state_path.display());
    println!("  {}", written_registry_path.display());
    println!();
    println!("Next:");
    println!("  Run `orbexa init` again to create the database.");

    Ok(())
}

fn build_discovery(loaded: &LoadedConfig, parent: &Page, children: &[Block]) -> BootstrapDiscovery {
    let mut child_pages = Vec::new();
    let mut child_databases = Vec::new();

    for child in children {
        match child.block_type.as_str() {
            "child_page" => {
                if let Some(title) = child.title() {
                    child_pages.push(DiscoveredObject {
                        id: child.id.clone(),
                        title: title.into(),
                    });
                }
            }
            "child_database" => {
                if let Some(title) = child.title() {
                    child_databases.push(DiscoveredObject {
                        id: child.id.clone(),
                        title: title.into(),
                    });
                }
            }
            _ => {}
        }
    }

    let matching_workspace_pages = child_pages
        .iter()
        .filter(|page| page.title == loaded.config.workspace.page_name)
        .cloned()
        .collect();

    BootstrapDiscovery {
        parent_title: parent.title().unwrap_or_else(|| "<untitled>".into()),
        matching_workspace_pages,
        child_pages,
        child_databases,
    }
}

fn load_orbexa_config(
    explicit_config_path: Option<PathBuf>,
) -> Result<LoadedConfig, Box<dyn std::error::Error>> {
    let config_path = resolve_config_path(explicit_config_path)?;
    let loaded = load_config(&config_path)?;
    Ok(loaded)
}

fn notion_token() -> Result<String, Box<dyn std::error::Error>> {
    let token = env::var("NOTION_API_KEY")
        .or_else(|_| env::var("NOTION_TOKEN"))
        .map_err(|_| "neither NOTION_API_KEY nor NOTION_TOKEN is set")?;

    if token.trim().is_empty() {
        return Err("Notion API token is empty".into());
    }

    Ok(token)
}

fn print_help() {
    println!(
        "Orbexa {}\n\nApplies Codexa-generated Notion artifacts to managed Notion pages, databases, and data sources.\n\nUSAGE:\n    orbexa check [--config <PATH>]\n    orbexa init [--dry-run] [--config <PATH>]\n    orbexa init --recreate-database [--dry-run]\n    orbexa apply <ARTIFACT_DIR> [--dry-run]\n    orbexa [OPTIONS]\n\nOPTIONS:\n    -h, --help       Print help\n    -V, --version    Print version",
        orbexa::VERSION
    );
}
