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
        document: &CreateDocumentPage<'_>,
    ) -> Result<Page, NotionError> {
        let request = CreateDocumentPageRequest {
            parent: DataSourceParent {
                data_source_id: document.data_source_id,
            },
            properties: DocumentPageProperties {
                document_id: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: document.document_id,
                        },
                    }],
                },
                sort_order: NumberPropertyValue {
                    number: document.sort_order,
                },
                name: TitlePropertyValue {
                    title: vec![RichText {
                        text: TextContent {
                            content: document.title,
                        },
                    }],
                },
                description: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: document.description,
                        },
                    }],
                },
                root: SelectPropertyValue {
                    select: SelectOption {
                        name: document.root,
                    },
                },
                product: SelectPropertyValue {
                    select: SelectOption {
                        name: document.product,
                    },
                },
                section: SelectPropertyValue {
                    select: SelectOption {
                        name: document.section,
                    },
                },
                kind: SelectPropertyValue {
                    select: SelectOption {
                        name: document.kind,
                    },
                },
                tags: MultiSelectPropertyValue {
                    multi_select: document
                        .tags
                        .iter()
                        .map(|tag| SelectOption { name: tag.as_str() })
                        .collect(),
                },
                status: SelectPropertyValue {
                    select: SelectOption {
                        name: document.status,
                    },
                },
                visibility: SelectPropertyValue {
                    select: SelectOption {
                        name: document.visibility,
                    },
                },
            },
            markdown: document.markdown,
            icon: PageIcon::from(document.icon),
            cover: PageCover::from(document.cover),
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

    /// Updates the managed properties, icon, and cover of a document page.
    pub fn update_document_page(
        &self,
        page_id: &str,
        document: &UpdateDocumentPage<'_>,
    ) -> Result<Page, NotionError> {
        let request = UpdateDocumentPageRequest {
            properties: DocumentPageProperties {
                document_id: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: document.document_id,
                        },
                    }],
                },
                sort_order: NumberPropertyValue {
                    number: document.sort_order,
                },
                name: TitlePropertyValue {
                    title: vec![RichText {
                        text: TextContent {
                            content: document.title,
                        },
                    }],
                },
                description: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: document.description,
                        },
                    }],
                },
                root: SelectPropertyValue {
                    select: SelectOption {
                        name: document.root,
                    },
                },
                product: SelectPropertyValue {
                    select: SelectOption {
                        name: document.product,
                    },
                },
                section: SelectPropertyValue {
                    select: SelectOption {
                        name: document.section,
                    },
                },
                kind: SelectPropertyValue {
                    select: SelectOption {
                        name: document.kind,
                    },
                },
                tags: MultiSelectPropertyValue {
                    multi_select: document
                        .tags
                        .iter()
                        .map(|tag| SelectOption { name: tag.as_str() })
                        .collect(),
                },
                status: SelectPropertyValue {
                    select: SelectOption {
                        name: document.status,
                    },
                },
                visibility: SelectPropertyValue {
                    select: SelectOption {
                        name: document.visibility,
                    },
                },
            },
            icon: PageIcon::from(document.icon),
            cover: PageCover::from(document.cover),
        };

        let response = self
            .http
            .patch(format!("https://api.notion.com/v1/pages/{page_id}"))
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
            .send()?;

        parse_response(response)
    }

    /// Replaces all page body content using Notion-flavored Markdown.
    pub fn replace_page_markdown(
        &self,
        page_id: &str,
        markdown: &str,
    ) -> Result<PageMarkdown, NotionError> {
        let request = ReplacePageMarkdownRequest {
            request_type: "replace_content",
            replace_content: ReplaceContent {
                new_str: markdown,
                allow_deleting_content: false,
            },
        };

        let response = self
            .http
            .patch(format!(
                "https://api.notion.com/v1/pages/{page_id}/markdown"
            ))
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
            .send()?;

        parse_response(response)
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

    /// Retrieves a Notion data source by ID.
    pub fn retrieve_data_source(&self, data_source_id: &str) -> Result<DataSource, NotionError> {
        let response = self
            .http
            .get(format!(
                "https://api.notion.com/v1/data_sources/{data_source_id}"
            ))
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .send()?;

        parse_response(response)
    }

    /// Adds any missing managed properties and rejects incompatible property types.
    pub fn ensure_document_schema(&self, data_source_id: &str) -> Result<Vec<String>, NotionError> {
        let data_source = self.retrieve_data_source(data_source_id)?;
        let expected = managed_property_types();
        let mut missing = std::collections::BTreeMap::new();
        let mut added = Vec::new();

        for (name, expected_type) in expected.iter().copied() {
            match data_source.properties.get(name) {
                Some(property) if property.property_type == expected_type => {}
                Some(property) => {
                    return Err(NotionError::SchemaDrift {
                        property: name.to_owned(),
                        expected: expected_type.to_owned(),
                        actual: property.property_type.clone(),
                    });
                }
                None => {
                    missing.insert(name.to_owned(), property_schema(expected_type));
                    added.push(name.to_owned());
                }
            }
        }

        if missing.is_empty() {
            return Ok(added);
        }

        let response = self
            .http
            .patch(format!(
                "https://api.notion.com/v1/data_sources/{data_source_id}"
            ))
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&UpdateDataSourceRequest {
                properties: missing,
            })
            .send()?;
        let _: DataSource = parse_response(response)?;
        Ok(added)
    }

    /// Finds all live pages whose Document ID exactly matches the supplied value.
    pub fn query_pages_by_document_id(
        &self,
        data_source_id: &str,
        document_id: &str,
    ) -> Result<Vec<Page>, NotionError> {
        let request = QueryDataSourceRequest {
            filter: QueryFilter {
                property: "Document ID",
                rich_text: RichTextEquals {
                    equals: document_id,
                },
            },
            page_size: 100,
        };
        let response = self
            .http
            .post(format!(
                "https://api.notion.com/v1/data_sources/{data_source_id}/query"
            ))
            .bearer_auth(&self.token)
            .header("Notion-Version", &self.api_version)
            .json(&request)
            .send()?;
        let result: QueryDataSourceResponse = parse_response(response)?;
        Ok(result
            .results
            .into_iter()
            .filter(|page| !page.in_trash)
            .collect())
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

/// Parameters for updating a managed Notion document page.
pub struct UpdateDocumentPage<'a> {
    pub document_id: &'a str,
    pub sort_order: i64,
    pub title: &'a str,
    pub description: &'a str,
    pub root: &'a str,
    pub product: &'a str,
    pub section: &'a str,
    pub kind: &'a str,
    pub tags: &'a [String],
    pub status: &'a str,
    pub visibility: &'a str,
    pub icon: &'a WorkspaceIcon,
    pub cover: &'a WorkspaceCover,
}

/// Parameters for creating a managed Notion document page.
pub struct CreateDocumentPage<'a> {
    pub data_source_id: &'a str,
    pub document_id: &'a str,
    pub sort_order: i64,
    pub title: &'a str,
    pub description: &'a str,
    pub root: &'a str,
    pub product: &'a str,
    pub section: &'a str,
    pub kind: &'a str,
    pub tags: &'a [String],
    pub status: &'a str,
    pub visibility: &'a str,
    pub markdown: &'a str,
    pub icon: &'a WorkspaceIcon,
    pub cover: &'a WorkspaceCover,
}

fn parse_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::blocking::Response,
) -> Result<T, NotionError> {
    let status = response.status();
    let text = response.text()?;
    if !status.is_success() {
        return Err(NotionError::Api {
            status: status.as_u16(),
            body: text,
        });
    }
    Ok(serde_json::from_str(&text)?)
}

#[derive(Debug, Serialize)]
struct UpdateDocumentPageRequest<'a> {
    properties: DocumentPageProperties<'a>,
    icon: PageIcon<'a>,
    cover: PageCover<'a>,
}

#[derive(Debug, Serialize)]
struct ReplacePageMarkdownRequest<'a> {
    #[serde(rename = "type")]
    request_type: &'a str,
    replace_content: ReplaceContent<'a>,
}

#[derive(Debug, Serialize)]
struct ReplaceContent<'a> {
    new_str: &'a str,
    allow_deleting_content: bool,
}

#[derive(Debug, Serialize)]
struct CreateDocumentPageRequest<'a> {
    parent: DataSourceParent<'a>,
    properties: DocumentPageProperties<'a>,
    markdown: &'a str,
    icon: PageIcon<'a>,
    cover: PageCover<'a>,
}

#[derive(Debug, Serialize)]
struct DataSourceParent<'a> {
    data_source_id: &'a str,
}

#[derive(Debug, Serialize)]
struct DocumentPageProperties<'a> {
    #[serde(rename = "Document ID")]
    document_id: RichTextPropertyValue<'a>,
    #[serde(rename = "Sort Order")]
    sort_order: NumberPropertyValue,
    #[serde(rename = "Name")]
    name: TitlePropertyValue<'a>,
    #[serde(rename = "Description")]
    description: RichTextPropertyValue<'a>,
    #[serde(rename = "Root")]
    root: SelectPropertyValue<'a>,
    #[serde(rename = "Product")]
    product: SelectPropertyValue<'a>,
    #[serde(rename = "Section")]
    section: SelectPropertyValue<'a>,
    #[serde(rename = "Kind")]
    kind: SelectPropertyValue<'a>,
    #[serde(rename = "Tags")]
    tags: MultiSelectPropertyValue<'a>,
    #[serde(rename = "Status")]
    status: SelectPropertyValue<'a>,
    #[serde(rename = "Visibility")]
    visibility: SelectPropertyValue<'a>,
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
struct NumberPropertyValue {
    number: i64,
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

#[derive(Debug, Default, Serialize)]
struct DocumentProperties {
    #[serde(rename = "Document ID")]
    document_id: RichTextPropertySchema,
    #[serde(rename = "Sort Order")]
    sort_order: NumberPropertySchema,
    #[serde(rename = "Name")]
    name: TitlePropertySchema,
    #[serde(rename = "Description")]
    description: RichTextPropertySchema,
    #[serde(rename = "Root")]
    root: SelectPropertySchema,
    #[serde(rename = "Product")]
    product: SelectPropertySchema,
    #[serde(rename = "Section")]
    section: SelectPropertySchema,
    #[serde(rename = "Kind")]
    kind: SelectPropertySchema,
    #[serde(rename = "Tags")]
    tags: MultiSelectPropertySchema,
    #[serde(rename = "Status")]
    status: SelectPropertySchema,
    #[serde(rename = "Visibility")]
    visibility: SelectPropertySchema,
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
struct NumberPropertySchema {
    number: std::collections::BTreeMap<String, String>,
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

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DataSource {
    pub object: String,
    pub id: String,
    #[serde(default)]
    pub in_trash: bool,
    #[serde(default)]
    pub properties: std::collections::BTreeMap<String, DataSourceProperty>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DataSourceProperty {
    #[serde(rename = "type")]
    pub property_type: String,
}

impl DataSource {
    pub fn missing_managed_properties(&self) -> Result<Vec<String>, NotionError> {
        let mut missing = Vec::new();
        for (name, expected_type) in managed_property_types().iter().copied() {
            match self.properties.get(name) {
                Some(property) if property.property_type == expected_type => {}
                Some(property) => {
                    return Err(NotionError::SchemaDrift {
                        property: name.to_owned(),
                        expected: expected_type.to_owned(),
                        actual: property.property_type.clone(),
                    });
                }
                None => missing.push(name.to_owned()),
            }
        }
        Ok(missing)
    }
}

#[derive(Debug, Serialize)]
struct UpdateDataSourceRequest {
    properties: std::collections::BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct QueryDataSourceRequest<'a> {
    filter: QueryFilter<'a>,
    page_size: u16,
}

#[derive(Debug, Serialize)]
struct QueryFilter<'a> {
    property: &'a str,
    rich_text: RichTextEquals<'a>,
}

#[derive(Debug, Serialize)]
struct RichTextEquals<'a> {
    equals: &'a str,
}

#[derive(Debug, Deserialize)]
struct QueryDataSourceResponse {
    results: Vec<Page>,
}

fn managed_property_types() -> &'static [(&'static str, &'static str)] {
    &[
        ("Name", "title"),
        ("Document ID", "rich_text"),
        ("Sort Order", "number"),
        ("Description", "rich_text"),
        ("Root", "select"),
        ("Product", "select"),
        ("Section", "select"),
        ("Kind", "select"),
        ("Tags", "multi_select"),
        ("Status", "select"),
        ("Visibility", "select"),
    ]
}

fn property_schema(property_type: &str) -> serde_json::Value {
    let mut schema = serde_json::Map::new();
    schema.insert(property_type.to_owned(), serde_json::json!({}));
    serde_json::Value::Object(schema)
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

/// Page content returned by Notion's Markdown content endpoint.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct PageMarkdown {
    pub id: String,
    pub markdown: String,
    #[serde(default)]
    pub truncated: bool,
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
            let Some(title_items) = property.get("title").and_then(|value| value.as_array()) else {
                continue;
            };

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
    Api {
        status: u16,
        body: String,
    },
    SchemaDrift {
        property: String,
        expected: String,
        actual: String,
    },
}

impl std::fmt::Display for NotionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Http(error) => write!(formatter, "Notion HTTP error: {error}"),
            Self::Json(error) => write!(formatter, "Notion JSON error: {error}"),
            Self::Api { status, body } => {
                write!(formatter, "Notion API error {status}: {body}")
            }
            Self::SchemaDrift {
                property,
                expected,
                actual,
            } => write!(
                formatter,
                "Notion schema drift for `{property}`: expected `{expected}`, found `{actual}`"
            ),
        }
    }
}

impl NotionError {
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::Api { status: 404, .. })
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
                document_id: RichTextPropertyValue {
                    rich_text: vec![RichText {
                        text: TextContent {
                            content: "lureva.playbooks.handoff",
                        },
                    }],
                },
                sort_order: NumberPropertyValue { number: 100 },
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
                root: SelectPropertyValue {
                    select: SelectOption { name: "knowledge" },
                },
                product: SelectPropertyValue {
                    select: SelectOption { name: "lureva" },
                },
                section: SelectPropertyValue {
                    select: SelectOption { name: "Playbooks" },
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
                visibility: SelectPropertyValue {
                    select: SelectOption { name: "private" },
                },
            },
            markdown: "# Body",
            icon: PageIcon::Emoji { emoji: "📘" },
            cover: PageCover::External {
                external: ExternalFile {
                    url: "https://example.com/cover.png",
                },
            },
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
        assert_eq!(
            json["properties"]["Document ID"]["rich_text"][0]["text"]["content"],
            "lureva.playbooks.handoff"
        );
        assert_eq!(json["properties"]["Sort Order"]["number"], 100);
        assert_eq!(json["properties"]["Root"]["select"]["name"], "knowledge");
        assert_eq!(json["properties"]["Product"]["select"]["name"], "lureva");
        assert_eq!(json["properties"]["Section"]["select"]["name"], "Playbooks");
        assert_eq!(json["properties"]["Kind"]["select"]["name"], "playbook");
        assert_eq!(
            json["properties"]["Tags"]["multi_select"][0]["name"],
            "lureva"
        );
        assert_eq!(json["properties"]["Status"]["select"]["name"], "active");
        assert_eq!(
            json["properties"]["Visibility"]["select"]["name"],
            "private"
        );
        assert_eq!(json["markdown"], "# Body");
        assert_eq!(json["icon"]["emoji"], "📘");
        assert_eq!(
            json["cover"]["external"]["url"],
            "https://example.com/cover.png"
        );
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
    fn extracts_page_title_when_non_title_properties_come_first() {
        let json = r#"
{
  "object": "page",
  "id": "399a1865-b187-81e1-989a-cd03ee23b483",
  "url": "https://app.notion.com/p/Lureva-Lightroom-Handoff-Manual",
  "properties": {
    "Description": {
      "id": "description",
      "type": "rich_text",
      "rich_text": [
        {
          "plain_text": "Daily Lightroom Classic handoff workflow."
        }
      ]
    },
    "Root": {
      "id": "root",
      "type": "select",
      "select": {
        "name": "knowledge"
      }
    },
    "Name": {
      "id": "title",
      "type": "title",
      "title": [
        {
          "plain_text": "Lureva Lightroom Handoff Manual"
        }
      ]
    }
  }
}
"#;

        let page: Page = serde_json::from_str(json).expect("page should parse");
        assert_eq!(
            page.title().as_deref(),
            Some("Lureva Lightroom Handoff Manual")
        );
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
