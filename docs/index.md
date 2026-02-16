# QuerySmith

A Rust-native Data Agent that transforms natural language queries into SQL.

## Overview

QuerySmith is inspired by OpenAI's Data Agent and combines:
- RAG-powered context enrichment
- Memory system with multiple scopes
- Self-correction capabilities
- Multiple interfaces (CLI, Web, Slack)

## Crates

| Crate | Description |
|-------|-------------|
| `agent-core` | Core agent runtime and tools |
| `metadata-svc` | Schema and lineage storage |
| `rag-engine` | Semantic search and retrieval |
| `warehouse-conn` | Database connectors |
| `memory-svc` | Memory management |
| `workflow-engine` | Workflow automation |

## Quick Start

```bash
# Build
cargo build

# Run CLI
cargo run -p query-smith-cli -- repl

# Run web server
cargo run -p query-smith-web
```

## Configuration

Set environment variables:
- `DATABASE_URL` - Database connection string
- `SLACK_BOT_TOKEN` - Slack bot token
- `RUST_LOG` - Logging level

## Development

```bash
# Run tests
cargo test

# Run clippy
cargo clippy

# Format
cargo fmt
```
