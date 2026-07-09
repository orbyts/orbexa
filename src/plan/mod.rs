use std::path::Path;

use crate::config::{BootstrapMode, Config};

/// Renders the initial Notion bootstrap plan.
#[must_use]
pub fn render_init_plan(config_path: &Path, state_dir: &Path, config: &Config) -> String {
    let mode = match config.notion.bootstrap.mode {
        BootstrapMode::Verify => "verify",
        BootstrapMode::Create => "create",
    };

    let mut output = String::new();

    output.push_str("Orbexa init plan\n\n");

    output.push_str("Config:\n");
    output.push_str(&format!("  {}\n\n", config_path.display()));

    output.push_str("Mode:\n");
    output.push_str(&format!("  {mode}\n\n"));

    output.push_str("Verify:\n");
    output.push_str("  NOTION_API_KEY or NOTION_TOKEN present\n");
    output.push_str(&format!(
        "  Parent page reachable: {}\n",
        config.notion.parent_page_id
    ));
    output.push_str(&format!(
        "  Notion API version: {}\n\n",
        config.notion.api_version
    ));

    match config.notion.bootstrap.mode {
        BootstrapMode::Verify => {
            output.push_str("Would verify:\n");
            output.push_str(&format!("  Page         {}\n", config.workspace.page_name));
            output.push_str(&format!(
                "  Database     {}\n",
                config.workspace.database_name
            ));
            output.push_str(&format!(
                "  Data source  {}\n\n",
                config.workspace.data_sources.documents.name
            ));
        }
        BootstrapMode::Create => {
            output.push_str("Would create or verify:\n");
            output.push_str(&format!("  Page         {}\n", config.workspace.page_name));
            output.push_str(&format!(
                "  Database     {}\n",
                config.workspace.database_name
            ));
            output.push_str(&format!(
                "  Data source  {}\n\n",
                config.workspace.data_sources.documents.name
            ));
        }
    }

    output.push_str("Would write:\n");
    output.push_str(&format!("  {}/state.toml\n", state_dir.display()));
    output.push_str(&format!("  {}/notion.lock\n\n", state_dir.display()));

    output.push_str("Collision policy:\n");
    output.push_str("  If a matching Notion object exists but is not recorded in Orbexa state,\n");
    output.push_str(
        "  Orbexa must stop with a clear error unless the object is explicitly adopted.\n",
    );

    output
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::config::Config;

    use super::render_init_plan;

    #[test]
    fn renders_create_plan() {
        let source = r#"
schema = "orbexa/config@1"

[notion]
api_version = "2026-03-11"
parent_page_id = "398a1865b187802aa885d97afc99896f"

[notion.bootstrap]
mode = "create"
root = "parent_page"

[workspace]
page_name = "Codexa"
database_name = "Knowledge"

[workspace.data_sources.documents]
name = "Documents"
kind = "documents"

[artifacts]
input = "../codexa/dist/notion"

[sync]
mode = "export"
managed_by = "orbexa"
on_missing = "mark_stale"
on_drift = "warn_and_skip"
"#;

        let config: Config = toml::from_str(source).expect("config should parse");
        let plan = render_init_plan(
            &PathBuf::from("/tmp/config.toml"),
            &PathBuf::from("/tmp/state/orbexa"),
            &config,
        );

        assert!(plan.contains("Orbexa init plan"));
        assert!(plan.contains("Page         Codexa"));
        assert!(plan.contains("Database     Knowledge"));
        assert!(plan.contains("Data source  Documents"));
    }
}
