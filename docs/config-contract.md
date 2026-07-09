# Orbexa Config Contract

Orbexa is configured by a deterministic TOML file.

The config tells Orbexa where it is allowed to create or verify Notion workspace objects, which schema shape to enforce, and where Codexa artifacts will come from.

Orbexa should not scan a Notion workspace broadly and guess what to mutate.

## Required environment

```text
NOTION_TOKEN
```

The token must never be committed, printed, or written to logs.

## Bootstrap model

Orbexa supports two bootstrap modes:

```text
verify
create
```

`verify` is the default. It checks that the configured Notion objects already exist and match the expected schema.

`create` allows Orbexa to create the managed workspace structure under a configured root.

Orbexa should support a root page first. Account-level or workspace-level creation can be added later if Notion exposes a stable API path for it.

## Minimal config

```toml
schema = "orbexa/config@1"

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

## Create-mode config

```toml
schema = "orbexa/config@1"

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

## Bootstrap behavior

`orbexa init` should:

1. Verify `NOTION_TOKEN` is present.
2. Verify access to `notion.parent_page_id`.
3. Resolve or create a child page named `workspace.page_name`.
4. Resolve or create a database named `workspace.database_name` under that page.
5. Resolve or create the configured data sources.
6. Verify or update expected data source properties.
7. Write local state under `.orbexa/`.

## Name collision policy

Orbexa should be deterministic and conservative.

If `notion.bootstrap.mode = "create"` and a page, database, or data source with the requested name already exists, Orbexa must not silently adopt it.

Default behavior:

```text
name exists + no Orbexa state + no explicit ID = error
```

The error should explain what happened and how to fix it.

Allowed fixes:

1. Rename the requested object in the config.
2. Provide the existing Notion object ID explicitly.
3. Run a future explicit adoption command.

Example future adoption command:

```bash
orbexa adopt database --id <notion-database-id> --as knowledge
```

## Local state

Orbexa should record created or adopted Notion IDs locally.

Suggested layout:

```text
.orbexa/
├── state.toml
└── notion.lock
```

Example state:

```toml
schema = "orbexa/state@1"

[workspace]
page_id = "..."
database_id = "..."

[workspace.data_sources.documents]
id = "..."
name = "Documents"
```

Future runs should prefer IDs from local state over name lookup.

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

## Safety policy

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

## Recommended first Notion shape

```text
Existing user-selected Notion parent page
└── Codexa
    └── Knowledge database
        └── Documents data source
            ├── kind = playbook
            ├── kind = flow
            ├── kind = tech_doc
            ├── kind = reference
            └── kind = project_note
```

Start with one `Documents` data source and use `Kind` for classification.

Do not create separate `Flows` and `Playbooks` data sources until the schema actually needs to diverge.
