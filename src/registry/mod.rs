use crate::config::Config;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRegistry {
    pub schema: String,
    pub name: String,
    pub notion: RegistryNotion,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryNotion {
    pub parent_page_id: String,
    pub workspace_page_id: String,
    pub workspace_page_name: String,
    pub workspace_page_url: Option<String>,
    #[serde(default)]
    pub roots: BTreeMap<String, RegistryRoot>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryRoot {
    pub database_name: String,
    pub database_id: String,
    pub data_source_name: String,
    pub data_source_id: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedRegistry {
    pub path: PathBuf,
    pub registry: WorkspaceRegistry,
}

pub fn resolve_registry_path(config: &Config) -> Result<PathBuf, RegistryError> {
    let home = config_home()?;
    Ok(home
        .join("orbexa/workspaces")
        .join(format!("{}.toml", slug(&config.workspace.page_name))))
}
pub fn load_registry(path: impl AsRef<Path>) -> Result<Option<LoadedRegistry>, RegistryError> {
    let path = path.as_ref().to_path_buf();
    if !path.exists() {
        return Ok(None);
    }
    let registry: WorkspaceRegistry = toml::from_str(&fs::read_to_string(&path)?)?;
    if registry.schema != "orbexa/workspace@2" {
        return Err(RegistryError::InvalidSchema(registry.schema));
    }
    Ok(Some(LoadedRegistry { path, registry }))
}
pub fn write_registry(
    path: impl AsRef<Path>,
    registry: &WorkspaceRegistry,
) -> Result<PathBuf, RegistryError> {
    let path = path.as_ref().to_path_buf();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, format!("{}\n", toml::to_string_pretty(registry)?))?;
    Ok(path)
}
pub fn registry_from_workspace_page(
    config: &Config,
    id: impl Into<String>,
    url: Option<String>,
) -> WorkspaceRegistry {
    WorkspaceRegistry {
        schema: "orbexa/workspace@2".into(),
        name: config.workspace.page_name.clone(),
        notion: RegistryNotion {
            parent_page_id: config.notion.parent_page_id.clone(),
            workspace_page_id: id.into(),
            workspace_page_name: config.workspace.page_name.clone(),
            workspace_page_url: url,
            roots: BTreeMap::new(),
        },
    }
}
pub fn upsert_root(registry: &mut WorkspaceRegistry, key: impl Into<String>, root: RegistryRoot) {
    registry.notion.roots.insert(key.into(), root);
}

fn config_home() -> Result<PathBuf, RegistryError> {
    if let Ok(v) = env::var("XDG_CONFIG_HOME") {
        if !v.trim().is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    let home = env::var("HOME").map_err(|_| RegistryError::MissingHome)?;
    Ok(PathBuf::from(home).join(".config"))
}
fn slug(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[derive(Debug)]
pub enum RegistryError {
    Io(io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    InvalidSchema(String),
    MissingHome,
}
impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "registry I/O error: {e}"),
            Self::TomlDeserialize(e) => write!(f, "registry TOML error: {e}"),
            Self::TomlSerialize(e) => write!(f, "registry TOML serialization error: {e}"),
            Self::InvalidSchema(s) => write!(f, "unsupported registry schema `{s}`"),
            Self::MissingHome => write!(f, "HOME is not set and no XDG config path is available"),
        }
    }
}
impl std::error::Error for RegistryError {}
impl From<io::Error> for RegistryError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}
impl From<toml::de::Error> for RegistryError {
    fn from(v: toml::de::Error) -> Self {
        Self::TomlDeserialize(v)
    }
}
impl From<toml::ser::Error> for RegistryError {
    fn from(v: toml::ser::Error) -> Self {
        Self::TomlSerialize(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn serializes_root_registry() {
        let mut r = WorkspaceRegistry {
            schema: "orbexa/workspace@2".into(),
            name: "Codexa".into(),
            notion: RegistryNotion {
                parent_page_id: "p".into(),
                workspace_page_id: "w".into(),
                workspace_page_name: "Codexa".into(),
                workspace_page_url: None,
                roots: BTreeMap::new(),
            },
        };
        upsert_root(
            &mut r,
            "docs",
            RegistryRoot {
                database_name: "Docs".into(),
                database_id: "d".into(),
                data_source_name: "Documents".into(),
                data_source_id: "s".into(),
            },
        );
        let t = toml::to_string(&r).unwrap();
        assert!(t.contains("[notion.roots.docs]"));
    }
}
