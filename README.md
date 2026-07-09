cat > README.md <<'EOF'

# Orbexa

Orbexa applies Codexa-generated artifacts to downstream systems such as Notion.

Current status: early experimental CLI.

Orbexa is designed around a Git-first content workflow:

1. Git repositories remain the source of truth.

2. Codexa validates Markdown/frontmatter and emits versioned artifacts.

3. Orbexa consumes those artifacts and applies them to managed Notion pages, databases, and data sources.

## Current commands

```bash

orbexa check

orbexa init --dry-run

orbexa init

orbexa init --recreate-database --dry-run

orbexa init --recreate-database

orbexa apply <ARTIFACT_DIR> --dry-run

orbexa apply <ARTIFACT_DIR>

Environment

Orbexa expects a Notion API token in:
NOTION_API_KEY

NOTION_TOKEN is accepted as a fallback.
EOF
fi

if [ ! -f LICENSE ]; then
cat > LICENSE <<‘EOF’
MIT License

Copyright (c) 2026 Suhail

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the Software), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in
all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED AS IS, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
THE SOFTWARE.
EOF
fi

cargo fmt
cargo test
cargo package –list
cargo publish –dry-run

