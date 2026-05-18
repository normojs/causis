# Causis

[中文 README](README-zh.md)

Causis is a Rust-first causal knowledge engine for explainable and auditable enterprise decisions.

It is designed as **Causal as a Service** infrastructure: a foundation that developers, ISVs, and system integrators can embed to build decision applications with formal causal reasoning, multi-source data governance, and full evidence provenance.

> Every decision should have a trusted core.

## Status

Causis has moved from architecture-only planning into the first implementation milestone. The repository now contains planning documents, a minimal Rust workspace, `causis-core` domain types, a `causis-cli` demo runner, and a deterministic leave-approval fixture.

## What Causis Is

Causis is not only a database, a GraphRAG tool, or a vertical business application. It aims to combine:

- **Causal reasoning**: causal tracing, intervention analysis, counterfactual reasoning, and root-cause queries.
- **Evidence provenance**: every derived fact and decision can be traced back to original source files, locations, versions, and processing steps.
- **Conflict resolution**: multi-source contradictions are detected, preserved, resolved, and audited.
- **Knowledge graph infrastructure**: temporal property graphs, rule modeling, graph traversal, and graph-vector hybrid retrieval.
- **Private deployment**: a lightweight Rust-oriented architecture for embedded or private environments.

## Architecture

Causis is organized into eight major modules:

```text
API Layer
  REST / GraphQL / SDK / CLI

Trusted Explanation and Audit
  Evidence chains, reports, graph visualization, audit logs

Causal Inference and Query Engine
  Causal tracing, intervention, counterfactuals, root cause

Conflict Resolution
  Rule-based, quality-weighted, LLM-assisted, human-in-the-loop

Entity Disambiguation and Relation Completion
  Fuzzy entity matching, semantic matching, link prediction

Knowledge Graph
  Temporal property graph, Cypher/SQL, graph-vector retrieval

Fact Zone
  Cleaning, normalization, quality scoring, conflict marking

Evidence Lake
  Immutable raw evidence, metadata, provenance, version snapshots

Multi-source Ingest
  Documents, databases, object storage, streams
```

## Module Overview

| Module | Responsibility | Current design direction |
| --- | --- | --- |
| Multi-source ingest | Parse documents and connect structured data sources | Ingestor-Core, DocTor, Arrow streams |
| Evidence lake | Store immutable raw evidence and metadata | RustFS, Apache Iceberg, provenance tables |
| Fact zone | Normalize facts and mark conflicts | Polars, DuckDB/DataFusion, quality scoring |
| Knowledge graph | Build temporal graph knowledge | LadybugDB, Cypher/SQL, vector search |
| Entity disambiguation | Resolve ambiguous entities and complete relations | Rules, embeddings, ONNX/Candle, review APIs |
| Conflict resolution | Decide trusted fact versions transparently | Four-layer resolution workflow |
| Causal inference | Run formal causal queries | DeepCausality, why-rs, CIfly |
| Explanation and audit | Produce evidence-backed explanations | Evidence chain builder, reports, audit logs |

## MVP Direction

The recommended first vertical slice is a traceable leave-approval decision workflow:

1. Ingest policy documents and employee/approval data.
2. Store original evidence with stable source anchors.
3. Normalize facts and mark conflicts.
4. Build a small temporal knowledge graph.
5. Resolve approval-chain conflicts when needed.
6. Run a causal trace query.
7. Return an explanation with evidence links and audit records.

This keeps the first milestone focused on the core value proposition: a decision that is computable, explainable, and traceable.

## Repository Map

```text
crates/
  causis-core                    Core IDs, facts, provenance, demo pipeline
  causis-cli                     Local CLI demo runner

fixtures/
  leave-approval                 Deterministic P0 demo data

docs/
  000...008                    Module design documents
  Causis ... v1.0.md           Architecture and development plan
  商业模式.md                    Business model
  竞品分析.md                    Competitor analysis
  项目推进方案.md                Project execution plan
  子项目/Runcible容器.md         Lightweight container subproject idea

README.md                      English overview
README-zh.md                   Chinese overview
```

## Key Documents

- [Project master plan](docs/Causis%20项目总体规划.md)
- [Development architecture and Cube evaluation](docs/开发架构优化与%20Cube%20集成评估.md)
- [Development order and feature implementation plan](docs/开发顺序与功能实现规划.md)
- [Architecture and feature final review](docs/架构与功能终审报告.md)
- [Architecture and development plan](docs/Causis%20架构与开发总体规划书%20v1.0.md)
- [Multi-source ingest](docs/001、多源数据接入与适配.md)
- [Evidence lake](docs/002.证据湖（原始层）设计说明书%20.md)
- [Fact zone](docs/003、事实层（Fact%20Zone）.md)
- [Knowledge graph](docs/004、知识图谱（推理层）.md)
- [Entity disambiguation](docs/005、实体对齐与消歧.md)
- [Conflict resolution](docs/006、冲突检测与协调.md)
- [Causal inference](docs/007、因果推理与查询引擎.md)
- [Trusted explanation and audit](docs/008、可信解释与审计.md)

## Roadmap

The current planning documents describe three broad stages:

1. **MVP validation**: implement one end-to-end decision scenario and prove evidence-backed causal tracing.
2. **Platformization**: generalize APIs, SDKs, low-code tooling, and human review flows.
3. **Ecosystem**: build a causal model marketplace, partner integrations, and industry templates.

## Development Quick Start

The first implementation milestone is a local, deterministic leave-approval demo. It does not require RustFS, Iceberg, LadybugDB, Cube, or any external service.

```bash
cargo test --workspace
cargo run -p causis-cli -- demo leave-approval
```

The demo loads `fixtures/leave-approval`, detects a manager conflict, resolves it with an authority-first rule, traces the approval path, prints an evidence-backed explanation JSON, and writes it to `target/causis/leave-approval/explanation.json`.

## License

Causis is licensed under either [Apache License 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.
