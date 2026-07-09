use serde::{Deserialize, Serialize};

use crate::config::{WorkspaceCover, WorkspaceIcon};

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

    /// Creates a child page under an existing page.
    pub fn create_child_page(
        &self,
        parent_page_id: &str,
        title: &str,
        icon: &WorkspaceIcon,
        cover: &WorkspaceCover,
    ) -> Result<Page, NotionError> {
        let request = CreatePageRequest {
            parent: PageParent {
                parent_type: "page_id",
                page_id: parent_page_id,
            },
            properties: PageTitleProperties {
                title: TitleProperty {
                    title: vec![RichText {
                        text: TextContent { content: title },
                    }],
                },
            },
            icon: PageIcon::from(icon),
            cover: PageCover::from(cover),
        };

        let response = self
            .http
            .post("https://api.notion.com/v1/pages")
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
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

    /// Retrieves a Notion database by ID.
    pub fn retrieve_database(&self, database_id: &str) -> Result<Database, NotionError> {
        let response = self
            .http
            .get(format!("https://api.notion.com/v1/databases/{database_id}"))
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

        let database = serde_json::from_str(&text)?;
        Ok(database)
    }

    /// Creates a document page under a Notion data source.
    pub fn create_document_page(
        &self,
        data_source_id: &str,
        title: &str,
        description: &str,
        kind: &str,
        tags: &[String],
        status: &str,
        markdown: &str,
    ) -> Result<Page, NotionError> {
        let request = CreateDocumentPageRequest {
            parent: DataSourceParent { data_source_id },
            properties: DocumentPageProperties {
                name: TitlePropertyValue {
                    title: vec![RichText {
                        text: TextContent { content: title },
                    }],
                },
                description: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: description,
                        },
                    }],
                },
                kind: SelectPropertyValue {
                    select: SelectOption { name: kind },
                },
                tags: MultiSelectPropertyValue {
                    multi_select: tags
                        .iter()
                        .map(|tag| SelectOption { name: tag.as_str() })
                        .collect(),
                },
                status: SelectPropertyValue {
                    select: SelectOption { name: status },
                },
            },
            markdown,
        };

        let response = self
            .http
            .post("https://api.notion.com/v1/pages")
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
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

    /// Creates a database under an existing page with an initial data source.
    pub fn create_database(
        &self,
        parent_page_id: &str,
        title: &str,
        data_source_name: &str,
    ) -> Result<Database, NotionError> {
        let request = CreateDatabaseRequest {
            parent: PageParent {
                parent_type: "page_id",
                page_id: parent_page_id,
            },
            title: vec![RichText {
                text: TextContent { content: title },
            }],
            description: vec![RichText {
                text: TextContent {
                    content: "Managed by Orbexa from Codexa artifacts.",
                },
            }],
            is_inline: true,
            initial_data_source: InitialDataSource {
                title: vec![RichText {
                    text: TextContent {
                        content: data_source_name,
                    },
                }],
                properties: DocumentProperties::default(),
            },
        };

        let response = self
            .http
            .post("https://api.notion.com/v1/databases")
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
            .send()?;

        let status = response.status();
        let text = response.text()?;

        if !status.is_success() {
            return Err(NotionError::Api {
                status: status.as_u16(),
                body: text,
            });
        }

        let database = serde_json::from_str(&text)?;
        Ok(database)
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

#[derive(Debug, Serialize)]
struct CreateDocumentPageRequest<'a> {
    parent: DataSourceParent<'a>,
    properties: DocumentPageProperties<'a>,
    markdown: &'a str,
}

#[derive(Debug, Serialize)]
struct DataSourceParent<'a> {
    data_source_id: &'a str,
}

#[derive(Debug, Serialize)]
struct DocumentPageProperties<'a> {
    #[serde(rename = "Name")]
    name: TitlePropertyValue<'a>,
    #[serde(rename = "Description")]
    description: RichTextPropertyValue<'a>,
    #[serde(rename = "Kind")]
    kind: SelectPropertyValue<'a>,
    #[serde(rename = "Tags")]
    tags: MultiSelectPropertyValue<'a>,
    #[serde(rename = "Status")]
    status: SelectPropertyValue<'a>,
}

#[derive(Debug, Serialize)]
struct TitlePropertyValue<'a> {
    title: Vec<RichText<'a>>,
}

#[derive(Debug, Serialize)]
struct RichTextPropertyValue<'a> {
    rich_text: Vec<RichText<'a>>,
}

#[derive(Debug, Serialize)]
struct SelectPropertyValue<'a> {
    select: SelectOption<'a>,
}

#[derive(Debug, Serialize)]
struct MultiSelectPropertyValue<'a> {
    multi_select: Vec<SelectOption<'a>>,
}

#[derive(Debug, Serialize)]
struct SelectOption<'a> {
    name: &'a str,
}

#[derive(Debug, Serialize)]
struct CreateDatabaseRequest<'a> {
    parent: PageParent<'a>,
    title: Vec<RichText<'a>>,
    description: Vec<RichText<'a>>,
    is_inline: bool,
    initial_data_source: InitialDataSource<'a>,
}

#[derive(Debug, Serialize)]
struct InitialDataSource<'a> {
    title: Vec<RichText<'a>>,
    properties: DocumentProperties,
}

#[derive(Debug, Serialize)]
struct DocumentProperties {
    #[serde(rename = "Name")]
    name: TitlePropertySchema,
    #[serde(rename = "Description")]
    description: RichTextPropertySchema,
    #[serde(rename = "Kind")]
    kind: SelectPropertySchema,
    #[serde(rename = "Tags")]
    tags: MultiSelectPropertySchema,
    #[serde(rename = "Status")]
    status: SelectPropertySchema,
}

impl Default for DocumentProperties {
    fn default() -> Self {
        Self {
            name: TitlePropertySchema::default(),
            description: RichTextPropertySchema::default(),
            kind: SelectPropertySchema::default(),
            tags: MultiSelectPropertySchema::default(),
            status: SelectPropertySchema::default(),
        }
    }
}

#[derive(Debug, Default, Serialize)]
struct TitlePropertySchema {
    title: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Default, Serialize)]
struct RichTextPropertySchema {
    rich_text: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Default, Serialize)]
struct SelectPropertySchema {
    select: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Default, Serialize)]
struct MultiSelectPropertySchema {
    multi_select: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct CreatePageRequest<'a> {
    parent: PageParent<'a>,
    properties: PageTitleProperties<'a>,
    icon: PageIcon<'a>,
    cover: PageCover<'a>,
}

#[derive(Debug, Serialize)]
struct PageParent<'a> {
    #[serde(rename = "type")]
    parent_type: &'static str,
    page_id: &'a str,
}

#[derive(Debug, Serialize)]
struct PageTitleProperties<'a> {
    title: TitleProperty<'a>,
}

#[derive(Debug, Serialize)]
struct TitleProperty<'a> {
    title: Vec<RichText<'a>>,
}

#[derive(Debug, Serialize)]
struct RichText<'a> {
    text: TextContent<'a>,
}

#[derive(Debug, Serialize)]
struct TextContent<'a> {
    content: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PageIcon<'a> {
    Emoji { emoji: &'a str },
    Icon { icon: NotionNativeIcon<'a> },
    External { external: ExternalFile<'a> },
}

impl<'a> From<&'a WorkspaceIcon> for PageIcon<'a> {
    fn from(icon: &'a WorkspaceIcon) -> Self {
        match icon {
            WorkspaceIcon::Emoji { emoji } => Self::Emoji { emoji },
            WorkspaceIcon::Icon { name, color } => Self::Icon {
                icon: NotionNativeIcon { name, color },
            },
            WorkspaceIcon::External { url } => Self::External {
                external: ExternalFile { url },
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct NotionNativeIcon<'a> {
    name: &'a str,
    color: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum PageCover<'a> {
    External { external: ExternalFile<'a> },
}

impl<'a> From<&'a WorkspaceCover> for PageCover<'a> {
    fn from(cover: &'a WorkspaceCover) -> Self {
        match cover {
            WorkspaceCover::External { url } => Self::External {
                external: ExternalFile { url },
            },
        }
    }
}

#[derive(Debug, Serialize)]
struct ExternalFile<'a> {
    url: &'a str,
}

/// Minimal database response shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Database {
    pub object: String,
    pub id: String,
    #[serde(default)]
    pub in_trash: bool,
    pub data_sources: Vec<DatabaseDataSource>,
    pub url: Option<String>,
}

/// Minimal child data source reference in a database response.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatabaseDataSource {
    pub id: String,
    pub name: String,
}

impl Database {
    /// Returns the first data source matching the configured name.
    #[must_use]
    pub fn data_source_named(&self, name: &str) -> Option<&DatabaseDataSource> {
        self.data_sources.iter().find(|source| source.name == name)
    }
}

/// Minimal page response shape.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Page {
    pub object: String,
    pub id: String,
    #[serde(default)]
    pub in_trash: bool,
    pub properties: serde_json::Value,
    pub url: Option<String>,
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
    use super::*;

    use super::{Block, Database, Page, PageCover, PageIcon};
    use crate::config::{WorkspaceCover, WorkspaceIcon};

    #[test]
    fn serializes_document_page_properties() {
        let request = CreateDocumentPageRequest {
            parent: DataSourceParent {
                data_source_id: "data-source-id",
            },
            properties: DocumentPageProperties {
                name: TitlePropertyValue {
                    title: vec![RichText {
                        text: TextContent { content: "Title" },
                    }],
                },
                description: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: "Description",
                        },
                    }],
                },
                kind: SelectPropertyValue {
                    select: SelectOption { name: "playbook" },
                },
                tags: MultiSelectPropertyValue {
                    multi_select: vec![SelectOption { name: "lureva" }],
                },
                status: SelectPropertyValue {
                    select: SelectOption { name: "active" },
                },
            },
            markdown: "# Body",
        };

        let json = serde_json::to_value(request).expect("request should serialize");

        assert_eq!(json["parent"]["data_source_id"], "data-source-id");
        assert_eq!(
            json["properties"]["Name"]["title"][0]["text"]["content"],
            "Title"
        );
        assert_eq!(
            json["properties"]["Description"]["rich_text"][0]["text"]["content"],
            "Description"
        );
        assert_eq!(json["properties"]["Kind"]["select"]["name"], "playbook");
        assert_eq!(
            json["properties"]["Tags"]["multi_select"][0]["name"],
            "lureva"
        );
        assert_eq!(json["properties"]["Status"]["select"]["name"], "active");
        assert_eq!(json["markdown"], "# Body");
    }

    #[test]
    fn extracts_database_data_source() {
        let json = r#"
{
  "object": "database",
  "id": "database-id",
  "url": "https://app.notion.com/database-id",
  "data_sources": [
    {
      "id": "data-source-id",
      "name": "Documents"
    }
  ]
}
"#;

        let database: Database = serde_json::from_str(json).expect("database should parse");

        assert_eq!(database.id, "database-id");
        assert_eq!(
            database
                .data_source_named("Documents")
                .map(|source| source.id.as_str()),
            Some("data-source-id")
        );
    }

    #[test]
    fn extracts_page_title() {
        let json = r#"
{
  "object": "page",
  "id": "398a1865-b187-802a-a885-d97afc99896f",
  "url": "https://app.notion.com/p/Codexa-Test",
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

    #[test]
    fn serializes_native_icon() {
        let workspace_icon = WorkspaceIcon::Icon {
            name: "book".into(),
            color: "lightgray".into(),
        };
        let icon = PageIcon::from(&workspace_icon);

        let json = serde_json::to_value(icon).expect("icon should serialize");

        assert_eq!(json["type"], "icon");
        assert_eq!(json["icon"]["name"], "book");
        assert_eq!(json["icon"]["color"], "lightgray");
    }

    #[test]
    fn serializes_external_cover() {
        let workspace_cover = WorkspaceCover::External {
            url: "https://example.com/cover.jpg".into(),
        };
        let cover = PageCover::from(&workspace_cover);

        let json = serde_json::to_value(cover).expect("cover should serialize");

        assert_eq!(json["type"], "external");
        assert_eq!(json["external"]["url"], "https://example.com/cover.jpg");
    }
}
