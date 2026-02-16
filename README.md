# QuerySmith

A Rust-native Data Agent that transforms natural language queries into SQL, inspired by OpenAI's Data Agent. QuerySmith combines RAG, memory systems, and self-correction capabilities to provide production-grade SQL query generation.

## Features

- **Natural Language to SQL**: Convert questions into executable SQL queries
- **RAG-powered Context**: Semantic search over table schemas, documentation, and metadata
- **Self-Correction**: Automatic query debugging with up to 3 retry attempts
- **Memory System**: Persistent memory across conversations with global and user-scoped storage
- **Multi-source Retrieval**: Query tables, documentation, and memories together

## Crates

| Crate | Description |
|-------|-------------|
| `agent-core` | Core agent runtime and tool definitions |
| `metadata-svc` | Schema storage and lineage graph management |
| `rag-engine` | Semantic search and context enrichment |
| `warehouse-conn` | Database connectors (PostgreSQL, SQLite) |

## Quick Start

```bash
# Build the project
cargo build

# Run tests
cargo test
```

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    QuerySmith                        │
├──────────────┬──────────────┬────────────────────────┤
│  agent-core  │ metadata-svc │     rag-engine        │
│  (Runtime)   │  (Storage)   │    (Retrieval)         │
├──────────────┴──────────────┴────────────────────────┤
│                  warehouse-conn                      │
│              (Database Connectors)                   │
└─────────────────────────────────────────────────────┘
```

## Development

QuerySmith follows a 15-phase development roadmap:

- **v0.1.0** (Phase 1-7): Core agent with tools and execution
- **v0.2.0** (Phase 8-12): Web interface, CLI, and integrations
- **v1.0.0** (Phase 13-15): Workflows, documentation, and performance

See [ROADMAP.md](./ROADMAP.md) for detailed development plan.

## License

MIT License - see [LICENSE](./LICENSE)
