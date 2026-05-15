# 开发架构优化与 Cube 集成评估

本文档回答三个问题：

1. 当前 Causis 已有开发架构是否需要优化？
2. 当前技术选型是否需要调整？
3. Cube.js/Cube 是否可以在 Causis 中使用，应该放在哪一层？

结论先行：

- 当前 Causis 的总体方向是对的：因果推理、证据溯源、冲突协调、可信审计这四个核心支柱应保持不变。
- 需要优化的是“开发架构”，也就是如何从宏大架构落到可实现、可测试、可替换的工程结构。
- Cube 可以使用，但不应成为 Causis Core 的推理核心。它适合成为 P1/P2 阶段的“分析语义层”，服务 Causis Studio、BI、指标分析、运营看板和 AI Agent 的指标上下文。

## 1. 当前架构评价

### 1.1 已经正确的部分

当前项目已有几个非常好的判断：

- 定位不是普通 RAG，而是可信决策基础设施。
- 采用八层模块划分，边界基本清楚。
- 将事实层和冲突协调层分开，这是很重要的架构成熟度。
- 把证据链和审计作为底层能力，而不是最后补一个日志系统。
- Rust-first、单节点优先、可嵌入部署的方向适合早期产品验证。
- 将 LLM 限制在抽取、语义辅助、解释润色，而不是让 LLM 做最终形式化推理。

这些方向不建议推翻。

### 1.2 需要优化的部分

当前更需要优化的是工程落地方式：

| 问题 | 风险 | 优化方向 |
| --- | --- | --- |
| 模块很多，容易同时开工 | 长期停留在文档和框架 | 用 MVP 纵向切片约束范围 |
| 技术栈很先进，但集成成本高 | 一开始就被基础设施拖住 | 先 trait + in-memory/file adapter，再接真实组件 |
| RustFS/Iceberg/LadybugDB/DeepCausality 都是重依赖 | P0 阶段验证慢 | P0 只稳定领域模型和接口 |
| 文档里既有产品架构，也有实现细节 | 读者难判断优先级 | 增加 ADR 和分阶段技术矩阵 |
| 缺少应用开发层设计 | Core 好但难演示价值 | 增加 Studio/API/SDK/MCP/BI 的应用层规划 |
| 缺少分析语义层 | 指标、看板、BI、运营分析难统一 | 引入 Cube 作为可选分析语义层 |

## 2. 推荐的开发架构

### 2.1 采用六边形架构

Causis Core 应采用六边形架构，也可以理解为 ports and adapters：

```text
                  ┌────────────────────┐
                  │      Causis API     │
                  │ REST / SDK / MCP    │
                  └─────────┬──────────┘
                            │
┌───────────────────────────▼───────────────────────────┐
│                     Causis Core                         │
│  Domain types / provenance / facts / graph / causal     │
│  resolution / explanation / audit                       │
└──────────────┬─────────────┬─────────────┬─────────────┘
               │             │             │
       ┌───────▼──────┐ ┌────▼─────┐ ┌────▼────────┐
       │ Storage       │ │ Graph    │ │ Model/LLM   │
       │ adapters      │ │ adapters │ │ adapters    │
       └───────────────┘ └──────────┘ └─────────────┘
```

核心原则：

- Core 不直接依赖 RustFS、Iceberg、LadybugDB、Cube、LLM provider。
- Core 只依赖 trait 和领域模型。
- 具体基础设施放在 adapter crate。
- MVP 可以用 in-memory 或 local file adapter 跑通。
- 后续替换真实组件时不伤害领域模型。

### 2.2 推荐 Rust Workspace

建议按“稳定内核 + 可替换适配器 + 应用入口”拆分。

```text
crates/
  causis-core
  causis-ingest
  causis-evidence
  causis-facts
  causis-graph
  causis-resolution
  causis-causal
  causis-explain
  causis-api
  causis-cli

adapters/
  causis-adapter-local
  causis-adapter-duckdb
  causis-adapter-iceberg
  causis-adapter-rustfs
  causis-adapter-ladybug
  causis-adapter-cube
  causis-adapter-llm

apps/
  causis-server
  causis-studio
  causis-mcp-server
```

P0 阶段可以不完整创建全部目录，但设计上应遵循这个边界。

### 2.3 分阶段实现策略

#### P0：不依赖重型基础设施

先做：

- `causis-core`
- `causis-evidence`
- `causis-facts`
- `causis-graph` 的 in-memory 实现
- `causis-causal` 的 trace 模型
- `causis-explain`
- `causis-cli`

暂缓：

- 真 Iceberg 写入。
- 真 LadybugDB 集成。
- 真 RustFS 部署。
- Cube 集成。
- 前端 Studio。
- LLM 辅助裁决。

P0 成功标准：

- 一条命令跑通 demo。
- 每条结论有证据链。
- 冲突不丢失。
- 输出解释 JSON。

#### P1：接入真实基础设施

加入：

- RustFS/S3-compatible storage。
- Iceberg 或 DuckDB-backed fact store。
- LadybugDB adapter。
- Postgres connector。
- Axum API。
- 基础 Studio。
- Cube semantic layer adapter。

#### P2：平台化和生态

加入：

- MCP Server。
- SDK。
- Cube-powered analytics。
- Marketplace。
- 企业权限。
- 多租户。
- 审计报告生成。

## 3. 技术选型优化建议

### 3.1 保留的核心技术

| 技术 | 建议 | 理由 |
| --- | --- | --- |
| Rust | 保留 | 核心能力需要可嵌入、稳定、安全、低资源 |
| Axum | 保留 | Rust API 服务生态成熟，适合 Causis Server |
| Arrow | 保留 | 适合作为模块间列式数据交换格式 |
| Polars/DataFusion/DuckDB | 保留 | 各自覆盖 DataFrame、查询引擎、嵌入式分析 |
| Apache Iceberg | 保留但 P1 接入 | 适合事实表、快照、时间旅行 |
| RustFS/S3-compatible storage | 保留但 P1 接入 | 适合证据湖原始文件层 |
| LadybugDB | 保留但 adapter 化 | 适合图谱，但不要让 Core 强依赖 |
| DeepCausality/why-rs/CIfly | 保留为因果技术方向 | P0 先抽象 CausalEngine，逐步集成 |
| Candle/ONNX | 保留为 P1/P2 增强 | 用于本地模型、嵌入和轻量判别 |

### 3.2 需要新增的技术角色

| 新增角色 | 推荐技术 | 放置位置 |
| --- | --- | --- |
| 语义层/指标层 | Cube Core | `causis-adapter-cube`，P1/P2 |
| API Schema | OpenAPI/utoipa | `causis-api` |
| 配置管理 | TOML/YAML + serde | `causis-core` |
| 数据契约 | JSON Schema / Arrow schema | `causis-core` / `causis-facts` |
| ADR 管理 | Markdown ADR | `docs/adr/` |
| Demo fixtures | JSON/CSV/Markdown/PDF fixtures | `examples/` 或 `fixtures/` |
| MCP 工具层 | Causis MCP Server | `apps/causis-mcp-server` |

### 3.3 不建议过早引入的技术

- Kubernetes：P0 不需要。
- Kafka/Pulsar：P0 不需要，CDC 可以先批处理或 Postgres connector。
- 大型向量数据库：P0 不需要，先用嵌入字段和简单检索。
- GPU 推理：P0 不需要。
- 完整分布式图数据库：P0 不需要。
- 多租户权限系统：P0 不需要，但数据模型要预留。

## 4. Cube 是什么

Cube Core 是一个开源语义层项目，官方定位是用于 AI、BI 和嵌入式分析的 semantic layer。它是 headless 的，提供 REST、GraphQL 和 SQL API。官方 README 明确说 Cube 可用于构建 embedded analytics、BI 工具，也可以为 AI agents 提供数据上下文。

Cube 的关键能力：

- 定义统一的 metrics、dimensions、segments 和 joins。
- 通过 REST、GraphQL、SQL API 对外服务。
- 支持 BI 工具和嵌入式分析。
- 支持多种 SQL 数据源。
- 支持 pre-aggregations，以提升分析查询性能。
- 可通过 SQL API 与 BI 工具集成。

官方资料也说明，Cube 面向的是 SQL 数据源；它不适合直接访问 REST/GraphQL API 或任意文件。如果要查询文件，应通过 DuckDB、Athena、Trino 等 SQL 层接入 Parquet、CSV、JSON 或对象存储数据。

## 5. Cube 在 Causis 中能做什么

### 5.1 最适合的位置：分析语义层

Cube 不应该放在 Causis 的因果推理核心里，而应放在事实层和应用层之间：

```text
Evidence Lake
  -> Fact Zone
  -> Knowledge Graph / Causal Engine
  -> Explanation and Audit

Fact Zone / Audit Tables
  -> Cube Semantic Layer
  -> Studio / BI / Metrics API / AI Agent Context
```

也就是说：

- Causis Core 负责事实、证据、冲突、因果、审计。
- Cube 负责把事实和审计结果变成可查询的业务指标语义层。

### 5.2 可以使用的场景

#### 场景一：Causis Studio 的指标层

Cube 可以为 Studio 提供：

- 数据源数量。
- 证据入湖量。
- 事实数量。
- 冲突组数量。
- 自动裁决率。
- 人工审核积压。
- 推理查询次数。
- 审计报告数量。
- 平均证据链长度。
- 高风险决策占比。

这些是运营和治理指标，不是因果推理核心。

#### 场景二：企业 BI 集成

企业可能希望用 Superset、Tableau、Power BI、Metabase 等工具查看 Causis 运行状态。Cube 的 SQL API 和 semantic layer 能让 Causis 暴露稳定指标，而不是让 BI 工具直接扫底层事实表。

#### 场景三：AI Agent 的指标上下文

Agent 可以问：

- 最近哪个部门的冲突率最高？
- 哪类制度文件最常引发冲突？
- 本周人工裁决的高风险事项有哪些？
- 哪些数据源质量分数下降？

这些问题更像 analytics query，而不是 causal query。Cube 很适合承接。

#### 场景四：多租户指标隔离

Cube 支持在语义层做安全控制和上下文过滤。Causis 企业版将来需要多租户、部门级视图、客户隔离时，Cube 可以作为分析查询的安全边界之一。

#### 场景五：预聚合加速

Causis 的审计和治理指标可能会被频繁查询。例如：

- 每日入湖证据量。
- 每个业务域的冲突趋势。
- 每种裁决策略的命中率。
- 每个数据源的质量趋势。

Cube 的 pre-aggregations 可以把这些指标预计算，减少对底层事实表和审计表的压力。

## 6. Cube 不应该做什么

Cube 不适合承担以下职责：

| 不建议职责 | 原因 |
| --- | --- |
| 原始证据湖 | Cube 不负责不可变文件存储和证据版本 |
| Provenance 内核 | Cube 不负责源文件段落级溯源 |
| 冲突裁决 | Cube 可以统计冲突，不应裁决冲突 |
| 知识图谱存储 | Cube 是语义指标层，不是因果图数据库 |
| 因果推理 | Cube 能做分析查询，不做 do-operator 或反事实 |
| 审计真相源 | Cube 可以读审计表，不应成为审计主存储 |
| P0 必需组件 | MVP 不应依赖 Cube 才能跑通 |

## 7. 推荐集成方式

### 7.1 Causis 与 Cube 的关系

推荐定义：

> Cube is the optional analytics semantic layer for Causis, not the causal core.

中文可以定义为：

> Cube 是 Causis 的可选分析语义层，不是因果推理内核。

### 7.2 数据路径

推荐数据路径：

```text
Causis facts / audit tables
  -> DuckDB / Postgres / Trino / Athena / ClickHouse
  -> Cube data model
  -> REST / GraphQL / SQL API
  -> Studio / BI / Agent
```

P1 最简单方案：

- Causis 把 demo facts 和 audit logs 写入 DuckDB 或 Postgres。
- Cube 连接 DuckDB/Postgres。
- Cube 定义 Causis governance metrics。
- Studio 或 BI 读取 Cube API。

P2 云原生方案：

- Causis 事实层使用 Iceberg。
- 通过 Trino/Athena/Databricks/DuckDB 暴露 SQL。
- Cube 连接 SQL 查询层。
- Cube 负责语义指标和预聚合。

### 7.3 Cube 数据模型建议

可先定义这些 cubes：

| Cube | 来源 | 指标 |
| --- | --- | --- |
| `evidence_files` | 证据元数据表 | 文件数、大小、来源、入湖时间 |
| `facts` | 事实表 | 事实数、实体数、质量分数 |
| `conflicts` | 冲突组表 | 冲突数、类型、状态 |
| `resolutions` | 裁决日志 | 自动裁决率、人工裁决率、策略命中率 |
| `causal_traces` | 推理审计表 | 查询量、路径长度、置信度 |
| `audit_events` | 审计日志 | 操作次数、用户、事件类型 |

### 7.4 Adapter 设计

建议新增 adapter：

```text
adapters/causis-adapter-cube
  - generate_cube_models()
  - sync_semantic_layer()
  - export_governance_metrics()
  - validate_cube_connection()
```

但 P1 之前不需要实现。P0 只需要在文档和架构里预留位置。

## 8. 是否需要优化现有技术选型

需要，但不是推翻，而是分层和降风险。

### 8.1 当前选型优化后的优先级

| 优先级 | 技术 | 动作 |
| --- | --- | --- |
| P0 | Rust、serde、thiserror、tracing、clap | 立即使用 |
| P0 | in-memory/local adapters | 先跑通核心链路 |
| P0 | JSON fixtures | 先稳定领域模型 |
| P0 | Axum | 可在 CLI demo 后接入 |
| P1 | DuckDB/Postgres | 作为真实查询和 Cube 接入桥 |
| P1 | RustFS/Iceberg | 接入证据湖和事实层 |
| P1 | LadybugDB | 接入图谱层 |
| P1 | Cube | 接入分析语义层 |
| P1/P2 | DeepCausality/why-rs/CIfly | 增强因果能力 |
| P2 | WASM plugins | 连接器和模板生态 |
| P2 | Kubernetes/HA | 企业规模化部署 |

### 8.2 新增架构图

```text
                    ┌────────────────────────┐
                    │        Causis Studio    │
                    │  Graph / Audit / BI UI  │
                    └───────────┬────────────┘
                                │
       ┌────────────────────────┼────────────────────────┐
       │                        │                        │
┌──────▼──────┐          ┌──────▼──────┐          ┌──────▼──────┐
│ Causis API  │          │ Cube Layer  │          │ MCP Server  │
│ causal/audit│          │ metrics/BI  │          │ agent tools │
└──────┬──────┘          └──────┬──────┘          └──────┬──────┘
       │                        │                        │
┌──────▼────────────────────────▼────────────────────────▼──────┐
│                         Causis Core                            │
│ Evidence / Facts / Conflicts / Graph / Causal / Explanation    │
└──────┬─────────────┬───────────────┬───────────────┬──────────┘
       │             │               │               │
┌──────▼──────┐ ┌────▼─────┐ ┌───────▼───────┐ ┌────▼──────────┐
│ RustFS/S3   │ │ Iceberg  │ │ LadybugDB     │ │ LLM/ONNX      │
│ evidence    │ │ facts    │ │ graph/audit   │ │ optional AI   │
└─────────────┘ └──────────┘ └───────────────┘ └───────────────┘
```

## 9. 决策建议

### 9.1 对当前开发架构

建议优化，不推翻。

应补充：

- 六边形架构说明。
- workspace 拆分策略。
- adapter-first 原则。
- P0/P1/P2 技术矩阵。
- ADR 机制。
- fixture-driven demo。

### 9.2 对当前技术选型

建议保留主线，但调整接入顺序：

1. 先实现领域模型和本地 adapter。
2. 再接 DuckDB/Postgres 作为过渡 SQL 层。
3. 再接 RustFS/Iceberg/LadybugDB。
4. 最后接 DeepCausality、why-rs、Cube、Studio、MCP。

### 9.3 对 Cube

建议纳入规划，但列为 P1/P2。

Cube 的定位：

- 不是 Causis 的核心存储。
- 不是 Causis 的因果引擎。
- 不是 Causis 的审计真相源。
- 是 Causis 的分析语义层、BI 接入层、指标 API 层、Agent analytics context 层。

一句话：

> Causis 负责可信决策，Cube 负责可信决策系统的指标语义和分析消费。

## 10. 下一步行动

建议按以下顺序执行：

1. 新增 `docs/adr/0001-core-architecture.md`，确定六边形架构。
2. 新增 `docs/adr/0002-storage-and-adapter-strategy.md`，确定 P0 先本地 adapter。
3. 新增 `docs/adr/0003-cube-semantic-layer.md`，确定 Cube 是可选分析语义层。
4. 初始化 Rust workspace。
5. 实现 `causis-core` 领域 ID 和 provenance schema。
6. 实现本地 demo fixture。
7. 到 P1 时再创建 `causis-adapter-cube`。

## 参考资料

- Cube Core GitHub README: https://github.com/cube-js/cube
- Cube data sources documentation: https://cube.dev/docs/product/configuration/data-sources
- Cube pre-aggregations documentation: https://cube.dev/docs/product/data-modeling/reference/pre-aggregations
- Cube Semantic Layer Sync documentation: https://cube.dev/docs/product/apis-integrations/semantic-layer-sync
- Cube SQL API query pushdown blog: https://cube.dev/blog/query-push-down-in-cubes-semantic-layer
