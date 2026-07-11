use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Config {
    pub schema: String,
    pub notion: NotionConfig,
    pub workspace: WorkspaceConfig,
    pub artifacts: ArtifactConfig,
    pub sync: SyncConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct NotionConfig {
    pub api_version: String,
    pub parent_page_id: String,
    pub bootstrap: BootstrapConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct BootstrapConfig {
    pub mode: BootstrapMode,
    pub root: BootstrapRoot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapMode {
    Verify,
    Create,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BootstrapRoot {
    ParentPage,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceConfig {
    pub page_name: String,
    pub appearance: WorkspaceAppearance,
    pub roots: BTreeMap<String, RootConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RootConfig {
    pub database_name: String,
    pub data_source_name: String,
    pub appearance: WorkspaceAppearance,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct WorkspaceAppearance {
    pub icon: WorkspaceIcon,
    pub cover: WorkspaceCover,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceIcon {
    Emoji { emoji: String },
    Icon { name: String, color: String },
    External { url: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WorkspaceCover {
    External { url: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ArtifactConfig {
    pub input: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SyncConfig {
    pub on_missing: String,
    pub on_drift: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedConfig {
    pub path: PathBuf,
    pub config: Config,
}

pub fn load_config(path: impl AsRef<Path>) -> Result<LoadedConfig, ConfigError> {
    let path = path.as_ref().to_path_buf();
    let source = fs::read_to_string(&path)?;
    let config: Config = toml::from_str(&source)?;

    if config.schema != "orbexa/config@2" {
        return Err(ConfigError::InvalidSchema(config.schema));
    }
    if config.workspace.roots.is_empty() {
        return Err(ConfigError::MissingRoots);
    }
    for (key, root) in &config.workspace.roots {
        if key.trim().is_empty()
            || root.database_name.trim().is_empty()
            || root.data_source_name.trim().is_empty()
        {
            return Err(ConfigError::InvalidRoot(key.clone()));
        }
    }

    Ok(LoadedConfig { path, config })
}

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

pub fn resolve_state_dir() -> Result<PathBuf, ConfigError> {
    if let Ok(state_home) = env::var("XDG_STATE_HOME") {
        if !state_home.trim().is_empty() {
            return Ok(PathBuf::from(state_home).join("orbexa"));
        }
    }
    let home = env::var("HOME").map_err(|_| ConfigError::MissingHome)?;
    Ok(PathBuf::from(home).join(".local/state/orbexa"))
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Toml(toml::de::Error),
    InvalidSchema(String),
    MissingRoots,
    InvalidRoot(String),
    MissingHome,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "config I/O error: {e}"),
            Self::Toml(e) => write!(f, "config TOML error: {e}"),
            Self::InvalidSchema(s) => write!(f, "unsupported config schema `{s}`"),
            Self::MissingRoots => write!(f, "workspace.roots must define at least one root"),
            Self::InvalidRoot(root) => write!(f, "invalid workspace root `{root}`"),
            Self::MissingHome => write!(f, "HOME is not set and no XDG path is available"),
        }
    }
}
impl std::error::Error for ConfigError {}
impl From<io::Error> for ConfigError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}
impl From<toml::de::Error> for ConfigError {
    fn from(v: toml::de::Error) -> Self {
        Self::Toml(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG: &str = r#"
schema = "orbexa/config@2"

[notion]
api_version = "2026-03-11"
parent_page_id = "parent"

[notion.bootstrap]
mode = "create"
root = "parent_page"

[workspace]
page_name = "Codexa"

[workspace.appearance.icon]
type = "emoji"
emoji = "🧭"

[workspace.appearance.cover]
type = "external"
url = "https://example.com/workspace.jpg"

[workspace.roots.docs]
database_name = "Docs"
data_source_name = "Documents"

[workspace.roots.docs.appearance.icon]
type = "emoji"
emoji = "📘"

[workspace.roots.docs.appearance.cover]
type = "external"
url = "https://example.com/docs.jpg"

[workspace.roots.knowledge]
database_name = "Knowledge"
data_source_name = "Documents"

[workspace.roots.knowledge.appearance.icon]
type = "emoji"
emoji = "📚"

[workspace.roots.knowledge.appearance.cover]
type = "external"
url = "https://example.com/knowledge.jpg"

[artifacts]
input = "../codexa/dist/notion"

[sync]
on_missing = "recreate"
on_drift = "update"
"#;

    #[test]
    fn parses_root_oriented_config() {
        let config: Config = toml::from_str(CONFIG).unwrap();
        assert_eq!(config.schema, "orbexa/config@2");
        assert_eq!(config.workspace.roots.len(), 2);
        assert_eq!(config.workspace.roots["docs"].database_name, "Docs");
        assert_eq!(
            config.workspace.roots["knowledge"].database_name,
            "Knowledge"
        );
    }
}
