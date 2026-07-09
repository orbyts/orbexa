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
    use super::Page;

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
}
