use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Codexa Notion artifact manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionManifest {
    pub schema: String,
    pub producer: String,
    pub producer_version: String,
    pub pages: Vec<NotionManifestPage>,
}

/// One page entry in a Codexa Notion manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionManifestPage {
    pub document_id: String,
    pub path: String,
}

/// Codexa Notion page artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionPageArtifact {
    pub schema: String,
    pub producer: String,
    pub producer_version: String,
    pub document: ArtifactDocument,
    #[serde(default)]
    pub navigation: ArtifactNavigation,
    pub source: ArtifactSource,
    pub target: ArtifactTarget,
    #[serde(default)]
    pub web: Option<ArtifactWeb>,
    pub content: ArtifactContent,
}

/// User-facing document metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactDocument {
    pub schema: String,
    pub id: String,
    pub title: String,
    pub description: String,
    pub kind: String,
    pub status: String,
    pub visibility: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Shared tree/navigation placement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactNavigation {
    pub root: String,
    pub product: String,
    pub section: Option<String>,
    pub order: Option<i64>,
}

impl Default for ArtifactNavigation {
    fn default() -> Self {
        Self {
            root: "knowledge".into(),
            product: "general".into(),
            section: None,
            order: None,
        }
    }
}

/// Source provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactSource {
    pub repository: String,
    pub path: String,
    pub commit: String,
    pub content_hash: String,
}

/// Target placement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactTarget {
    pub workspace: String,
    pub data_source: String,
}

/// Website placement metadata emitted by Codexa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactWeb {
    pub collection: String,
    pub slug: String,
}

/// Page body content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactContent {
    pub format: String,
    pub markdown: String,
}

/// Loaded manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedManifest {
    pub path: PathBuf,
    pub manifest: NotionManifest,
}

/// Loaded page artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedPageArtifact {
    pub path: PathBuf,
    pub artifact: NotionPageArtifact,
}

/// Loads the Codexa Notion manifest from an artifact directory.
pub fn load_manifest(input_dir: impl AsRef<Path>) -> Result<LoadedManifest, ArtifactError> {
    let path = input_dir.as_ref().join("manifest.json");
    let source = fs::read_to_string(&path)?;
    let manifest: NotionManifest = serde_json::from_str(&source)?;

    if manifest.schema != "codexa.notion.manifest@1" {
        return Err(ArtifactError::InvalidSchema(manifest.schema));
    }

    Ok(LoadedManifest { path, manifest })
}

/// Loads one page artifact from a manifest-relative path.
pub fn load_page_artifact(
    input_dir: impl AsRef<Path>,
    page_path: &str,
) -> Result<LoadedPageArtifact, ArtifactError> {
    let path = input_dir.as_ref().join(page_path);
    let source = fs::read_to_string(&path)?;
    let artifact: NotionPageArtifact = serde_json::from_str(&source)?;

    if artifact.schema != "codexa.notion.page@1" {
        return Err(ArtifactError::InvalidSchema(artifact.schema));
    }

    Ok(LoadedPageArtifact { path, artifact })
}

/// Artifact loading errors.
#[derive(Debug)]
pub enum ArtifactError {
    Io(io::Error),
    Json(serde_json::Error),
    InvalidSchema(String),
}

impl std::fmt::Display for ArtifactError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "artifact I/O error: {error}"),
            Self::Json(error) => write!(formatter, "artifact JSON error: {error}"),
            Self::InvalidSchema(schema) => {
                write!(formatter, "unsupported artifact schema `{schema}`")
            }
        }
    }
}

impl std::error::Error for ArtifactError {}

impl From<io::Error> for ArtifactError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ArtifactError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::NotionPageArtifact;

    #[test]
    fn parses_page_artifact_shape() {
        let json = r##"
{
  "schema": "codexa.notion.page@1",
  "producer": "codexa",
  "producer_version": "0.0.2",
  "document": {
    "schema": "codexa.document@1",
    "id": "notes.example",
    "title": "Example",
    "description": "One line.",
    "kind": "note",
    "status": "active",
    "visibility": "private",
    "tags": ["test"]
  },
  "navigation": {
    "root": "docs",
    "product": "codexa",
    "section": "Guides",
    "order": 10
  },
  "source": {
    "repository": "archivora/knowledge",
    "path": "notes/example.md",
    "commit": "abc123",
    "content_hash": "sha256:abc"
  },
  "target": {
    "workspace": "codexa",
    "data_source": "documents"
  },
  "content": {
    "format": "markdown",
    "markdown": "# Example"
  }
}
"##;

        let artifact: NotionPageArtifact =
            serde_json::from_str(json).expect("artifact should parse");

        assert_eq!(artifact.document.id, "notes.example");
        assert_eq!(artifact.document.description, "One line.");
        assert_eq!(artifact.navigation.root, "docs");
        assert_eq!(artifact.navigation.product, "codexa");
        assert_eq!(artifact.target.data_source, "documents");
    }
}
