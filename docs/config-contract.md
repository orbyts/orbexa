# Orbexa Config Contract

Orbexa is configured by a deterministic TOML file stored under the user's XDG config directory.

The config tells Orbexa where it is allowed to create or verify Notion workspace objects, which schema shape to enforce, and where Codexa artifacts will come from.

Orbexa should not scan a Notion workspace broadly and guess what to mutate.

## Default config path

Orbexa should resolve its default config path in this order:

```text
$ORBEXA_CONFIG
$XDG_CONFIG_HOME/orbexa/config.toml
~/.config/orbexa/config.toml
```

For Suhail's setup, `XDG_CONFIG_HOME` points to the `bindu` Git repository:

```text
$XDG_CONFIG_HOME=/Users/suhail/.config
```

So the normal config file should be:

```text
$XDG_CONFIG_HOME/orbexa/config.toml
```

Example:

```bash
mkdir -p "$XDG_CONFIG_HOME/orbexa"
cp config/orbexa.example.toml "$XDG_CONFIG_HOME/orbexa/config.toml"
```

## Required environment

```text
NOTION_TOKEN
```

The token must never be committed, printed, or written to logs.

Do not store `NOTION_TOKEN` in the Orbexa config file. Store it in the shell environment, a local secret manager, or an ignored machine-local environment file.

## Minimal verify config

```toml
schema = "orbexa/config@2"

[notion]
api_version = "2026-03-11"
parent_page_id = "replace-with-shared-parent-page-id"

[notion.bootstrap]
mode = "verify"
root = "parent_page"

[workspace]
page_name = "Codexa"
database_name = "Knowledge"

[workspace.data_sources.documents]
name = "Documents"
kind = "documents"

[artifacts]
input = "../codexa/dist/notion"

[sync]
mode = "export"
managed_by = "orbexa"
on_missing = "mark_stale"
on_drift = "warn_and_skip"
```

## Minimal create config

```toml
schema = "orbexa/config@2"

[notion]
api_version = "2026-03-11"
parent_page_id = "replace-with-shared-parent-page-id"

[notion.bootstrap]
mode = "create"
root = "parent_page"

[workspace]
page_name = "Codexa"
database_name = "Knowledge"

[workspace.data_sources.documents]
name = "Documents"
kind = "documents"

[artifacts]
input = "../codexa/dist/notion"

[sync]
mode = "export"
managed_by = "orbexa"
on_missing = "mark_stale"
on_drift = "warn_and_skip"
```

## Bootstrap modes

### verify

`verify` is the safe default.

In this mode, `orbexa init` should verify that the configured Notion objects exist and match the expected schema.

It should not create pages, databases, data sources, or properties.

### create

`create` is explicit bootstrap mode.

In this mode, `orbexa init` may create the configured Notion page, database, data sources, and properties under the configured root.

Creation must be deterministic.

If a page, database, or data source with the requested name already exists but is not present in Orbexa state and is not explicitly identified by ID in the config, Orbexa should stop with a clear collision error.

Orbexa should not silently adopt existing Notion objects by name.

## Bootstrap behavior

`orbexa init` should:

1. Resolve the config path.
2. Verify `NOTION_TOKEN` is present.
3. Verify access to `notion.parent_page_id`.
4. Find or create a child page named `workspace.page_name`, depending on bootstrap mode.
5. Find or create a database named `workspace.database_name`, depending on bootstrap mode.
6. Find or create the configured data sources, depending on bootstrap mode.
7. Verify or create expected data source properties, depending on bootstrap mode.
8. Write local state under `$XDG_STATE_HOME/orbexa/`.

## State path

Orbexa should keep user state out of the config repo.

Default state path resolution:

```text
$ORBEXA_STATE_DIR
$XDG_STATE_HOME/orbexa
~/.local/state/orbexa
```

Suggested files:

```text
$XDG_STATE_HOME/orbexa/state.toml
$XDG_STATE_HOME/orbexa/notion.lock
```

The config file can live in Git, but the state and lock files should normally remain machine-local unless the user intentionally chooses otherwise.

## Initial data source schema

The first `Documents` data source should include:

```text
Name
Orbexa ID
Codexa ID
Kind
Visibility
Status
Source Repository
Source Path
Source Commit
Content Hash
Last Synced At
Managed By
Sync State
Canonical Route
Published URL
Private URL
```

## Ownership policy

Orbexa owns only pages it created or explicitly adopted.

A page is managed if either:

```text
Managed By = orbexa
```

and:

```text
Codexa ID = <stable document id>
```

or the page appears in Orbexa's local sync lock.

Manual Notion pages are allowed, but Orbexa must not mutate them unless they are explicitly adopted.

## Collision policy

Name collisions are not adoption.

If `workspace.page_name`, `workspace.database_name`, or a configured data source name already exists under the configured parent and Orbexa does not have a recorded ID for it, Orbexa should return a graceful error.

Example:

```text
error: Notion object collision

A page named `Codexa` already exists under the configured parent page, but it is not recorded in Orbexa state.

Orbexa will not adopt it by name.

Options:
  1. provide the existing page ID in config
  2. run an explicit future adoption command
  3. choose a different workspace.page_name
```

## Safety policy

Orbexa should be boring and predictable.

Rules:

1. Never scan all of Notion looking for things to mutate.
2. Never infer ownership from name alone.
3. Never log `NOTION_TOKEN`.
4. Default to verify-only behavior.
5. Require explicit create mode for bootstrap creation.
6. Require explicit adoption for existing Notion objects not found in Orbexa state.
7. Never delete Notion pages by default.
