use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::artifact::NotionPageArtifact;

/// Portable Orbexa sync lock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    pub schema: String,
    #[serde(default)]
    pub pages: Vec<LockedPage>,
}

/// One synced page entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPage {
    pub codexa_id: String,
    pub notion_page_id: String,
    pub notion_page_url: Option<String>,
    pub workspace: String,
    pub data_source: String,
    pub source_repository: String,
    pub source_path: String,
    pub source_commit: String,
    pub content_hash: String,
}

/// Resolves the portable lock path for a workspace.
pub fn resolve_lock_path(workspace: &str) -> Result<PathBuf, LockError> {
    let config_home = if let Ok(config_home) = env::var("XDG_CONFIG_HOME") {
        if !config_home.trim().is_empty() {
            PathBuf::from(config_home)
        } else {
            fallback_config_home()?
        }
    } else {
        fallback_config_home()?
    };

    Ok(config_home
        .join("orbexa")
        .join("locks")
        .join(format!("{}.toml", slug(workspace))))
}

/// Loads a lock file if it exists.
pub fn load_lock(path: impl AsRef<Path>) -> Result<LockFile, LockError> {
    let path = path.as_ref();

    if !path.exists() {
        return Ok(LockFile {
            schema: "orbexa/lock@1".into(),
            pages: Vec::new(),
        });
    }

    let source = fs::read_to_string(path)?;
    let lock: LockFile = toml::from_str(&source)?;

    if lock.schema != "orbexa/lock@1" {
        return Err(LockError::InvalidSchema(lock.schema));
    }

    Ok(lock)
}

/// Writes the lock file.
pub fn write_lock(path: impl AsRef<Path>, lock: &LockFile) -> Result<PathBuf, LockError> {
    let path = path.as_ref().to_path_buf();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut bytes = toml::to_string_pretty(lock)?.into_bytes();
    bytes.push(b'\n');
    fs::write(&path, bytes)?;

    Ok(path)
}

/// Returns an existing locked page by Codexa ID.
#[must_use]
pub fn locked_page<'a>(lock: &'a LockFile, codexa_id: &str) -> Option<&'a LockedPage> {
    lock.pages.iter().find(|page| page.codexa_id == codexa_id)
}

/// Adds or replaces one locked page entry.
pub fn upsert_locked_page(
    lock: &mut LockFile,
    artifact: &NotionPageArtifact,
    notion_page_id: impl Into<String>,
    notion_page_url: Option<String>,
) {
    let entry = LockedPage {
        codexa_id: artifact.document.id.clone(),
        notion_page_id: notion_page_id.into(),
        notion_page_url,
        workspace: artifact.target.workspace.clone(),
        data_source: artifact.target.data_source.clone(),
        source_repository: artifact.source.repository.clone(),
        source_path: artifact.source.path.clone(),
        source_commit: artifact.source.commit.clone(),
        content_hash: artifact.source.content_hash.clone(),
    };

    if let Some(existing) = lock
        .pages
        .iter_mut()
        .find(|page| page.codexa_id == entry.codexa_id)
    {
        *existing = entry;
    } else {
        lock.pages.push(entry);
    }
}

fn fallback_config_home() -> Result<PathBuf, LockError> {
    let home = env::var("HOME").map_err(|_| LockError::MissingHome)?;
    Ok(PathBuf::from(home).join(".config"))
}

fn slug(value: &str) -> String {
    value
        .trim()
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

/// Lock errors.
#[derive(Debug)]
pub enum LockError {
    Io(io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    InvalidSchema(String),
    MissingHome,
}

impl std::fmt::Display for LockError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "lock I/O error: {error}"),
            Self::TomlDeserialize(error) => write!(formatter, "lock TOML error: {error}"),
            Self::TomlSerialize(error) => {
                write!(formatter, "lock TOML serialization error: {error}")
            }
            Self::InvalidSchema(schema) => write!(formatter, "unsupported lock schema `{schema}`"),
            Self::MissingHome => write!(
                formatter,
                "HOME is not set and no XDG config path is available"
            ),
        }
    }
}

impl std::error::Error for LockError {}

impl From<io::Error> for LockError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::de::Error> for LockError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDeserialize(error)
    }
}

impl From<toml::ser::Error> for LockError {
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSerialize(error)
    }
}

#[cfg(test)]
mod tests {
    use super::LockFile;

    #[test]
    fn parses_empty_lock() {
        let source = r#"
schema = "orbexa/lock@1"
"#;

        let lock: LockFile = toml::from_str(source).expect("lock should parse");

        assert_eq!(lock.schema, "orbexa/lock@1");
        assert!(lock.pages.is_empty());
    }
}
