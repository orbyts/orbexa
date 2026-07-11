---
schema: codexa.document@2
id: orbexa.guides.quick-start
title: Orbexa Quick Start
description: Configure Notion roots, initialize managed databases, and apply Codexa artifacts idempotently.
kind: guide
status: active
visibility: public
tags:
  - orbexa
  - quick-start
  - notion
navigation:
  root: docs
  product: orbexa
  section: Guides
  order: 100
distribution:
  notion: true
  web: public
  obsidian: true
notion:
  workspace: codexa
web:
  slug: /docs/orbexa/guides/quick-start
---

# Orbexa Quick Start

Orbexa consumes Codexa Notion artifacts. Git repositories remain authoritative; Orbexa records only endpoint state such as Notion database IDs, page IDs, and rendered hashes.

Before continuing, build artifacts using [[codexa.guides.quick-start|Codexa Quick Start]].

## 1. Configure Notion access

Set a Notion integration token:

```bash
export NOTION_API_KEY="..."
```

The integration needs permission to read, insert, and update content beneath the configured parent page.

## 2. Configure roots

Orbexa maps conceptual Codexa roots to Notion databases. A minimal `orbexa/config@2` setup includes `docs` and `knowledge` roots:

```toml
[workspace.roots.docs]
database_name = "Docs"
data_source_name = "Documents"

[workspace.roots.knowledge]
database_name = "Knowledge"
data_source_name = "Documents"
```

See [[orbexa.reference.config|Orbexa Configuration]] for the complete file.

## 3. Check configuration

```bash
orbexa check
```

## 4. Initialize infrastructure

```bash
orbexa init --dry-run
orbexa init
```

`init` creates only missing configured roots. Running it again verifies existing registered infrastructure without creating duplicates.

## 5. Apply Codexa artifacts

```bash
orbexa apply "$MATRIX/crates/codexa/dist/notion" --dry-run
orbexa apply "$MATRIX/crates/codexa/dist/notion"
```

Artifacts with `root: docs` route to the Docs database. Artifacts with `root: knowledge` route to Knowledge.

## 6. Verify idempotency

Run the apply command again:

```bash
orbexa apply "$MATRIX/crates/codexa/dist/notion" --dry-run
```

Unchanged pages should be skipped. Changed source content updates the same Notion page rather than creating a duplicate.

Orbexa resolves cross-document links using the two-pass process described in [[orbexa.concepts.two-pass-sync|Two-Pass Sync]].
