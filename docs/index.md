---
schema: codexa.document@2
id: orbexa.docs.index
title: Orbexa Documentation
description: Entry point for Orbexa setup, configuration, synchronization, and operations documentation.
kind: index
status: active
visibility: public
tags:
  - orbexa
  - notion
  - documentation
navigation:
  root: docs
  product: orbexa
  section: Overview
  order: 0
distribution:
  notion: true
  web: public
  obsidian: true
notion:
  workspace: codexa
web:
  slug: /docs/orbexa
---

# Orbexa Documentation

Orbexa is the Notion endpoint adapter for Codexa artifacts. It owns Notion infrastructure, page identity, link resolution, synchronization state, and repair behavior. It never becomes the source of truth.

Start with [[orbexa.guides.quick-start|Orbexa Quick Start]]. Review [[orbexa.reference.config|Orbexa Configuration]] for root databases and appearance. The synchronization model is described in [[orbexa.concepts.two-pass-sync|Two-Pass Sync]].

For source document authoring, see [[codexa.reference.frontmatter|Codexa Frontmatter Reference]].
