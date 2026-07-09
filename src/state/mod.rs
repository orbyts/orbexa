use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Orbexa-generated local state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub schema: String,
    pub notion: NotionState,
}

/// Notion object IDs created or adopted by Orbexa.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotionState {
    pub parent_page_id: String,
    pub workspace_page_id: String,
    pub workspace_page_name: String,
    pub workspace_page_url: Option<String>,
}

impl State {
    /// Creates initial state after workspace page creation.
    #[must_use]
    pub fn workspace_page(
        parent_page_id: impl Into<String>,
        workspace_page_id: impl Into<String>,
        workspace_page_name: impl Into<String>,
        workspace_page_url: Option<String>,
    ) -> Self {
        Self {
            schema: "orbexa/state@1".into(),
            notion: NotionState {
                parent_page_id: parent_page_id.into(),
                workspace_page_id: workspace_page_id.into(),
                workspace_page_name: workspace_page_name.into(),
                workspace_page_url,
            },
        }
    }
}

/// Returns the canonical state file path for a state directory.
#[must_use]
pub fn state_path(state_dir: &Path) -> PathBuf {
    state_dir.join("state.toml")
}

/// Writes state to disk.
pub fn write_state(state_dir: &Path, state: &State) -> Result<PathBuf, StateError> {
    fs::create_dir_all(state_dir)?;
    let path = state_path(state_dir);
    let mut bytes = toml::to_string_pretty(state)?.into_bytes();
    bytes.push(b'\n');
    fs::write(&path, bytes)?;
    Ok(path)
}

/// State errors.
#[derive(Debug)]
pub enum StateError {
    Io(io::Error),
    Toml(toml::ser::Error),
}

impl std::fmt::Display for StateError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "state I/O error: {error}"),
            Self::Toml(error) => write!(formatter, "state TOML error: {error}"),
        }
    }
}

impl std::error::Error for StateError {}

impl From<io::Error> for StateError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<toml::ser::Error> for StateError {
    fn from(error: toml::ser::Error) -> Self {
        Self::Toml(error)
    }
}

#[cfg(test)]
mod tests {
    use super::State;

    #[test]
    fn serializes_state() {
        let state = State::workspace_page(
            "parent",
            "workspace",
            "Codexa",
            Some("https://app.notion.com/p/Codexa".into()),
        );

        let toml = toml::to_string_pretty(&state).expect("state should serialize");

        assert!(toml.contains("schema = \"orbexa/state@1\""));
        assert!(toml.contains("workspace_page_id = \"workspace\""));
        assert!(toml.contains("workspace_page_name = \"Codexa\""));
    }
}
