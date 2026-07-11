---
schema: codexa.document@2
id: orbexa.concepts.two-pass-sync
title: Orbexa Two-Pass Sync
description: How Orbexa establishes page identities, resolves logical links, and updates Notion without duplicates.
kind: concept
status: active
visibility: public
tags:
  - orbexa
  - synchronization
  - links
  - notion
navigation:
  root: docs
  product: orbexa
  section: Concepts
  order: 300
distribution:
  notion: true
  web: public
  obsidian: true
notion:
  workspace: codexa
web:
  slug: /docs/orbexa/concepts/two-pass-sync
---

# Orbexa Two-Pass Sync

Logical document links cannot be rendered reliably until every target page has a stable Notion identity. Orbexa therefore applies a Codexa bundle in two passes.

## Pass 1: establish identity

Orbexa loads every artifact, validates its configured root, and checks the lock for an existing page.

For each document it either:

- reuses the valid locked page;
- recreates a missing or trashed page according to policy; or
- creates a new placeholder page.

The result is a complete mapping from Codexa document ID to Notion page ID and URL.

## Pass 2: render and synchronize

Orbexa resolves logical links such as:

```md
[[codexa.reference.frontmatter|Codexa Frontmatter Reference]]
```

into ordinary Markdown links targeting the known Notion page URL. It then computes the rendered-content hash and compares it with the lock.

- unchanged rendered content is skipped;
- changed content updates the existing page;
- newly created pages receive final properties and content;
- no duplicate page is created merely because source content changed.

## Why dependent pages may update

If a linked page is recreated, its Notion URL changes. A source document that did not change may still need an update because its rendered link destination changed. Tracking the rendered-content hash captures this endpoint-level dependency.

Codexa's endpoint-neutral model is described in [[codexa.concepts.navigation-and-links|Navigation and Links]].

## Identity recovery

A local lock is an optimization, not the only source of page identity. When a lock entry is unavailable or stale, Orbexa queries the destination data source for the exact Codexa `Document ID`. This lets a new machine or a repaired configuration adopt the existing Notion page without creating a duplicate.

Orbexa refuses to choose between multiple live pages with the same `Document ID`. Resolve the duplicate in Notion and run apply again.

## Sort order

Orbexa writes `navigation.order` to the Notion `Sort Order` number property on both create and update. Configure database views to sort by `Sort Order` ascending, then `Name` ascending. Values such as `100`, `200`, and `300` leave room for later insertion.
