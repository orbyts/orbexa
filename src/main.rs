use std::{collections::BTreeMap, env, path::PathBuf, process::ExitCode};

use orbexa::{
    artifact::{NotionPageArtifact, load_manifest, load_page_artifact},
    config::{LoadedConfig, RootConfig, load_config, resolve_config_path, resolve_state_dir},
    lock::{clear_root, load_lock, locked_page, resolve_lock_path, upsert_locked_page, write_lock},
    notion::{CreateDocumentPage, NotionClient, UpdateDocumentPage},
    registry::{
        RegistryRoot, WorkspaceRegistry, load_registry, registry_from_workspace_page,
        resolve_registry_path, upsert_root, write_registry,
    },
    render::{
        NotionIdentity, render_notion_markdown, rendered_content_hash, validate_link_targets,
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
        [command, flag, path] if command == "check" && flag == "--config" => {
            check(Some(path.into()))
        }
        [command] if command == "init" => init(None, false),
        [command, flag] if command == "init" && flag == "--dry-run" => init(None, true),
        [command, flag, root] if command == "init" && flag == "--recreate-root" => {
            recreate_root(root, false)
        }
        [command, flag, root, dry]
            if command == "init" && flag == "--recreate-root" && dry == "--dry-run" =>
        {
            recreate_root(root, true)
        }
        [command, flag, path] if command == "init" && flag == "--config" => {
            init(Some(path.into()), false)
        }
        [command, flag, path, dry]
            if command == "init" && flag == "--config" && dry == "--dry-run" =>
        {
            init(Some(path.into()), true)
        }
        [command, input] if command == "apply" => apply(input.into(), false),
        [command, input, flag] if command == "apply" && flag == "--dry-run" => {
            apply(input.into(), true)
        }
        _ => {
            print_help();
            Err("invalid arguments".into())
        }
    }
}

fn check(config_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(config_path)?;
    let client = client(&loaded)?;
    let parent = client.retrieve_page(&loaded.config.notion.parent_page_id)?;
    println!("Orbexa check\n\nConfig:\n  {}", loaded.path.display());
    println!("\nNotion:\n  parent page id: {}", parent.id);
    println!(
        "  parent title:   {}",
        parent.title().unwrap_or_else(|| "<untitled>".into())
    );
    println!("  api version:    {}", loaded.config.notion.api_version);
    println!(
        "\nWorkspace target:\n  page: {}",
        loaded.config.workspace.page_name
    );
    println!("  roots:");
    for (key, root) in &loaded.config.workspace.roots {
        println!(
            "    {key}: {} / {}",
            root.database_name, root.data_source_name
        );
    }
    Ok(())
}

fn init(config_path: Option<PathBuf>, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(config_path)?;
    let registry_path = resolve_registry_path(&loaded.config)?;
    let client = client(&loaded)?;

    let Some(mut loaded_registry) = load_registry(&registry_path)? else {
        if dry_run {
            println!(
                "Orbexa init plan\n\nWould create workspace page:\n  {}",
                loaded.config.workspace.page_name
            );
            println!("Would write registry:\n  {}", registry_path.display());
            println!("A following init run would create roots:");
            for (key, root) in &loaded.config.workspace.roots {
                println!("  {key}: {}", root.database_name);
            }
            return Ok(());
        }
        let page = client.create_child_page(
            &loaded.config.notion.parent_page_id,
            &loaded.config.workspace.page_name,
            &loaded.config.workspace.appearance.icon,
            &loaded.config.workspace.appearance.cover,
        )?;
        let registry =
            registry_from_workspace_page(&loaded.config, page.id.clone(), page.url.clone());
        let written = write_registry(&registry_path, &registry)?;
        let state = State::workspace_page(
            loaded.config.notion.parent_page_id.clone(),
            page.id.clone(),
            loaded.config.workspace.page_name.clone(),
            page.url.clone(),
        );
        let state_path = write_state(&resolve_state_dir()?, &state)?;
        println!(
            "Orbexa init\n\nCreated workspace page:\n  {} {}",
            loaded.config.workspace.page_name, page.id
        );
        println!(
            "Wrote:\n  {}\n  {}",
            written.display(),
            state_path.display()
        );
        println!("\nNext: run `orbexa init` again to create configured roots.");
        return Ok(());
    };

    let workspace = client.retrieve_page(&loaded_registry.registry.notion.workspace_page_id)?;
    if workspace.in_trash {
        return Err("registered workspace page is in trash".into());
    }

    let mut changed = false;
    println!(
        "Orbexa init\n\nWorkspace:\n  {} {}",
        loaded_registry.registry.notion.workspace_page_name, workspace.id
    );
    println!("\nRoots:");
    for (key, root_config) in &loaded.config.workspace.roots {
        if let Some(root) = loaded_registry.registry.notion.roots.get(key) {
            validate_registered_root(&client, key, root)?;
            println!(
                "  {key}: verified {} {}",
                root.database_name, root.database_id
            );
            continue;
        }
        if dry_run {
            println!(
                "  {key}: would create {} / {}",
                root_config.database_name, root_config.data_source_name
            );
            continue;
        }
        let root = create_root(&client, &loaded_registry.registry, root_config)?;
        println!(
            "  {key}: created {} {}",
            root.database_name, root.database_id
        );
        upsert_root(&mut loaded_registry.registry, key.clone(), root);
        changed = true;
    }
    if changed {
        let path = write_registry(&registry_path, &loaded_registry.registry)?;
        println!("\nUpdated registry:\n  {}", path.display());
    } else if !dry_run {
        println!("\nNo changes made.");
    }
    Ok(())
}

fn recreate_root(root_key: &str, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(None)?;
    let root_config = loaded
        .config
        .workspace
        .roots
        .get(root_key)
        .ok_or_else(|| format!("unknown configured root `{root_key}`"))?;
    let registry_path = resolve_registry_path(&loaded.config)?;
    let mut loaded_registry = load_registry(&registry_path)?
        .ok_or("workspace registry is missing; run `orbexa init` first")?;
    let client = client(&loaded)?;
    let lock_path = resolve_lock_path(&loaded_registry.registry.name)?;
    let mut lock = load_lock(&lock_path)?;

    println!("Orbexa recreate root\n\nRoot: {root_key}");
    if let Some(current) = loaded_registry.registry.notion.roots.get(root_key) {
        println!(
            "Current database: {} {}",
            current.database_name, current.database_id
        );
    }
    if dry_run {
        println!(
            "Would create: {} / {}",
            root_config.database_name, root_config.data_source_name
        );
        println!(
            "Would clear {} page lock(s).",
            lock.pages.iter().filter(|p| p.root == root_key).count()
        );
        return Ok(());
    }
    let root = create_root(&client, &loaded_registry.registry, root_config)?;
    upsert_root(
        &mut loaded_registry.registry,
        root_key.to_string(),
        root.clone(),
    );
    let cleared = clear_root(&mut lock, root_key);
    write_registry(&registry_path, &loaded_registry.registry)?;
    write_lock(&lock_path, &lock)?;
    println!("Created: {} {}", root.database_name, root.database_id);
    println!("Cleared page locks: {cleared}");
    Ok(())
}

fn apply(input_dir: PathBuf, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(None)?;
    let registry_path = resolve_registry_path(&loaded.config)?;
    let loaded_registry = load_registry(&registry_path)?
        .ok_or("workspace registry is missing; run `orbexa init` first")?;
    let manifest = load_manifest(&input_dir)?;
    let client = client(&loaded)?;
    let lock_path = resolve_lock_path(&loaded_registry.registry.name)?;
    let mut lock = load_lock(&lock_path)?;

    let mut roots = BTreeMap::new();
    for (key, root) in &loaded_registry.registry.notion.roots {
        validate_registered_root(&client, key, root)?;
        roots.insert(key.clone(), root.clone());
    }

    let mut artifacts = Vec::new();
    for entry in &manifest.manifest.pages {
        let artifact = load_page_artifact(&input_dir, &entry.path)?.artifact;
        validate_artifact_target(&artifact, &loaded_registry.registry)?;
        if !roots.contains_key(&artifact.target.root) {
            return Err(format!(
                "artifact `{}` targets uninitialized root `{}`",
                artifact.document.id, artifact.target.root
            )
            .into());
        }
        artifacts.push(artifact);
    }
    validate_link_targets(&artifacts)?;

    println!(
        "Orbexa apply\n\nInput:\n  {}\nManifest:\n  {}",
        input_dir.display(),
        manifest.path.display()
    );

    if dry_run {
        return apply_dry_run(&client, &artifacts, &roots, &lock);
    }

    let mut identities = BTreeMap::new();

    // Pass 1: establish every Notion page identity before rendering links.
    for artifact in &artifacts {
        let root = &roots[&artifact.target.root];
        let appearance = &loaded.config.workspace.roots[&artifact.target.root].appearance;

        let (page, identity_changed) = if let Some(existing) =
            locked_page(&lock, &artifact.document.id).cloned()
        {
            if existing.root != artifact.target.root {
                return Err(format!(
                    "document `{}` moved from root `{}` to `{}`; explicit move support is not implemented yet",
                    artifact.document.id, existing.root, artifact.target.root
                )
                .into());
            }
            let page = client.retrieve_page(&existing.notion_page_id)?;
            if page.in_trash {
                if loaded.config.sync.on_missing != "recreate" {
                    return Err(format!(
                        "locked page `{}` is in trash and sync.on_missing is `{}`",
                        existing.notion_page_id, loaded.config.sync.on_missing
                    )
                    .into());
                }
                (
                    create_placeholder_page(&client, artifact, root, appearance)?,
                    true,
                )
            } else {
                (page, false)
            }
        } else {
            (
                create_placeholder_page(&client, artifact, root, appearance)?,
                true,
            )
        };

        let page_url = page
            .url
            .clone()
            .ok_or_else(|| format!("Notion page `{}` has no URL", page.id))?;
        identities.insert(
            artifact.document.id.clone(),
            NotionIdentity {
                page_id: page.id.clone(),
                page_url: page_url.clone(),
                title: artifact.document.title.clone(),
            },
        );

        if identity_changed {
            upsert_locked_page(&mut lock, artifact, page.id, Some(page_url), "");
        }
    }

    // Pass 2: resolve all logical links and update only changed pages.
    for artifact in &artifacts {
        let identity = &identities[&artifact.document.id];
        let rendered = render_notion_markdown(artifact, &identities)?;
        let rendered_hash = rendered_content_hash(artifact, &rendered);
        let existing = locked_page(&lock, &artifact.document.id)
            .cloned()
            .ok_or("page identity disappeared during apply")?;

        if existing.source_content_hash == artifact.source.content_hash
            && existing.rendered_content_hash == rendered_hash
        {
            println!(
                "Skip:\n  {} already synced to {}",
                artifact.document.id, identity.page_id
            );
            continue;
        }

        let appearance = &loaded.config.workspace.roots[&artifact.target.root].appearance;
        client.update_document_page(
            &identity.page_id,
            &UpdateDocumentPage {
                title: &artifact.document.title,
                description: &artifact.document.description,
                root: &artifact.navigation.root,
                product: &artifact.navigation.product,
                kind: &artifact.document.kind,
                tags: &artifact.document.tags,
                status: &artifact.document.status,
                visibility: &artifact.document.visibility,
                icon: &appearance.icon,
                cover: &appearance.cover,
            },
        )?;
        client.replace_page_markdown(&identity.page_id, &rendered)?;

        upsert_locked_page(
            &mut lock,
            artifact,
            identity.page_id.clone(),
            Some(identity.page_url.clone()),
            rendered_hash,
        );
        println!("Updated:\n  {} {}", artifact.document.id, identity.page_id);
    }

    write_lock(&lock_path, &lock)?;
    Ok(())
}

fn apply_dry_run(
    client: &NotionClient,
    artifacts: &[NotionPageArtifact],
    roots: &BTreeMap<String, RegistryRoot>,
    lock: &orbexa::lock::LockFile,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut identities = BTreeMap::new();
    let mut missing = std::collections::BTreeSet::new();

    for artifact in artifacts {
        if let Some(existing) = locked_page(lock, &artifact.document.id) {
            let page = client.retrieve_page(&existing.notion_page_id)?;
            if !page.in_trash {
                if let Some(url) = page.url {
                    identities.insert(
                        artifact.document.id.clone(),
                        NotionIdentity {
                            page_id: page.id,
                            page_url: url,
                            title: artifact.document.title.clone(),
                        },
                    );
                    continue;
                }
            }
        }
        missing.insert(artifact.document.id.clone());
        let root = &roots[&artifact.target.root];
        println!(
            "Would create:\n  {} → {}\n  Root: {}\n  Product: {}",
            artifact.document.id,
            root.database_name,
            artifact.target.root,
            artifact.navigation.product
        );
    }

    for artifact in artifacts {
        if missing.contains(&artifact.document.id) {
            println!(
                "Would render after identity creation:\n  {}",
                artifact.document.id
            );
            continue;
        }
        if artifact
            .links
            .iter()
            .any(|link| !identities.contains_key(&link.target_id))
        {
            println!(
                "Would refresh links after identity creation:\n  {}",
                artifact.document.id
            );
            continue;
        }

        let rendered = render_notion_markdown(artifact, &identities)?;
        let rendered_hash = rendered_content_hash(artifact, &rendered);
        let existing =
            locked_page(lock, &artifact.document.id).ok_or("dry-run identity has no lock entry")?;
        if existing.source_content_hash == artifact.source.content_hash
            && existing.rendered_content_hash == rendered_hash
        {
            println!(
                "Skip:\n  {} already synced to {}",
                artifact.document.id, existing.notion_page_id
            );
        } else {
            println!(
                "Would update:\n  {} {}",
                artifact.document.id, existing.notion_page_id
            );
        }
    }
    Ok(())
}

fn create_placeholder_page(
    client: &NotionClient,
    artifact: &NotionPageArtifact,
    root: &RegistryRoot,
    appearance: &orbexa::config::WorkspaceAppearance,
) -> Result<orbexa::notion::Page, Box<dyn std::error::Error>> {
    let page = client.create_document_page(&CreateDocumentPage {
        data_source_id: &root.data_source_id,
        title: &artifact.document.title,
        description: &artifact.document.description,
        root: &artifact.navigation.root,
        product: &artifact.navigation.product,
        kind: &artifact.document.kind,
        tags: &artifact.document.tags,
        status: &artifact.document.status,
        visibility: &artifact.document.visibility,
        markdown: "_Orbexa is preparing this page._",
        icon: &appearance.icon,
        cover: &appearance.cover,
    })?;
    println!("Created identity:\n  {} {}", artifact.document.id, page.id);
    Ok(page)
}

fn create_root(
    client: &NotionClient,
    registry: &WorkspaceRegistry,
    config: &RootConfig,
) -> Result<RegistryRoot, Box<dyn std::error::Error>> {
    let database = client.create_database(
        &registry.notion.workspace_page_id,
        &config.database_name,
        &config.data_source_name,
    )?;
    let data_source_id = database
        .data_source_named(&config.data_source_name)
        .or_else(|| database.data_sources.first())
        .ok_or("created database did not return a data source")?
        .id
        .clone();
    Ok(RegistryRoot {
        database_name: config.database_name.clone(),
        database_id: database.id,
        data_source_name: config.data_source_name.clone(),
        data_source_id,
    })
}

fn validate_registered_root(
    client: &NotionClient,
    key: &str,
    root: &RegistryRoot,
) -> Result<(), Box<dyn std::error::Error>> {
    let database = client.retrieve_database(&root.database_id)?;
    if database.in_trash {
        return Err(format!("registered root `{key}` database `{}` is in trash; run `orbexa init --recreate-root {key}`", root.database_id).into());
    }
    if !database
        .data_sources
        .iter()
        .any(|source| source.id == root.data_source_id)
    {
        return Err(format!(
            "registered root `{key}` data source `{}` is missing",
            root.data_source_id
        )
        .into());
    }
    Ok(())
}

fn validate_artifact_target(
    artifact: &NotionPageArtifact,
    registry: &WorkspaceRegistry,
) -> Result<(), Box<dyn std::error::Error>> {
    if !artifact
        .target
        .workspace
        .eq_ignore_ascii_case(&registry.name)
    {
        return Err(format!(
            "artifact `{}` targets workspace `{}` but registry is `{}`",
            artifact.document.id, artifact.target.workspace, registry.name
        )
        .into());
    }
    if artifact.target.root != artifact.navigation.root {
        return Err(format!(
            "artifact `{}` target root and navigation root disagree",
            artifact.document.id
        )
        .into());
    }
    Ok(())
}

fn client(loaded: &LoadedConfig) -> Result<NotionClient, Box<dyn std::error::Error>> {
    Ok(NotionClient::new(
        notion_token()?,
        loaded.config.notion.api_version.clone(),
    ))
}
fn load_orbexa_config(path: Option<PathBuf>) -> Result<LoadedConfig, Box<dyn std::error::Error>> {
    Ok(load_config(resolve_config_path(path)?)?)
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
        "Orbexa {}\n\nUSAGE:\n    orbexa check [--config <PATH>]\n    orbexa init [--dry-run] [--config <PATH>]\n    orbexa init --recreate-root <ROOT> [--dry-run]\n    orbexa apply <ARTIFACT_DIR> [--dry-run]\n",
        orbexa::VERSION
    );
}
