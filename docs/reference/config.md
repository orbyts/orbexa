---
schema: codexa.document@2
id: orbexa.reference.config
title: Orbexa Configuration
description: Reference for the root-oriented Orbexa configuration schema and per-root Notion appearance.
kind: reference
status: active
visibility: public
tags:
  - orbexa
  - configuration
  - notion
navigation:
  root: docs
  product: orbexa
  section: Reference
  order: 200
distribution:
  notion: true
  web: public
  obsidian: true
notion:
  workspace: codexa
web:
  slug: /docs/orbexa/reference/config
---

# Orbexa Configuration

Orbexa uses `orbexa/config@2`. The config describes the Notion workspace and root databases that render Codexa's conceptual tree.

## Complete example

```toml
schema = "orbexa/config@2"

[notion]
api_version = "2026-03-11"
parent_page_id = "<parent-page-id>"

[notion.bootstrap]
mode = "create"
root = "parent_page"

[workspace]
page_name = "Codexa"

[workspace.appearance.icon]
type = "emoji"
emoji = "🧭"

[workspace.appearance.cover]
type = "external"
url = "https://example.com/workspace-cover.jpg"

[workspace.roots.docs]
database_name = "Docs"
data_source_name = "Documents"

[workspace.roots.docs.appearance.icon]
type = "emoji"
emoji = "📘"

[workspace.roots.docs.appearance.cover]
type = "external"
url = "https://example.com/docs-cover.jpg"

[workspace.roots.knowledge]
database_name = "Knowledge"
data_source_name = "Documents"

[workspace.roots.knowledge.appearance.icon]
type = "emoji"
emoji = "📚"

[workspace.roots.knowledge.appearance.cover]
type = "external"
url = "https://example.com/knowledge-cover.jpg"

[artifacts]
input = "../codexa/dist/notion"

[sync]
on_missing = "recreate"
on_drift = "update"
```

## Root mapping

The root keys must match `navigation.root` in Codexa source documents:

```text
docs       → Docs database
knowledge  → Knowledge database
```

New roots such as `code-reference` or `media` can be added later without changing the source identity model.

## Appearance inheritance

Each root defines the default icon and cover for pages created in that database. Database covers remain subject to Notion's database limitations; page appearance is applied to managed document pages.

## Registry and lock state

Orbexa stores resolved Notion identities under its config directory. These files are endpoint state, not content sources. Source metadata remains governed by [[codexa.reference.frontmatter|Codexa Frontmatter Reference]].
