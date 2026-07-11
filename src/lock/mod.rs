use crate::artifact::NotionPageArtifact;
use serde::{Deserialize, Serialize};
use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    pub schema: String,
    #[serde(default)]
    pub pages: Vec<LockedPage>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockedPage {
    pub codexa_id: String,
    pub root: String,
    pub notion_page_id: String,
    pub notion_page_url: Option<String>,
    pub source_repository: String,
    pub source_path: String,
    pub source_commit: String,
    pub source_content_hash: String,
    pub rendered_content_hash: String,
}
pub fn resolve_lock_path(workspace: &str) -> Result<PathBuf, LockError> {
    let home = config_home()?;
    Ok(home
        .join("orbexa/locks")
        .join(format!("{}.toml", slug(workspace))))
}
pub fn load_lock(path: impl AsRef<Path>) -> Result<LockFile, LockError> {
    let p = path.as_ref();
    if !p.exists() {
        return Ok(LockFile {
            schema: "orbexa/lock@2".into(),
            pages: vec![],
        });
    }
    let lock: LockFile = toml::from_str(&fs::read_to_string(p)?)?;
    if lock.schema != "orbexa/lock@2" {
        return Err(LockError::InvalidSchema(lock.schema));
    }
    Ok(lock)
}
pub fn write_lock(path: impl AsRef<Path>, lock: &LockFile) -> Result<PathBuf, LockError> {
    let p = path.as_ref().to_path_buf();
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&p, format!("{}\n", toml::to_string_pretty(lock)?))?;
    Ok(p)
}
pub fn locked_page<'a>(lock: &'a LockFile, id: &str) -> Option<&'a LockedPage> {
    lock.pages.iter().find(|p| p.codexa_id == id)
}
pub fn upsert_locked_page(
    lock: &mut LockFile,
    artifact: &NotionPageArtifact,
    page_id: impl Into<String>,
    url: Option<String>,
    rendered_hash: impl Into<String>,
) {
    let entry = LockedPage {
        codexa_id: artifact.document.id.clone(),
        root: artifact.target.root.clone(),
        notion_page_id: page_id.into(),
        notion_page_url: url,
        source_repository: artifact.source.repository.clone(),
        source_path: artifact.source.path.clone(),
        source_commit: artifact.source.commit.clone(),
        source_content_hash: artifact.source.content_hash.clone(),
        rendered_content_hash: rendered_hash.into(),
    };
    if let Some(old) = lock
        .pages
        .iter_mut()
        .find(|p| p.codexa_id == entry.codexa_id)
    {
        *old = entry
    } else {
        lock.pages.push(entry)
    }
}
pub fn clear_root(lock: &mut LockFile, root: &str) -> usize {
    let before = lock.pages.len();
    lock.pages.retain(|p| p.root != root);
    before - lock.pages.len()
}
fn config_home() -> Result<PathBuf, LockError> {
    if let Ok(v) = env::var("XDG_CONFIG_HOME") {
        if !v.trim().is_empty() {
            return Ok(PathBuf::from(v));
        }
    }
    let h = env::var("HOME").map_err(|_| LockError::MissingHome)?;
    Ok(PathBuf::from(h).join(".config"))
}
fn slug(v: &str) -> String {
    v.trim()
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
pub enum LockError {
    Io(io::Error),
    TomlDeserialize(toml::de::Error),
    TomlSerialize(toml::ser::Error),
    InvalidSchema(String),
    MissingHome,
}
impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "lock I/O error: {e}"),
            Self::TomlDeserialize(e) => write!(f, "lock TOML error: {e}"),
            Self::TomlSerialize(e) => write!(f, "lock TOML serialization error: {e}"),
            Self::InvalidSchema(s) => write!(f, "unsupported lock schema `{s}`"),
            Self::MissingHome => write!(f, "HOME is not set and no XDG config path is available"),
        }
    }
}
impl std::error::Error for LockError {}
impl From<io::Error> for LockError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}
impl From<toml::de::Error> for LockError {
    fn from(v: toml::de::Error) -> Self {
        Self::TomlDeserialize(v)
    }
}
impl From<toml::ser::Error> for LockError {
    fn from(v: toml::ser::Error) -> Self {
        Self::TomlSerialize(v)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_empty() {
        let l: LockFile = toml::from_str("schema = \"orbexa/lock@2\"\n").unwrap();
        assert!(l.pages.is_empty());
    }
}
