use std::{env, path::PathBuf, process::ExitCode};

use orbexa::{
    config::{load_config, resolve_config_path, resolve_state_dir},
    plan::render_init_plan,
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

fn init(
    explicit_config_path: Option<PathBuf>,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let config_path = resolve_config_path(explicit_config_path)?;
    let loaded = load_config(&config_path)?;
    let state_dir = resolve_state_dir()?;

    let token = env::var("NOTION_TOKEN").map_err(|_| "NOTION_TOKEN is not set")?;
    if token.trim().is_empty() {
        return Err("NOTION_TOKEN is empty".into());
    }

    if dry_run {
        print!(
            "{}",
            render_init_plan(&loaded.path, &state_dir, &loaded.config)
        );
        return Ok(());
    }

    Err("orbexa init without --dry-run is not implemented yet".into())
}

fn print_help() {
    println!(
        "Orbexa {}\n\nApplies Codexa-generated Notion artifacts to managed Notion pages, databases, and data sources.\n\nUSAGE:\n    orbexa init [--dry-run] [--config <PATH>]\n    orbexa [OPTIONS]\n\nOPTIONS:\n    -h, --help       Print help\n    -V, --version    Print version",
        orbexa::VERSION
    );
}
