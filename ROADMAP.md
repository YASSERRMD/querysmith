# QuerySmith - Complete Development Roadmap
Project: QuerySmith - Rust-native Data Agent (OpenAI Data Agent clone)
Git Owner: yasserrmd arafath.yasser@gmail.com
Target: Production-grade SQL agent with RAG, memory, self-correction

## DEVELOPMENT PHASES

### phase-1-project-skeleton
```
feat/phase1: initialize Cargo.toml + workspace + gitignore
feat/phase1: add project README + LICENSE (MIT)
feat/phase1: create crate structure (agent-core, metadata-svc, etc.)
Deliverables:
- Cargo.toml workspace
- Crate layout: agent-core, metadata-svc, rag-engine, warehouse-conn
- .gitignore, README.md
```

### phase-2-warehouse-connector
```
feat/phase2: implement Warehouse trait + Postgres connector  
feat/phase2: add sqlx + async traits
feat/phase2: basic query execution + schema fetch
Deliverables:
- warehouse-conn crate with Warehouse trait
- Postgres impl (query, get_schema, preview_table)
- Unit tests
```

### phase-3-metadata-service
```
feat/phase3: implement MetadataService + schema storage  
feat/phase3: lineage graph ingestion (simple JSON)
feat/phase3: table annotations API
Deliverables:
- metadata-svc crate (SQLite/Postgres backend)
- GraphQL or REST API for table metadata
- Schema + lineage storage
```

### phase-4-context-enricher
```
feat/phase4: build offline context pipeline
feat/phase4: table context generation (schema+lineage+annotations)
feat/phase4: BARQ integration for vector embeddings
Deliverables:
- context-enricher binary
- Context blob generation
- Embeddings pipeline → BARQ index
```

### phase-5-rag-service
```
feat/phase5: RAG retrieval service
feat/phase5: semantic search over table contexts
feat/phase5: multi-source retrieval (tables+docs+memories)
Deliverables:
- rag-svc with /retrieve endpoint
- KNN search via BARQ
- Context chunking + ranking
```

### phase-6-tool-definitions
```
feat/phase6: define Tool trait + JSON schemas
feat/phase6: implement SearchTablesTool, RunSqlTool, DebugQueryTool
feat/phase6: tool registry
Deliverables:
- agent-core::tools module
- Serde JSON schemas for LLM tool-calling
- Mock tool execution
```

### phase-7-agent-runtime
```
feat/phase7: agent orchestrator loop
feat/phase7: integrate Munimax M2.5 system prompt
feat/phase7: tool execution + self-correction (max 3 retries)
Deliverables:
- agent-runtime crate
- Conversation state management
- LLM client wrapper (OpenRouter/Ollama)
```

### phase-8-memory-service
```
feat/phase8: MemoryService with scopes (global/user)
feat/phase8: memory injection into agent prompts
feat/phase8: correction extraction from conversations
Deliverables:
- memory-svc crate
- Memory retrieval by table/question similarity
- Save/extract logic
```

### phase-9-evaluation-harness
```
feat/phase9: Evals framework (golden SQL dataset)
feat/phase9: SQL result comparison + LLM grading
feat/phase9: regression detection
Deliverables:
- evals binary
- Test dataset format
- Score reporting
```

### phase-10-web-interface
```
feat/phase10: Axum HTTP + WebSocket gateway
feat/phase10: chat streaming endpoint
feat/phase10: auth middleware
Deliverables:
- query-smith-web crate
- /chat streaming API
- Basic frontend (HTMX or Svelte)
```

### phase-11-slack-bot
```
feat/phase11: Slack Events API integration
feat/phase11: slash command + thread conversations
feat/phase11: memory persistence per Slack user
Deliverables:
- slack-bot binary
- Thread ↔ conversation ID mapping
```

### phase-12-cli-tool
```
feat/phase12: CLI binary (clap)
feat/phase12: interactive REPL mode
feat/phase12: script mode for workflows
Deliverables:
- query-smith CLI
- Shell completions
```

### phase-13-workflows
```
feat/phase13: workflow YAML definitions
feat/phase13: workflow engine
feat/phase13: scheduled execution
Deliverables:
- Workflow DSL
- workflow-engine crate
```

### phase-14-docs-ci-cd
```
feat/phase14: documentation site (mdbook)
feat/phase14: GitHub Actions CI/CD
feat/phase14: Docker images + deployment manifests
Deliverables:
- Full docs
- CI pipeline
- Docker Compose for local dev
```

### phase-15-performance
```
feat/phase15: connection pooling optimizations
feat/phase15: caching layer (memories/context)
feat/phase15: async streaming improvements
Deliverables:
- Perf benchmarks
- Production optimizations
```

## RELEASE TAGS
- v0.1.0-mvp          # phase-1 to phase-7
- v0.2.0-interfaces   # phase-8 to phase-12  
- v1.0.0-stable       # phase-13 to phase-15

## DEPLOYMENT TARGETS
- Local dev: docker compose up
- Staging: Fly.io / Render
- Prod: Kubernetes / your infra
