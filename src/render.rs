//! Target-specific rendering for Codexa logical links.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::artifact::NotionPageArtifact;

/// Stable Notion identity for a Codexa document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotionIdentity {
    pub page_id: String,
    pub page_url: String,
    pub title: String,
}

/// Resolves logical Codexa links into ordinary Markdown links for Notion.
pub fn render_notion_markdown(
    artifact: &NotionPageArtifact,
    identities: &BTreeMap<String, NotionIdentity>,
) -> Result<String, RenderError> {
    let mut rendered = artifact.content.markdown.clone();

    for link in &artifact.links {
        let identity = identities
            .get(&link.target_id)
            .ok_or_else(|| RenderError::MissingTarget(link.target_id.clone()))?;
        let label = link.label.as_deref().unwrap_or(identity.title.as_str());
        let replacement = format!("[{label}]({})", identity.page_url);
        rendered = rendered.replace(&link.raw, &replacement);
    }

    Ok(rendered)
}

/// Computes the lock hash for the final rendered page state.
#[must_use]
pub fn rendered_content_hash(artifact: &NotionPageArtifact, markdown: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(artifact.document.title.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.document.description.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.navigation.root.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.navigation.product.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.document.kind.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.document.status.as_bytes());
    hasher.update([0]);
    hasher.update(artifact.document.visibility.as_bytes());
    hasher.update([0]);
    for tag in &artifact.document.tags {
        hasher.update(tag.as_bytes());
        hasher.update([0]);
    }
    hasher.update(markdown.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

/// Validates that every logical link resolves inside the loaded artifact set.
pub fn validate_link_targets(artifacts: &[NotionPageArtifact]) -> Result<(), RenderError> {
    let ids = artifacts
        .iter()
        .map(|artifact| artifact.document.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();

    for artifact in artifacts {
        for link in &artifact.links {
            if !ids.contains(link.target_id.as_str()) {
                return Err(RenderError::MissingTarget(link.target_id.clone()));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    MissingTarget(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingTarget(target) => {
                write!(
                    formatter,
                    "logical link target `{target}` is not in the artifact bundle"
                )
            }
        }
    }
}

impl std::error::Error for RenderError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{
        ArtifactContent, ArtifactDocument, ArtifactLink, ArtifactNavigation, ArtifactSource,
        ArtifactTarget, NotionPageArtifact,
    };

    fn artifact(markdown: &str, links: Vec<ArtifactLink>) -> NotionPageArtifact {
        NotionPageArtifact {
            schema: "codexa.notion.page@2".into(),
            producer: "codexa".into(),
            producer_version: "0.0.2".into(),
            document: ArtifactDocument {
                schema: "codexa.document@2".into(),
                id: "codexa.guides.quick-start".into(),
                title: "Codexa Quick Start".into(),
                description: "Start here.".into(),
                kind: "guide".into(),
                status: "active".into(),
                visibility: "public".into(),
                tags: vec!["codexa".into()],
            },
            navigation: ArtifactNavigation {
                root: "docs".into(),
                product: "codexa".into(),
                section: "Guides".into(),
                order: 10,
            },
            source: ArtifactSource {
                repository: "orbyts/codexa".into(),
                path: "docs/guides/quick-start.md".into(),
                commit: "abc".into(),
                content_hash: "sha256:source".into(),
            },
            target: ArtifactTarget {
                workspace: "Codexa".into(),
                root: "docs".into(),
            },
            web: None,
            links,
            content: ArtifactContent {
                format: "markdown".into(),
                markdown: markdown.into(),
            },
        }
    }

    #[test]
    fn resolves_labeled_and_unlabeled_links() {
        let artifact = artifact(
            "See [[orbexa.guides.quick-start|Orbexa Quick Start]] and [[codexa.reference.frontmatter]].",
            vec![
                ArtifactLink {
                    raw: "[[orbexa.guides.quick-start|Orbexa Quick Start]]".into(),
                    target_id: "orbexa.guides.quick-start".into(),
                    heading: None,
                    label: Some("Orbexa Quick Start".into()),
                },
                ArtifactLink {
                    raw: "[[codexa.reference.frontmatter]]".into(),
                    target_id: "codexa.reference.frontmatter".into(),
                    heading: None,
                    label: None,
                },
            ],
        );
        let identities = BTreeMap::from([
            (
                "orbexa.guides.quick-start".into(),
                NotionIdentity {
                    page_id: "one".into(),
                    page_url: "https://notion.test/orbexa".into(),
                    title: "Orbexa Quick Start".into(),
                },
            ),
            (
                "codexa.reference.frontmatter".into(),
                NotionIdentity {
                    page_id: "two".into(),
                    page_url: "https://notion.test/frontmatter".into(),
                    title: "Frontmatter Reference".into(),
                },
            ),
        ]);

        let rendered = render_notion_markdown(&artifact, &identities).unwrap();
        assert_eq!(
            rendered,
            "See [Orbexa Quick Start](https://notion.test/orbexa) and [Frontmatter Reference](https://notion.test/frontmatter)."
        );
    }

    #[test]
    fn rendered_hash_changes_with_resolved_content() {
        let artifact = artifact("Body", vec![]);
        assert_ne!(
            rendered_content_hash(&artifact, "Body"),
            rendered_content_hash(&artifact, "Changed")
        );
    }
}
