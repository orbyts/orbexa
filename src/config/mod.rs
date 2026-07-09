use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

/// Top-level Orbexa config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub schema: String,
    pub notion: NotionConfig,
    pub workspace: WorkspaceConfig,
    pub artifacts: ArtifactConfig,
    pub sync: SyncConfig,
}

/// Notion API and bootstrap target config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NotionConfig {
    pub api_version: String,
    pub parent_page_id: String,
    pub bootstrap: BootstrapConfig,
}

/// Controls whether Orbexa verifies or creates Notion workspace objects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BootstrapConfig {
    pub mode: BootstrapMode,
    pub root: BootstrapRoot,
}

/// Bootstrap mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapMode {
    Verify,
    Create,
}

/// Bootstrap root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapRoot {
    ParentPage,
}

/// Workspace object names.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceConfig {
    pub page_name: String,
    pub database_name: String,
    pub data_sources: DataSourcesConfig,
    #[serde(default)]
    pub appearance: WorkspaceAppearance,
}

/// Optional Notion appearance defaults for created workspace objects.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceAppearance {
    #[serde(default = "default_icon")]
    pub icon: WorkspaceIcon,
    #[serde(default = "default_cover")]
    pub cover: WorkspaceCover,
}

impl Default for WorkspaceAppearance {
    fn default() -> Self {
        Self {
            icon: default_icon(),
            cover: default_cover(),
        }
    }
}

/// Notion page icon config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceIcon {
    Emoji { emoji: String },
    Icon { name: String, color: String },
    External { url: String },
}

/// Notion page cover config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceCover {
    External { url: String },
}

fn default_icon() -> WorkspaceIcon {
    WorkspaceIcon::Icon {
        name: "book".into(),
        color: "lightgray".into(),
    }
}

fn default_cover() -> WorkspaceCover {
    WorkspaceCover::External {
        url: "https://res.cloudinary.com/dicttuyma/image/upload/w_1500,h_600,c_fill,g_auto/v1742094786/banner/notion_22.jpg".into(),
    }
}

/// Configured data sources.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DataSourcesConfig {
    pub documents: DataSourceConfig,
}

/// One data source config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DataSourceConfig {
    pub name: String,
    pub kind: String,
}

/// Artifact input config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ArtifactConfig {
    pub input: String,
}

/// Sync policy config.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SyncConfig {
    pub mode: String,
    pub managed_by: String,
    pub on_missing: String,
    pub on_drift: String,
}

/// Config file plus its resolved path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config: Config,
}

/// Loads config from the given path.
pub fn load_config(path: impl AsRef<Path>) -> Result<LoadedConfig, ConfigError> {
    let path = path.as_ref().to_path_buf();
    let source = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&source)?;

    if config.schema != "orbexa/config@1" {
        return Err(ConfigError::InvalidSchema(config.schema));
    }

    Ok(LoadedConfig { path, config })
}

/// Resolves the default Orbexa config path.
///
/// Precedence:
/// 1. Explicit `--config` path
/// 2. `ORBEXA_CONFIG`
/// 3. `$XDG_CONFIG_HOME/orbexa/config.toml`
/// 4. `$HOME/.config/orbexa/config.toml`
pub fn resolve_config_path(explicit: Option<PathBuf>) -> Result<PathBuf, ConfigError> {
    if let Some(path) = explicit {
        return Ok(path);
    }

    if let Ok(path) = env::var("ORBEXA_CONFIG") {
        if !path.trim().is_empty() {
            return Ok(PathBuf::from(path));
        }
    }

    if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        if !config_home.trim().is_empty() {
            return Ok(PathBuf::from(config_home).join("orbexa/config.toml"));
        }
    }

    let home = env::var("HOME").map_err(|_| ConfigError::MissingHome)?;
    Ok(PathBuf::from(home).join(".config/orbexa/config.toml"))
}

/// Resolves the Orbexa state directory.
///
/// Precedence:
/// 1. `$XDG_STATE_HOME/orbexa`
/// 2. `$HOME/.local/state/orbexa`
pub fn resolve_state_dir() -> Result<PathBuf, ConfigError> {
    if let Ok(state_home) = env::var("XDG_STATE_HOME") {
        if !state_home.trim().is_empty() {
            return Ok(PathBuf::from(state_home).join("orbexa"));
        }
    }

    let home = env::var("HOME").map_err(|_| ConfigError::MissingHome)?;
    Ok(PathBuf::from(home).join(".local/state/orbexa"))
}

/// Config loading errors.
#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
    InvalidSchema(String),
    MissingHome,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "config I/O error: {error}"),
            Self::Toml(error) => write!(formatter, "config TOML error: {error}"),
            Self::InvalidSchema(schema) => {
                write!(formatter, "unsupported config schema `{schema}`")
            }
            Self::MissingHome => {
                write!(formatter, "HOME is not set and no XDG path is available")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        Self::Toml(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_config_with_default_appearance() {
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

        assert_eq!(config.schema, "orbexa/config@1");
        assert_eq!(config.notion.bootstrap.mode, BootstrapMode::Create);
        assert_eq!(config.notion.bootstrap.root, BootstrapRoot::ParentPage);
        assert_eq!(config.workspace.page_name, "Codexa");
        assert_eq!(config.workspace.database_name, "Knowledge");
        assert_eq!(config.workspace.data_sources.documents.name, "Documents");

        assert_eq!(
            config.workspace.appearance.icon,
            WorkspaceIcon::Icon {
                name: "book".into(),
                color: "lightgray".into(),
            }
        );
    }

    #[test]
    fn parses_explicit_appearance() {
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

[workspace.appearance.icon]
type = "emoji"
emoji = "📚"

[workspace.appearance.cover]
type = "external"
url = "https://example.com/cover.jpg"

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

        assert_eq!(
            config.workspace.appearance.icon,
            WorkspaceIcon::Emoji {
                emoji: "📚".into()
            }
        );
        assert_eq!(
            config.workspace.appearance.cover,
            WorkspaceCover::External {
                url: "https://example.com/cover.jpg".into(),
            }
        );
    }
}
