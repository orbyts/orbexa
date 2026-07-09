use std::{env, path::PathBuf, process::ExitCode};

use orbexa::{
    config::{LoadedConfig, load_config, resolve_config_path, resolve_state_dir},
    notion::{Block, NotionClient, Page},
    plan::{BootstrapDiscovery, DiscoveredObject, render_init_plan_with_discovery},
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

fn init(
    explicit_config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let loaded = load_orbexa_config(explicit_config_path)?;
    let state_dir = resolve_state_dir()?;
    let token = notion_token()?;

    let client = NotionClient::new(token, loaded.config.notion.api_version.clone());
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
        return Ok(());
    }

    if !discovery.matching_workspace_pages.is_empty() {
        return Err(format!(
            "workspace page `{}` already exists under parent page; explicit adoption is not implemented yet",
            loaded.config.workspace.page_name
        )
        .into());
    }

    Err("orbexa init without --dry-run is not implemented yet".into())
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
        "Orbexa {}\n\nApplies Codexa-generated Notion artifacts to managed Notion pages, databases, and data sources.\n\nUSAGE:\n    orbexa check [--config <PATH>]\n    orbexa init [--dry-run] [--config <PATH>]\n    orbexa [OPTIONS]\n\nOPTIONS:\n    -h, --help       Print help\n    -V, --version    Print version",
        orbexa::VERSION
    );
}
