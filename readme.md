# Remo

[![协议: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](#开源协议)
[![Rust 1.93+](https://img.shields.io/badge/rust-1.93+-orange)](https://rustup.rs)
[![版本](https://img.shields.io/badge/version-0.5.1--dev-green)](https://github.com/suanx/remo)

**基于阶段执行的 Rust AI Agent 框架 —— 构建一次能力，实时调优行为，服务所有客户端。**

Remo 是一个模块化、插件驱动的框架，用于构建生产级 AI 智能体。它提供基于阶段的执行引擎、类型化工具系统、多协议服务器和丰富的扩展生态——全部基于安全高效的 Rust 语言。

---

## 核心特性

- **基于阶段的执行引擎** —— 确定性智能体循环，包含完整生命周期阶段（RunStart → BeforeInference → AfterInference → BeforeToolExecute → AfterToolExecute → StepEnd → RunEnd），在每个阶段实现精确控制。
- **插件架构** —— 扩展通过 `Plugin` trait 注册，支持状态键、阶段钩子、工具网关钩子和配置模式。未启用的特性零开销。
- **类型化工具系统** —— 工具实现 `TypedTool`，从 Rust 类型自动生成 JSON Schema。支持参数校验、挂起（人机协作）和通过 `StateCommand` 声明副作用。
- **状态管理** —— 线程级和运行级状态键，支持交换律/末次写入胜出合并策略。支持持久化、快照和并发更新。
- **基于阶段的执行引擎** —— 确定性智能体循环，包含完整生命周期阶段（RunStart → BeforeInference → AfterInference → BeforeToolExecute → AfterToolExecute → StepEnd → RunEnd），在每个阶段实现精确控制。
- **插件架构** —— 扩展通过 `Plugin` trait 注册，支持状态键、阶段钩子、工具网关钩子和配置模式。未启用的特性零开销。
- **类型化工具系统** —— 工具实现 `TypedTool`，从 Rust 类型自动生成 JSON Schema。支持参数校验、挂起（人机协作）和通过 `StateCommand` 声明副作用。
- **状态管理** —— 线程级和运行级状态键，支持交换律/末次写入胜出合并策略。支持持久化、快照和并发更新。
- **多协议服务器** —— HTTP 服务器，支持 SSE 流式传输、Agent-to-Agent（A2A）协议、Agent Client Protocol（ACP）和 MCP（Model Context Protocol）。
- **存储后端** —— 可插拔存储，支持内存、文件、PostgreSQL 和 SQLite。
- **21 个扩展** —— 记忆、RAG、多模态、工作流、安全沙箱、Playground、权限、可观测性、MCP、技能、提醒、生成式 UI、延迟加载工具、Web 搜索、LLM 评测、通知推送、语音交互、OpenCode CLI、讯飞星辰 MaaS、Agnes AI Gateway、媒体生成。
- **内置管理 UI** —— React 前端，暗色科技风，登录保护，实时状态监控，模型与通知通道配置。

```
┌──────────────────────────────────────────────────────────────────┐
│               awaken-app（React 管理后台 UI）                     │
│       /admin/chat  /admin/dashboard  /admin/settings             │
├──────────────────────────────────────────────────────────────────┤
│                  remo（门面 crate）                              │
│          单一依赖：导出所有扩展 + 运行时                           │
├──────────────────────────────────────────────────────────────────┤
│  扩展（特性门控 · 21 个）                                         │
│  ┌──────┐ ┌─────┐ ┌──────┐ ┌───────┐ ┌────────┐               │
│  │memory│ │rag  │ │workfl│ │multimo│ │sandbox │               │
│  │play..│ │permi│ │obser│ │mcp │sk│ │remind..│               │
│  │gen-ui│ │defe │ │search│ │eval..│ │notif.. │               │
│  │opcode│ │xfyun│ │agnes│ │media..│ │voice   │               │
│  └──────┘ └─────┘ └──────┘ └───────┘ └────────┘               │
├──────────────────────────────────────────────────────────────────┤
│  remo-runtime                                                    │
│  阶段引擎 · 插件系统 · 智能体循环 · 状态存储                      │
├──────────────────────────────────────────────────────────────────┤
│  remo-runtime-contract · remo-server-contract                  │
│  核心 trait · 状态模型 · 协议类型                                 │
├──────────────────────────────────────────────────────────────────┤
│  remo-stores · remo-server · remo-protocol-a2a              │
│  存储后端 · HTTP 服务器（+静态文件）· A2A/ACP/MCP 线上类型       │
└──────────────────────────────────────────────────────────────────┘
```

## 快速开始

在 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
remo = "0.5"
tokio = { version = "1", features = ["full"] }
```

构建一个最小智能体：

```rust
use remo::prelude::*;
use remo::engine::GenaiExecutor;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = AgentRuntimeBuilder::new()
        .with_agent_spec(
            AgentSpec::new("assistant")
                .with_model_id("gpt-4o-mini")
        )
        .with_provider("openai", Arc::new(GenaiExecutor::new()))
        .with_model(ModelSpec::new("gpt-4o-mini", "openai", "gpt-4o-mini"))
        .build()?;

    let activation = RunActivation::new("thread-1", vec![Message::user("你好！")])
        .with_agent_id("assistant");

    let result = runtime.run_to_completion(activation).await?;
    println!("{}", result.response);

    Ok(())
}
```

## 特性开关

默认启用全部特性（通过 `full` 标志）。可以禁用默认集，按需选择：

```toml
[dependencies]
remo = { version = "0.5", default-features = false, features = ["memory", "rag"] }
```

| 特性 | 扩展 crate | 说明 |
|------|-----------|------|
| `memory` | `remo-ext-memory` | 短期与长期记忆，支持检索 |
| `rag` | `remo-ext-rag` | 文档摄取、分块、关键词检索 |
| `multimodal` | `remo-ext-multimodal` | 文件解析、图片描述、内容路由 |
| `workflow` | `remo-ext-workflow` | 基于 DAG 的工作流执行引擎 |
| `sandbox` | `remo-ext-sandbox` | 工具执行的进程级隔离 |
| `playground` | `remo-ext-playground` | 对话回放、评分、对比 |
| `permission` | `remo-ext-permission` | 工具调用的允许/拒绝/询问策略 |
| `observability` | `remo-ext-observability` | OpenTelemetry + GenAI 语义约定 |
| `mcp` | `remo-ext-mcp` | Model Context Protocol 工具桥接 |
| `skills` | `remo-ext-skills` | 技能发现与调度 |
| `reminder` | `remo-ext-reminder` | 基于工具输出模式的上下文注入 |
| `generative-ui` | `remo-ext-generative-ui` | 工具调用的结构化 UI 渲染 |
| `server` | `remo-server` | HTTP 服务器（SSE、邮箱、协议适配） |
| `search` | `remo-ext-search` | 多引擎 Web 搜索与网页抓取 |
| `evaluator` | `remo-ext-evaluator` | LLM-as-judge 在线评测 |
| `notifications` | `remo-ext-notifications` | 多渠道通知（邮件/钉钉/企微/飞书/Slack/Telegram） |
| `voice` | `remo-ext-voice` | 语音交互（TTS/ASR） |
| `opencode` | `remo-ext-opencode` | OpenCode CLI 集成 + 免费模型自动发现 |
| `xfyun` | `remo-ext-xfyun` | 讯飞星辰 MaaS（星火大模型 + Embedding/Rerank + TTI 图片生成） |
| `agnes` | `remo-ext-agnes` | Agnes AI Gateway（免费 AI API 平台） |
| `media-gen` | `remo-ext-media-gen` | 图片与视频生成（DALL-E 3 / Agnes Image & Video） |
| `media-gen` | `remo-ext-media-gen` | 图片与视频生成（DALL-E 3 / Agnes Image & Video） |

## 工作空间 Crate 一览

工作空间包含 36 个 crate，按层次组织：

### 核心层

| Crate | 说明 |
|-------|------|
| `remo` | 门面 crate——所有功能的单一入口 |
| `remo-runtime` | 基于阶段的执行引擎、插件系统、智能体循环 |
| `remo-runtime-contract` | 核心 trait、状态模型、协议类型 |
| `remo-server-contract` | 服务器/存储合约接口 |
| `remo-contract` | 遗留合约兼容层 |

### 存储与服务器

| Crate | 说明 |
|-------|------|
| `remo-stores` | 存储后端（内存、文件、PostgreSQL、SQLite） |
| `remo-server` | 多协议 HTTP 服务器（SSE、A2A、ACP、MCP） |
| `remo-protocol-a2a` | A2A v1.0 线上类型与辅助工具 |

### 扩展层

| Crate | 说明 |
|-------|------|
| `remo-ext-memory` | 短期与长期记忆，支持关键词检索 |
| `remo-ext-rag` | RAG 管线：摄取 → 分块 → 检索 → 上下文注入 |
| `remo-ext-multimodal` | 文件解析（txt/md/csv/json）、图片描述、内容路由 |
| `remo-ext-workflow` | DAG 执行器，支持拓扑排序与并行分层执行 |
| `remo-ext-sandbox` | 进程级沙箱，支持资源限制与网络策略 |
| `remo-ext-playground` | 对话回放、Jaccard 相似度、评分卡评估 |
| `remo-ext-permission` | 允许/拒绝/询问策略，支持 glob/正则模式匹配 |
| `remo-ext-observability` | OpenTelemetry 链路追踪、Prometheus 指标、GenAI Span |
| `remo-ext-mcp` | Model Context Protocol 客户端，外部工具桥接 |
| `remo-ext-skills` | 技能发现、激活与资源加载 |
| `remo-ext-reminder` | 基于工具输出模式的自动上下文注入 |
| `remo-ext-generative-ui` | 智能体工具调用的结构化 UI 卡片渲染 |
| `remo-ext-deferred-tools` | 懒加载工具，基于搜索的按需发现 |
| `remo-ext-search` | 多引擎 Web 搜索（Tavily/SerpAPI/Bing/Google）与网页抓取 |
| `remo-ext-evaluator` | LLM-as-judge 在线评测，支持自定义评分标准 |
| `remo-ext-notifications` | 多渠道通知推送（邮件/钉钉/企微/飞书/Slack/Telegram） |
| `remo-ext-opencode` | OpenCode CLI + Zen 免费模型自动发现与调用 |
| `remo-ext-xfyun` | 讯飞星辰 MaaS 平台 — 星火大模型推理服务（OpenAI 兼容） |
| `remo-ext-agnes` | Agnes AI Gateway — 免费 AI API 平台（OpenAI 兼容） |
| `remo-ext-media-gen` | 图片与视频生成（DALL-E 3 / Agnes Image & Video） |
| `remo-ext-voice` | 语音交互（TTS 语音合成 + ASR 语音识别） |

### 工具与测试

| Crate | 说明 |
|-------|------|
| `remo-tool-pattern` | 工具调用模式匹配（glob、正则、字段条件） |
| `remo-eval` | 基于固件的回放与评分框架 |
| `remo-doctest` | 文档示例验证 |

## 插件系统

所有扩展通过 `Plugin` trait 集成：

```rust
use remo::plugins::{Plugin, PluginDescriptor, PluginRegistrar};
use remo::state::{KeyScope, MutationBatch, StateKeyOptions};
use remo::{Phase, PluginConfigKey, StateError, PhaseHook, StateCommand};

struct MyPlugin;

impl Plugin for MyPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: "my_plugin" }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        // 注册状态键
        registrar.register_key::<MyStateKey>(StateKeyOptions {
            persistent: true,
            retain_on_uninstall: false,
            scope: KeyScope::Thread,
        })?;

        // 注册阶段钩子
        registrar.register_phase_hook(
            "my_plugin", Phase::BeforeInference, MyHook,
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![ConfigSchema::for_key::<MyConfigKey>()]
    }
}
```

## 类型化工具

工具使用 Rust 类型自动推导 JSON Schema：

```rust
use remo::contract::tool::{TypedTool, ToolCallContext, ToolOutput, ToolError};
use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Deserialize, JsonSchema)]
struct CalculatorArgs {
    /// 要求值的数学表达式
    expression: String,
}

struct CalculatorTool;

impl TypedTool for CalculatorTool {
    type Args = CalculatorArgs;

    fn tool_id(&self) -> &str { "calculator" }
    fn name(&self) -> &str { "计算器" }
    fn description(&self) -> &str { "计算数学表达式" }

    async fn execute(
        &self,
        args: Self::Args,
        _ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        // 计算表达式...
        Ok(ToolResult::success("calculator", json!({"result": 42})).into())
    }
}
```

## 扩展详解

### 记忆系统（`remo-ext-memory`）

提供短期记忆（滑动窗口）和长期记忆（关键词评分），支持自动巩固和时间衰减。

| 工具 | 说明 |
|------|------|
| `memory:store` | 存储新记忆条目，支持重要性评分和标签 |
| `memory:recall` | 跨两个存储按关键词检索记忆 |
| `memory:list` | 按类型列出已存储的记忆（short_term / long_term / all） |

### RAG 管线（`remo-ext-rag`）

文档摄取管线，支持可配置的分块策略和基于关键词的检索。

| 工具 | 说明 |
|------|------|
| `rag:ingest` | 摄取文档，分块后存储以备检索 |
| `rag:query` | 检索知识库，返回相关性排序结果 |
| `rag:list` | 列出所有已摄取文档 |
| `rag:delete` | 删除文档及其所有分块 |

**分块策略**：`sentence`（按句子）· `paragraph`（按段落）· `recursive`（递归，默认）

### 工作流引擎（`remo-ext-workflow`）

基于 DAG 的工作流执行，支持拓扑排序和并行分层执行。

| 工具 | 说明 |
|------|------|
| `workflow:start` | 从 JSON 规格启动新工作流 |
| `workflow:status` | 查看工作流执行进度 |

**节点类型**：`Llm` · `Tool` · `Condition` · `Passthrough`

### 安全沙箱（`remo-ext-sandbox`）

进程级隔离，用于执行不受信任的工具，支持可配置的资源限制。

```yaml
# Agent 规格配置
sandbox:
  provider: process       # process | docker | wasm
  max_memory_mb: 256
  max_cpu_time_s: 30
  network_policy: outbound_only  # disabled | outbound_only | full
```

### Playground（`remo-ext-playground`）

对话回放引擎，支持 Jaccard 相似度评分和并排对比。

**评分指标**：准确性 · 相关性 · 延迟 · 成本 · 综合分

### Web 搜索（`remo-ext-search`）

多引擎 Web 搜索与网页内容抓取工具。

| 工具 | 说明 |
|------|------|
| `search:web` | 通过配置的搜索引擎（Tavily/SerpAPI/Bing/Google）搜索互联网 |
| `search:fetch` | 抓取指定 URL 的网页内容并转为 Markdown |

**配置示例**：
```yaml
search:
  provider: tavily        # tavily | serpapi | bing | google
  api_key: "${TAVILY_API_KEY}"
  max_results: 5
```

### LLM 评测（`remo-ext-evaluator`）

LLM-as-judge 在线评测工具，支持自定义评分标准。

| 工具 | 说明 |
|------|------|
| `evaluate:evaluate_response` | 对单条问答对进行质量评分 |
| `evaluate:evaluate_conversation` | 评估整个对话历史 |

**评分维度**：相关性 · 连贯性 · 完整性 · 自定义权重

### 通知推送（`remo-ext-notifications`）

多渠道消息通知推送工具，支持 6 种通道。

| 工具 | 说明 |
|------|------|
| `notifications:send_email` | 通过 SMTP 发送邮件通知 |
| `notifications:send_dingtalk` | 发送钉钉机器人消息（text/markdown） |
| `notifications:send_wecom` | 发送企业微信机器人消息 |
| `notifications:send_feishu` | 发送飞书机器人消息（text/interactive 卡片） |
| `notifications:send_slack` | 发送 Slack Incoming Webhook 消息 |
| `notifications:send_telegram` | 通过 Telegram Bot API 发送消息 |
### 语音交互（`remo-ext-voice`）

| `voice:speech_to_text` | 将语音转为文本（Whisper/Azure） |

### OpenCode 集成（`remo-ext-opencode`）

OpenCode CLI 集成与免费模型自动发现。

| 工具 | 说明 |
|------|------|
| `opencode:exec` | 执行 OpenCode CLI 进行代码生成与项目级任务 |
| `opencode:list_models` | 列出 OpenCode Zen 可用模型（含免费模型自动发现） |
| `opencode:check_cli` | 检查 OpenCode CLI 是否已安装并提供安装指引 |

**免费模型**：DeepSeek V4 Flash Free · Big Pickle · MiMo V2.5 Free · Nemotron 3 Ultra Free

### 媒体生成（`remo-ext-media-gen`）

图片与视频生成工具。

| 工具 | 说明 |
|------|------|
| `media:generate_image` | 文本生成图片（DALL-E 3 / Agnes Image Flash / OpenAI 兼容） |
| `media:generate_video` | 文本生成视频（Agnes Video / OpenAI 兼容） |

**支持模型：**
| 供应商 | 模型 | 类型 |
|--------|------|------|
| OpenAI | `dall-e-3` | 图片生成（1024x1024 / 1792x1024 / 1024x1792） |
| Agnes AI | `agnes-image-2.0-flash` / `agnes-image-2.1-flash` | 图片生成 |
## 管理后台 UI

Remo 内置了一个完整的管理后台前端，运行在同一个端口上，无需额外部署。

| 路径 | 功能 | 鉴权 |
|------|------|:----:|
| `/admin/login` | 管理员登录（输入 `REMO_ADMIN_API_BEARER_TOKEN`） | ❌ 公开 |
| `/admin/chat` | AI 对话界面（流式 SSE、图片上传、文件附件） | ✅ |
| `/admin/dashboard` | 实时监控仪表盘（健康检查、统计、运行记录） | ✅ |
| `/admin/settings` | 全局配置（模型、Vision、OpenCode、通知通道等） | ✅ |

### 配置说明

前端 `/admin/settings` 页面支持配置：
- **Provider 配置** —— OpenAI / Anthropic / DeepSeek / Ollama / 讯飞星辰 / Agnes AI
- **Vision** —— 图片识别（OpenAI / Anthropic / Ollama）
- **OpenCode** —— 免费模型自动发现 + CLI 工具开关
- **媒体生成** —— DALL-E 3 / Agnes Image & Video 的 API Key 与模型选择
- **讯飞星辰 MaaS** —— 区域选择、对话模型、Embedding/Rerank 模型、TTI 图片生成参数
- **通知通道** —— 6 个通道的 Webhook URL 和凭据，每个可独立启用

## 多协议服务器

`remo-server` crate 提供：

- **SSE 流式传输** —— 基于 Server-Sent Events 的实时智能体响应
- **A2A 协议** —— 智能体间委托与任务管理
- **ACP** —— Agent Client Protocol 集成
- **MCP** —— Model Context Protocol 外部工具桥接
- **邮箱** —— 线程级消息持久化与投递

## Docker 部署

```bash
# 一键启动（含 SQLite 持久化 + 内置前端）
docker compose up -d

# 查看日志
docker compose logs -f

# 停止
docker compose down
```

首次启动后，打开 **http://localhost:3000/admin/login**，使用 `docker-compose.yml` 中配置的 Token 登录。

### 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `REMO_ADDRESS` | `0.0.0.0:3000` | 监听地址 |
| `REMO_LOG` | `remo_server=info,...` | 日志级别（tracing/env-filter） |
| `REMO_ADMIN_API_BEARER_TOKEN` | `changeme-secret-token` | Admin API 鉴权 Token（必填） |
| `REMO_DATA_DIR` | `/data` | 数据目录（SQLite 持久化） |
| `REMO_STATIC_DIR` | `/app/static` | 前端静态文件目录（Docker 自动设置） |

### Docker 镜像

镜像自动构建并推送到 `ghcr.io/suanx/remo:latest`：
- 多阶段构建（前端 → Rust 编译 → 运行阶段）
- 内置前端 UI 到 `/app/static`
- 自动设置 `REMO_STATIC_DIR`

```bash
# 手动构建
docker build -t ghcr.io/suanx/remo .

# 运行
docker run -p 3000:3000 -e REMO_ADMIN_API_BEARER_TOKEN=my-token ghcr.io/suanx/remo
        docker run -p 3000:3000 -e REMO_ADMIN_API_BEARER_TOKEN=my-token ghcr.io/suanx/remo

GitHub Actions 自动构建流程参见 `.github/workflows/docker.yml`——推送 `main` 分支或 `v*` 标签时自动构建并推送到 `ghcr.io`。

## 供应商一览

| 供应商 | 类型 | 接入方式 |
|--------|------|----------|
| **OpenAI** | OpenAI 原生 | `api_key` |
| **Anthropic** | Anthropic 原生 | `api_key` |
| **DeepSeek** | OpenAI 兼容 | `api_key` + `base_url` |
| **讯飞星辰 MaaS** | **OpenAI 兼容** | `api_key` + `base_url`（多区域） |
| **Agnes AI** | **OpenAI 兼容** | `api_key` + `base_url` |
| **Ollama** | Ollama 原生 | `base_url` |
| **Groq** | OpenAI 兼容 | `api_key` + `base_url` |
| **OpenCode Zen** | OpenAI 兼容 | `api_key`（可选，免费模型无需 Key） |

### 讯飞星辰 MaaS（`remo-ext-xfyun`）

[讯飞星辰 MaaS 平台](https://www.xfyun.cn/) 提供星火大模型推理服务，使用 **OpenAI 兼容 API 协议**。

**接入区域：**
| 区域 | Base URL |
|------|----------|
| 华北-北京 | `https://maas-api.cn-huabei-1.xf-yun.com/v1` |
| 华东-上海 | `https://maas-api.cn-east-3.xf-yun.com/v1` |
| 华南-广州 | `https://maas-api.cn-south-1.xf-yun.com/v1` |

| `qwen3.5-2b` | 通义千问 3.5 2B 轻量模型 |

**Embedding & Rerank 服务：**
| 服务 | 路径 | 默认模型 | 说明 |
|------|------|---------|------|
| Embedding | `POST {base_url}/embeddings` | `sde0a5839` | 将文本转换为向量表示 |
| Rerank | `POST {base_url}/rerank` | `s125c8e0e` | 对文档按查询相关性重排序 |

**可用工具：**
| 工具 ID | 说明 |
|---------|------|
| `xfyun:get_embedding` | 调用 Embedding API，返回向量（前5维 + 维度 + token用量） |
| `xfyun:rerank_documents` | 调用 Rerank API，返回按相关性得分降序排列的文档 |
| `xfyun:generate_image` | TTI 图片生成（星火大模型 / Kolors），返回 base64 图片 |

**TTI 图片生成：**
| 项 | 说明 |
|------|------|
| 端点 | `POST https://maas-api.cn-huabei-1.xf-yun.com/v2.1/tti` |
| 鉴权 | Bearer Token + `app_id`（请求体 header 中） |
| 参数 | width/height、steps、guidance_scale、seed、scheduler |
| 调度器 | `DPM++ 2M Karras` / `DPM++ SDE Karras` / `DDIM` / `Euler a` / `Euler` |
| 回复 | base64 编码图片（`payload.choices.text[0].content`） |

**配置方式：**
1. 在[讯飞开放平台](https://www.xfyun.cn/) 创建应用，获取 API Key
2. 在前端 `/admin/settings` 页面配置 API Key、区域和模型
3. 或在 Provider 中选择 `xfyun`，手动填写 Base URL 使用

## 开发指南

### 环境要求

- Rust 1.93+（参见 `rust-toolchain.toml`）
- Windows 上需要 MSVC Build Tools

### 构建

```bash
cargo build --workspace
```

### 编译检查

```bash
cargo check --workspace
```

### 代码规范检查

```bash
cargo clippy --workspace
```

## 开源协议

本项目采用以下任一协议：

- [MIT 许可证](LICENSE-MIT)
- [Apache 许可证 2.0](LICENSE-APACHE)

由你选择。
