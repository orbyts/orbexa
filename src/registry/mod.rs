use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::config::Config;

/// Portable Git-managed workspace registry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceRegistry {
    pub schema: String,
    pub name: String,
    pub notion: RegistryNotion,
}

/// Notion IDs recorded for a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryNotion {
    pub parent_page_id: String,
    pub workspace_page_id: String,
    pub workspace_page_name: String,
    pub workspace_page_url: Option<String>,
    pub database: RegistryDatabase,
    pub data_sources: RegistryDataSources,
}

/// Recorded database ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryDatabase {
    pub name: String,
    pub id: String,
}

/// Recorded data source IDs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryDataSources {
    pub documents: RegistryDataSource,
}

/// Recorded data source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryDataSource {
    pub name: String,
    pub kind: String,
    pub id: String,
}

/// Loaded registry plus its path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedRegistry {
    pub path: PathBuf,
    pub registry: WorkspaceRegistry,
}

/// Resolves the registry path for the configured workspace.
///
/// For now, Orbexa uses the lowercased workspace page name as the registry
/// file stem. `Codexa` becomes `codexa.toml`.
pub fn resolve_registry_path(config: &Config) -> Result<PathBuf, RegistryError> {
    let config_home = if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        if !config_home.trim().is_empty() {
            PathBuf::from(config_home)
        } else {
            fallback_config_home()?
        }
    } else {
        fallback_config_home()?
    };

    Ok(config_home.join("orbexa").join("workspaces").join(format!(
        "{}.toml",
        registry_slug(&config.workspace.page_name)
    )))
}

/// Loads the workspace registry if it exists.
pub fn load_registry(path: impl AsRef<Path>) -> Result<Option<LoadedRegistry>, RegistryError> {
    let path = path.as_ref().to_path_buf();

    if !path.exists() {
        return Ok(None);
    }

    let source = fs::read_to_string(&path)?;
    let registry: WorkspaceRegistry = toml::from_str(&source)?;

    if registry.schema != "orbexa/workspace@1" {
        return Err(RegistryError::InvalidSchema(registry.schema));
    }

    Ok(Some(LoadedRegistry { path, registry }))
}

/// Writes a workspace registry.
pub fn write_registry(
    path: impl AsRef<Path>,
    registry: &WorkspaceRegistry,
) -> Result<PathBuf, RegistryError> {
    let path = path.as_ref().to_path_buf();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut bytes = toml::to_string_pretty(registry)?.into_bytes();
    bytes.push(b'\n');
    fs::write(&path, bytes)?;

    Ok(path)
}

/// Creates a registry after workspace page creation.
#[must_use]
pub fn registry_from_workspace_page(
    config: &Config,
    workspace_page_id: impl Into<String>,
    workspace_page_url: Option<String>,
) -> WorkspaceRegistry {
    WorkspaceRegistry {
        schema: "orbexa/workspace@1".into(),
        name: config.workspace.page_name.clone(),
        notion: RegistryNotion {
            parent_page_id: config.notion.parent_page_id.clone(),
            workspace_page_id: workspace_page_id.into(),
            workspace_page_name: config.workspace.page_name.clone(),
            workspace_page_url,
            database: RegistryDatabase {
                name: config.workspace.database_name.clone(),
                id: String::new(),
            },
            data_sources: RegistryDataSources {
                documents: RegistryDataSource {
                    name: config.workspace.data_sources.documents.name.clone(),
                    kind: config.workspace.data_sources.documents.kind.clone(),
                    id: String::new(),
                },
            },
        },
    }
}

fn fallback_config_home() -> Result<PathBuf, RegistryError> {
    let home = env::var("HOME").map_err(|_| RegistryError::MissingHome)?;
    Ok(PathBuf::from(home).join(".config"))
}

fn registry_slug(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Registry errors.
#[derive(Debug)]
pub enum RegistryError {
    Io(io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    InvalidSchema(String),
    MissingHome,
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "registry I/O error: {error}"),
            Self::TomlDeserialize(error) => write!(formatter, "registry TOML error: {error}"),
            Self::TomlSerialize(error) => {
                write!(formatter, "registry TOML serialization error: {error}")
            }
            Self::InvalidSchema(schema) => {
                write!(
                    formatter,
                    "unsupported workspace registry schema `{schema}`"
                )
            }
            Self::MissingHome => {
                write!(
                    formatter,
                    "HOME is not set and no XDG config path is available"
                )
            }
        }
    }
}

impl std::error::Error for RegistryError {}

impl From<io::Error> for RegistryError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for RegistryError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDeserialize(error)
    }
}

impl From<toml::ser::Error> for RegistryError {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSerialize(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{WorkspaceRegistry, registry_slug};

    #[test]
    fn slugifies_workspace_name() {
        assert_eq!(registry_slug("Codexa"), "codexa");
        assert_eq!(registry_slug("Codexa Test"), "codexa-test");
        assert_eq!(registry_slug("  My Workspace!  "), "my-workspace");
    }

    #[test]
    fn parses_registry() {
        let source = r#"
schema = "orbexa/workspace@1"

name = "Codexa"

[notion]
parent_page_id = "parent"
workspace_page_id = "workspace"
workspace_page_name = "Codexa"
workspace_page_url = "https://app.notion.com/p/Codexa"

[notion.database]
name = "Knowledge"
id = ""

[notion.data_sources.documents]
name = "Documents"
kind = "documents"
id = ""
"#;

        let registry: WorkspaceRegistry = toml::from_str(source).expect("registry should parse");

        assert_eq!(registry.schema, "orbexa/workspace@1");
        assert_eq!(registry.name, "Codexa");
        assert_eq!(registry.notion.workspace_page_id, "workspace");
        assert_eq!(registry.notion.database.name, "Knowledge");
        assert_eq!(registry.notion.data_sources.documents.name, "Documents");
    }
}
