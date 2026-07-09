use serde::Deserialize;

/// Minimal Notion API client.
#[derive(Debug, Clone)]
pub struct NotionClient {
    http: reqwest::blocking::Client,
    token: String,
    api_version: String,
}

impl NotionClient {
    /// Creates a client from a token and Notion API version.
    #[must_use]
    pub fn new(token: impl Into<String>, api_version: impl Into<String>) -> Self {
        Self {
            http: reqwest::blocking::Client::new(),
            token: token.into(),
            api_version: api_version.into(),
        }
    }

    /// Retrieves a Notion page by ID.
    pub fn retrieve_page(&self, page_id: &str) -> Result<Page, NotionError> {
        let url = format!("https://api.notion.com/v1/pages/{page_id}");

        let response = self
            .http
            .get(url)
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .send()?;

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            return Err(NotionError::Api {
                status: status.as_u16(),
                body: text,
            });
        }

        let page = serde_json::from_str(&text)?;
        Ok(page)
    }

    /// Retrieves all first-level block children for a block or page.
    pub fn retrieve_block_children(&self, block_id: &str) -> Result<Vec<Block>, NotionError> {
        let mut blocks = Vec::new();
        let mut start_cursor: Option<String> = None;

        loop {
            let url = format!("https://api.notion.com/v1/blocks/{block_id}/children");

            let mut request = self
                .http
                .get(&url)
                .bearer_auth(&self.token)
                .header("Notion-Version", &self.api_version)
                .query(&[("page_size", "100")]);

            if let Some(cursor) = &start_cursor {
                request = request.query(&[("start_cursor", cursor)]);
            }

            let response = request.send()?;
            let status = response.status();
            let text = response.text()?;

            if !status.is_success() {
                return Err(NotionError::Api {
                    status: status.as_u16(),
                    body: text,
                });
            }

            let list: BlockList = serde_json::from_str(&text)?;
            blocks.extend(list.results);

            if !list.has_more {
                break;
            }

            start_cursor = list.next_cursor;
            if start_cursor.is_none() {
                break;
            }
        }

        Ok(blocks)
    }
}

/// Minimal page response shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Page {
    pub object: String,
    pub id: String,
    pub properties: serde_json::Value,
}

impl Page {
    /// Best-effort title extraction for ordinary Notion pages.
    #[must_use]
    pub fn title(&self) -> Option<String> {
        let properties = self.properties.as_object()?;

        for property in properties.values() {
            let title_items = property.get("title")?.as_array()?;
            let mut title = String::new();

            for item in title_items {
                if let Some(text) = item.get("plain_text").and_then(|value| value.as_str()) {
                    title.push_str(text);
                }
            }

            if !title.is_empty() {
                return Some(title);
            }
        }

        None
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
struct BlockList {
    results: Vec<Block>,
    next_cursor: Option<String>,
    has_more: bool,
}

/// Minimal block response shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Block {
    pub object: String,
    pub id: String,
    #[serde(rename = "type")]
    pub block_type: String,
    pub child_page: Option<ChildPage>,
    pub child_database: Option<ChildDatabase>,
}

/// Child page block payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ChildPage {
    pub title: String,
}

/// Child database block payload.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ChildDatabase {
    pub title: String,
}

impl Block {
    /// Returns a display title for supported child blocks.
    #[must_use]
    pub fn title(&self) -> Option<&str> {
        match self.block_type.as_str() {
            "child_page" => self.child_page.as_ref().map(|page| page.title.as_str()),
            "child_database" => self
                .child_database
                .as_ref()
                .map(|database| database.title.as_str()),
            _ => None,
        }
    }
}

/// Notion API errors.
#[derive(Debug)]
pub enum NotionError {
    Http(reqwest::Error),
    Json(serde_json::Error),
    Api { status: u16, body: String },
}

impl std::fmt::Display for NotionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(formatter, "Notion HTTP error: {error}"),
            Self::Json(error) => write!(formatter, "Notion JSON error: {error}"),
            Self::Api { status, body } => {
                write!(formatter, "Notion API error {status}: {body}")
            }
        }
    }
}

impl std::error::Error for NotionError {}

impl From<reqwest::Error> for NotionError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

impl From<serde_json::Error> for NotionError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[cfg(test)]
mod tests {
    use super::{Block, Page};

    #[test]
    fn extracts_page_title() {
        let json = r#"
{
  "object": "page",
  "id": "398a1865-b187-802a-a885-d97afc99896f",
  "properties": {
    "title": {
      "id": "title",
      "type": "title",
      "title": [
        {
          "plain_text": "Codexa Test"
        }
      ]
    }
  }
}
"#;

        let page: Page = serde_json::from_str(json).expect("page should parse");
        assert_eq!(page.title().as_deref(), Some("Codexa Test"));
    }

    #[test]
    fn extracts_child_page_title() {
        let json = r#"
{
  "object": "block",
  "id": "abc",
  "type": "child_page",
  "child_page": {
    "title": "Codexa"
  }
}
"#;

        let block: Block = serde_json::from_str(json).expect("block should parse");
        assert_eq!(block.title(), Some("Codexa"));
    }

    #[test]
    fn extracts_child_database_title() {
        let json = r#"
{
  "object": "block",
  "id": "abc",
  "type": "child_database",
  "child_database": {
    "title": "Knowledge"
  }
}
"#;

        let block: Block = serde_json::from_str(json).expect("block should parse");
        assert_eq!(block.title(), Some("Knowledge"));
    }
}
