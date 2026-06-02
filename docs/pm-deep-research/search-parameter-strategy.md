# 搜索参数策略：配置 vs 接口 —— 业务逻辑 / 官方 MCP 参照 / 需求建议

> 目的：回答"Exa 这类搜索参数（`type`/`category`/内容深度/`maxAgeHours` …）应该写进配置文件、还是由接口/运行时决定"。本文给开发者讲清**业务逻辑**、**官方 Exa MCP 的取舍**、以及**我们从需求出发的建议**。
> 引擎边界：搜索引擎（`lapis-search`）由上游 **4o3F** 维护，PM DeepResearch 是消费方（ADR-0002）。因此本文是**提给 4o3F 的接口扩展需求**，不含 PM DeepResearch 侧代码改动——与 [`orchestration-interface.md §6`](orchestration-interface.md) 同一模式。

---

## 0. 这份文档解决什么

一句话问题：**Exa 搜索的一堆参数，是写死在 `lapis.toml` 里，还是由调用方（接口/Layer2）按需传？**

结论先行（两层判断）：

1. **参数属于"调用时/接口"层，不属于配置文件**——扩展通用搜索接口的语义，而不是为 Exa 单开一段配置。`lapis.toml` 只保留基础设施。
2. **每一项还要再切一刀：哪些提给上游 4o3F（引擎/接口/配置能力），哪些 PM DeepResearch 在 Skill 层自己做（策略取值 / prompt / 调用编排）。** 这一刀是本文最重要的产出（见 §5）——PM DeepResearch 是引擎消费方，要分清"提需求"与"自己干"。

下文用业务逻辑 + 官方 MCP 实证支撑第 1 点，用归属表落实第 2 点。

---

## 1. 三档现实：这不是"二选一"

"config vs 谁决定" 的真实结构是**三档**，代码里已经存在，`max_results` 就是现成范本：

| 档位 | 载体 | 现有字段 | 决策频率 | 性质 |
|---|---|---|---|---|
| 基础设施 | `lapis.toml` → `SearchProviderEndpoint` | `enabled / base_url / api_key_env / timeout_ms` | 部署时一次 | 环境/凭据 |
| 策略 | `SearchPolicy`（Layer1 每次研究任务装配，给**默认 + 上限**） | `max_results_per_query / freshness / *_domains / language / region` | 每个研究任务一次 | 本次研究的策略 |
| 运行时 | aspect agent 每条 query | `query` + 每条 `max_results`（受策略上限校验） | 每条搜索 | 战术 |

> 范本：`max_results` 由策略给上限（`crates/lapis-workflow/src/policy.rs:129`），运行时逐条在上限内挑。**所有新搜索参数都应套这一机制，不引入新概念。**

判断一个参数放哪，只问两件事：① 它随单条 query 变吗？② 它是成本杠杆吗（需要护栏）？

**再叠一层"谁拥有"**（这是 §5 那一刀的依据）：

- **基础设施 config + 接口 schema = 上游 4o3F**（引擎归上游维护，ADR-0002）。
- **策略取值（往字段里填什么）= PM DeepResearch Skill 层**（Layer1 装配，见 [`orchestration-interface.md §4`](orchestration-interface.md)）。
- **运行时 agent loop 在引擎内执行，但其行为由 Skill 注入的 `aspect_agent_prompt` 决定**。

所以同一个参数常常横跨两边：**字段与上限在上游，取值与时机在 Skill**。`type` 即如此——引擎接受它并校验上限，但"这次研究用哪档"由 Skill 按 tier 组装、可随时在 Skill 调。

---

## 2. 现状（已核对源码）

### 2.1 通用接口 `SearchRequest`（`crates/lapis-search/src/types.rs:24-34`）

当前字段：`provider / query / max_results / freshness{since,until} / language / region / include_domains / exclude_domains`。
**没有** `type` / `category` / 内容深度 / `maxAgeHours`。

### 2.2 Exa provider（`crates/lapis-search/src/provider/exa.rs`）

发往 Exa `/search` 的 body 当前为 6 个字段：`query / num_results / include_domains / exclude_domains / start_published_date / end_published_date`——即不含内容控制（`contents`）、`type`、`category`、`maxAgeHours`。

> 这里只陈述接口现状，不评判引擎实现。"需要返回内容"作为**能力需求**向上游提（见 §5.2-R7），不在此讨论现状是否欠缺。

### 2.3 配置 `SearchProviderEndpoint`（`crates/lapis-config/src/types.rs`，`lapis.example.toml`）

```toml
[search.providers.exa]
enabled = false
base_url = "https://api.exa.ai"
api_key_env = "EXA_API_KEY"
timeout_ms = 30000
model = ""
```

只承载基础设施，provider 无关。**当前配置里没有任何 Exa 专属调优值**——这是正确方向，本文建议保持。

---

## 3. 官方 Exa MCP 策略参照（已装 + 官方仓库）

我们装了官方 Exa MCP；它的设计直接回答了"配置 vs 接口"。

### 3.1 工具拆分：简单 vs 高级

| 工具 | 参数面 | 取舍 |
|---|---|---|
| `web_search_exa` | **只有** `query` + `numResults` | 常用场景：其余参数全在服务端默认，调用方不操心 |
| `web_search_advanced_exa` | ~25 个参数全开放 | 要精调时才用：`type` / `category` / 内容深度 / 时效 / 域名 / 文本过滤 / 子页 / 地域 … |
| `deep_researcher_start` | `instructions` + `model`(exa-research-fast/exa-research/pro) | **独立的 agentic 研究工具**，不是 search 的一个档 |

### 3.2 高级工具实际暴露的关键参数（取自已装工具 schema）

- `type`: 枚举 **`auto` / `fast` / `instant`**（`auto`=高质量且兼容所有过滤器，推荐）。注意：**没有 `deep`**——深度 agentic 研究是上面那个独立工具。
- `category`: `company / research paper / news / pdf / github / personal site / people / financial report`。
- 内容深度（多个独立开关，非单一枚举）：`enableHighlights` / `enableSummary` / `highlightsMaxCharacters` / `textMaxCharacters` / `contextMaxCharacters` / `summaryQuery` / `highlightsQuery` / `subpages`。
- 时效：`maxAgeHours`（0=必抓新鲜，省略=用缓存+新鲜回退）+ `livecrawlTimeout`。
- 过滤：`includeDomains` / `excludeDomains` / `includeText` / `excludeText` / `startPublishedDate` / `endPublishedDate` / `userLocation`。
- ⚠️ category 约束：`company` 禁用域名/日期过滤，`people` 禁用日期/文本过滤和 excludeDomains——**category 与域名/时效参数互斥需在调用侧处理**。

### 3.3 配置策略

官方 MCP 的"配置"只有两项：`EXA_API_KEY`（凭据）+ `tools=`（启用哪些工具的白名单，**能力门控**）。**零调优默认值**——所有 num/type/category/时效全靠 call-time 传。

### 3.4 可借鉴的四条取舍

1. **调优不进配置**：配置只放凭据 + 能力门控（启用哪些工具）。
2. **简单/高级双层** ≈ 我们的"策略默认 + 逐条覆盖"：常用走默认，要精调才展开全参数。
3. **意图工具被合并为 `category` 参数**：`company_research_exa` 已标 `[Deprecated: Use web_search_advanced_exa instead]`——证明 `category` 是 **call-time** 决策，不该为它单开 provider 或写配置。
4. **深度研究是独立工具，不是 search 档**：印证别把 Exa 的 agentic 深度模式接成常规检索路径。

---

## 4. 各参数业务逻辑 → 对应 PM DeepResearch 需求

| 参数 | 控制什么 | 成本 or 相关性 | 随 query 变？ | 对应 PM DeepResearch 需求 |
|---|---|---|---|---|
| `type`(auto/fast/instant) | 检索方法/速度 | **成本/延迟最大杠杆** | 否，更像"这次多深" | 由 Skill 按 tier/决策意图给默认（具体取值不在本文固化，见 §5.2-B 组 R3） |
| Exa deep_researcher | 自带规划+综合的 agentic 研究 | 成本极高 | — | **与 Layer1/2 自身编排重叠**，接了=双层 agent 付费 + 证据溯源失控 |
| `category` | 内容类型定向 | 相关性 | **是，强随维度** | 竞品图谱→company；市场规模→research paper/financial report；口碑→news |
| 内容深度(highlights/summary/text) | 返回正文粒度 | **token 预算** | 部分 | 直击 PM DeepResearch"token 浪费"痛点：默认 highlights 省 token，高价值 query 升 full |
| `maxAgeHours`(+livecrawlTimeout) | 缓存新鲜度 / 是否重爬 | 成本/延迟 | 是，随决策意图 | 时效维度抓新鲜、历史维度走缓存；**与 `Freshness`（按发布日筛文档）是两件事** |
| numResults / domains / published_date | 数量 / 范围 | — | 已处理 | 沿用现有"策略上限 + 逐条" |

---

## 5. 建议：先切"上游 vs Skill"，再落需求

本节是全文核心。每项先判归属，再给需求。

### 5.1 归档结论表（带"归属"）

| 项 | 放哪一档 | 归属 | 要点 |
|---|---|---|---|
| 接口扩展（接受中立语义参数 + 上限机制） | 接口 / schema | **上游 4o3F** | 加 provider 中立参数（深度 / 内容深度 / 时效），各 provider 内翻译；保 Grok/Exa 对称 |
| 返回内容（`contents`） | 接口 / provider | **上游 4o3F** | 搜索结果需能携带正文/摘要，供内容深度策略消费 |
| `maxAgeHours` vs `Freshness` 概念分离 | schema | **上游 4o3F** | 两个独立概念，不合并 |
| 抽象方式（中立枚举 / 透传 / 内置） | schema 设计 | **上游 4o3F**（我们给建议） | 见 §5.3 |
| 成本可被配置影响 | config | **上游 4o3F**（开放，不指定键） | 我们只提"成本应是配置可影响的维度"，具体机制/键由上游定 |
| `type` / 内容深度 / 时效的**取值** | 策略 | **Skill 层** | 按 tier + 决策意图装配；取值不在本文固化、可随时在 Skill 调 |
| `category` 选择 | 策略 / 运行时 | **Skill 层** | Layer1 预设 或 Layer2 逐条均可（开放，见 R4） |
| 不接 `deep_researcher` 为常规路径 | 能力边界 | **横跨** | 引擎默认不启用 / 留门控 + Skill 不调用 |
| 上限内 runtime 覆盖 | 运行时 | **横跨** | 引擎做上限校验 + `aspect_agent_prompt` 指导 agent |
| numResults / domains / 日期 | 已有 | 已有 | 沿用"策略上限 + 逐条"，不动 |

### 5.2 需求清单（按归属分组）

**A 组——提给上游 4o3F（引擎 / 接口 / 配置能力）**

- **R1｜接口扩展**：通用搜索接口增加一组 **provider 中立语义参数**（搜索深度 / 内容深度 / 时效要求），各 provider 内部翻译成原生参数；保持 Grok/Exa 接口对称。引擎同时提供"上限校验"机制（沿用 `max_results` 范式，`policy.rs:115-142`）。
- **R6｜概念分离**：时效（决定是否重爬）与 `Freshness`（按发布日期筛文档）作为两个独立概念，不合并。
- **R7｜返回内容能力**：搜索结果需能携带正文 / 摘要（`contents`）。
  - *为什么需要*：PM DeepResearch 的"内容深度"策略（highlights 省 token vs full text 深读）只有在结果能带正文时才成立——没有内容返回能力，整条内容深度策略无处落地。
  - *业务意义*：内容粒度直接决定 token 成本与证据可读性，是 PM DeepResearch "省 token / 宁少但真"的核心抓手。
- **R-cost｜成本可被配置影响（开放项）**：成本应是配置可以合理影响的维度。PM DeepResearch **不指定具体配置键**，把"如何让部署方控制成本"的机制交给上游设计。
- **R2｜配置纪律**：`lapis.toml` 只承载基础设施（`enabled/base_url/api_key/timeout`）；**不要**把逐查询调优值塞进配置。

**B 组——PM DeepResearch 在 Skill 层自己做（不提上游）**

- **R3｜策略取值**：由 **Skill（Layer1 装配）** 按 tier + 决策意图，为搜索深度 / 内容深度 / 时效填默认值。引擎只给字段与上限；**取值是 Skill 的事，可随时在 Skill 调，不改引擎、不改用户 config**（取值本身不在本文固化）。
- **R4｜category 选择**：由 Skill 决定研究维度对应的 `category`。**开放**——既可 Layer1 拆维度时预设，也可 Layer2 aspect agent 逐条选，两者皆可，落地时再定。注意 `company`/`people` 与域名/时效过滤互斥，需在选择侧处理。
- **R5（Skill 侧）｜不调用 deep 模式**：Skill 不把 Exa 深度 agentic 研究作为常规检索路径。

**横跨项**

- **R5（引擎侧）｜deep 模式默认关**：引擎不默认启用 deep agentic 模式；若提供，**留口子、默认关**，由配置门控。理由：Exa 深度模式自带规划+综合，与 PM DeepResearch 的 Layer1/2 编排重叠，常规接入 = 双层 agent 付费 + 证据溯源失控。
- **上限内 runtime 覆盖**：引擎提供上限校验，Skill 通过 `aspect_agent_prompt` 指导 agent 在上限内选 query / category / 深度。

### 5.3 待上游拍板的设计点（我们给建议）

R1 怎么落——`SearchRequest` 是 Grok/Exa **共用的中立抽象**，不能照抄 Exa 字面字段。三条路供 4o3F 选：

- **A. 中立语义枚举**（我们推荐）：新增 `depth` / `content_level` / `recency` 等中立枚举，各 provider 内翻译。✅ 类型安全、Grok/Exa 对称；❌ Grok 无对应物时需做空映射。
- **B. provider 透传 map**：`provider_overrides: { exa: {...} }`。✅ 快；❌ Exa 细节漏进通用层、丢校验。
- **C. 全锁进 ExaProvider**：中立层只传一个"深度提示"，Exa 字面枚举全留在 `exa.rs`。✅ 抽象最干净；❌ 上层精调要多绕一层。

---

## 6. 落地路径

- **A 组（§5.2-A）= 提给 4o3F 的接口 / 能力需求**，与 [`orchestration-interface.md §6`](orchestration-interface.md)"引擎边界"约定一致：PM DeepResearch 提需求，不改引擎源码。
- **B 组（§5.2-B）= PM DeepResearch 在 Skill 层自己做**，不依赖上游：策略取值、`category` 选择、不调用 deep 模式，都落在 Layer1/Layer2 的 prompt 与装配逻辑里。
- **横跨项**：Skill 侧先落实（不调用 deep 模式、上限内覆盖）；引擎侧"默认关 + 门控"随 A 组一并提交。
- 依赖关系：A 组的 **R7（返回内容能力）** 是 B 组"内容深度取值策略"的前提——内容深度策略要等上游具备内容返回能力后才有意义。
- **R5（Skill 侧不接 deep）现在就该遵守**，不依赖任何接口扩展。

---

## 参照

- 已装官方 Exa MCP 工具 schema（`web_search_exa` / `web_search_advanced_exa` / `deep_researcher_start`）
- 官方仓库：`github.com/exa-labs/exa-mcp-server`
- Exa 文档：`https://exa.ai/docs/reference/search-best-practices`
- 源码：`crates/lapis-search/src/types.rs`、`crates/lapis-search/src/provider/exa.rs`、`crates/lapis-workflow/src/policy.rs`、`crates/lapis-config/src/types.rs`
- 相关 spec：[`orchestration-interface.md`](orchestration-interface.md)、[`../configuration.md`](../configuration.md)、[`../mcp-usage.md`](../mcp-usage.md)、ADR-0002
