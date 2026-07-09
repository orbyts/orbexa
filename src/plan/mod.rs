use std::path::Path;

use crate::config::{BootstrapMode, Config};

/// Read-only discovery result for the configured Notion parent page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapDiscovery {
    pub parent_title: String,
    pub matching_workspace_pages: Vec<DiscoveredObject>,
    pub child_pages: Vec<DiscoveredObject>,
    pub child_databases: Vec<DiscoveredObject>,
}

/// A Notion object discovered during bootstrap planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredObject {
    pub id: String,
    pub title: String,
}

/// Renders the initial Notion bootstrap plan without live discovery.
#[must_use]
pub fn render_init_plan(config_path: &Path, state_dir: &Path, config: &Config) -> String {
    render_init_plan_with_discovery(config_path, state_dir, config, None)
}

/// Renders the initial Notion bootstrap plan.
#[must_use]
pub fn render_init_plan_with_discovery(
    config_path: &Path,
    state_dir: &Path,
    config: &Config,
    discovery: Option<&BootstrapDiscovery>,
) -> String {
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

    if let Some(discovery) = discovery {
        output.push_str("Discovered parent:\n");
        output.push_str(&format!("  Title: {}\n", discovery.parent_title));
        output.push_str(&format!("  Child pages: {}\n", discovery.child_pages.len()));
        output.push_str(&format!(
            "  Child databases: {}\n\n",
            discovery.child_databases.len()
        ));

        if !discovery.matching_workspace_pages.is_empty() {
            output.push_str("Collision:\n");
            for page in &discovery.matching_workspace_pages {
                output.push_str(&format!(
                    "  Page `{}` already exists under the parent: {}\n",
                    page.title, page.id
                ));
            }
            output.push_str("  Orbexa will not adopt or overwrite this page unless it is explicitly recorded in state.\n\n");
        }
    }

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

    use super::{
        BootstrapDiscovery, DiscoveredObject, render_init_plan, render_init_plan_with_discovery,
    };

    fn sample_config() -> Config {
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

        toml::from_str(source).expect("config should parse")
    }

    #[test]
    fn renders_create_plan() {
        let config = sample_config();
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

    #[test]
    fn renders_collision_when_workspace_page_exists() {
        let config = sample_config();
        let discovery = BootstrapDiscovery {
            parent_title: "Codexa Test".into(),
            matching_workspace_pages: vec![DiscoveredObject {
                id: "abc".into(),
                title: "Codexa".into(),
            }],
            child_pages: vec![DiscoveredObject {
                id: "abc".into(),
                title: "Codexa".into(),
            }],
            child_databases: Vec::new(),
        };

        let plan = render_init_plan_with_discovery(
            &PathBuf::from("/tmp/config.toml"),
            &PathBuf::from("/tmp/state/orbexa"),
            &config,
            Some(&discovery),
        );

        assert!(plan.contains("Collision:"));
        assert!(plan.contains("Page `Codexa` already exists"));
    }
}
