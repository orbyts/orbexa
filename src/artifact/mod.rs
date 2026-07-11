use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionManifest {
    pub schema: String,
    pub producer: String,
    pub producer_version: String,
    pub pages: Vec<NotionManifestPage>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionManifestPage {
    pub document_id: String,
    pub root: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionPageArtifact {
    pub schema: String,
    pub producer: String,
    pub producer_version: String,
    pub document: ArtifactDocument,
    pub navigation: ArtifactNavigation,
    pub source: ArtifactSource,
    pub target: ArtifactTarget,
    pub web: Option<ArtifactWeb>,
    pub links: Vec<ArtifactLink>,
    pub content: ArtifactContent,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDocument {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: String,
    pub status: String,
    pub visibility: String,
    pub tags: Vec<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactNavigation {
    pub root: String,
    pub product: String,
    pub section: String,
    pub order: i64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSource {
    pub repository: String,
    pub path: String,
    pub commit: String,
    pub content_hash: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTarget {
    pub workspace: String,
    pub root: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactWeb {
    pub slug: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactLink {
    pub raw: String,
    pub target_id: String,
    pub heading: Option<String>,
    pub label: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactContent {
    pub format: String,
    pub markdown: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedManifest {
    pub path: PathBuf,
    pub manifest: NotionManifest,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPageArtifact {
    pub path: PathBuf,
    pub artifact: NotionPageArtifact,
}

pub fn load_manifest(input_dir: impl AsRef<Path>) -> Result<LoadedManifest, ArtifactError> {
    let path = input_dir.as_ref().join("manifest.json");
    let manifest: NotionManifest = serde_json::from_str(&fs::read_to_string(&path)?)?;
    if manifest.schema != "codexa.notion.manifest@2" {
        return Err(ArtifactError::InvalidSchema(manifest.schema));
    }
    Ok(LoadedManifest { path, manifest })
}
pub fn load_page_artifact(
    input_dir: impl AsRef<Path>,
    page_path: &str,
) -> Result<LoadedPageArtifact, ArtifactError> {
    let path = input_dir.as_ref().join(page_path);
    let artifact: NotionPageArtifact = serde_json::from_str(&fs::read_to_string(&path)?)?;
    if artifact.schema != "codexa.notion.page@2" {
        return Err(ArtifactError::InvalidSchema(artifact.schema));
    }
    Ok(LoadedPageArtifact { path, artifact })
}

#[derive(Debug)]
pub enum ArtifactError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidSchema(String),
}
impl std::fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "artifact I/O error: {e}"),
            Self::Json(e) => write!(f, "artifact JSON error: {e}"),
            Self::InvalidSchema(s) => write!(f, "unsupported artifact schema `{s}`"),
        }
    }
}
impl std::error::Error for ArtifactError {}
impl From<io::Error> for ArtifactError {
    fn from(v: io::Error) -> Self {
        Self::Io(v)
    }
}
impl From<serde_json::Error> for ArtifactError {
    fn from(v: serde_json::Error) -> Self {
        Self::Json(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_page_v2() {
        let json = r##"{"schema":"codexa.notion.page@2","producer":"codexa","producer_version":"0.0.2","document":{"schema":"codexa.document@2","id":"codexa.index","title":"Codexa","description":"Docs","kind":"index","status":"active","visibility":"public","tags":[]},"navigation":{"root":"docs","product":"codexa","section":"Overview","order":0},"source":{"repository":"orbyts/codexa","path":"docs/index.md","commit":"abc","content_hash":"sha256:abc"},"target":{"workspace":"codexa","root":"docs"},"web":{"slug":"/docs/codexa"},"links":[],"content":{"format":"markdown","markdown":"# Codexa"}}"##;
        let artifact: NotionPageArtifact = serde_json::from_str(json).unwrap();
        assert_eq!(artifact.target.root, "docs");
    }
}
