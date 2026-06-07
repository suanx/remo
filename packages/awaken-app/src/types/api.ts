/* ===================================================================
 * API 类型定义
 *
 * 后端 REST API 的请求/响应类型及 SSE 事件类型。
 * 与 AG-UI 协议保持一致。
 * =================================================================== */

/**
 * 服务器健康状态响应。
 */
export interface HealthResponse {
  /** 服务状态（如 "ok"） */
  status: string;
  /** 服务版本号 */
  version: string;
  /** 服务已运行秒数 */
  uptime_seconds: number;
  /** 当前运行的 Agent 数量 */
  agents_running: number;
}

/**
 * 运行统计数据响应。
 */
export interface StatsResponse {
  /** 总运行次数 */
  total_runs: number;
  /** 总工具调用次数 */
  total_tool_calls: number;
  /** 总 Token 消耗 */
  total_tokens: number;
  /** 活跃线程数 */
  active_threads: number;
  /** 平均响应时间（毫秒） */
  avg_response_time_ms: number;
  /** 今日运行次数 */
  runs_today: number;
  /** 按模型分组的详细统计（可选） */
  stats_by_model?: Record<string, ModelStats>;
}

/**
 * 按模型统计。
 */
export interface ModelStats {
  /** 运行次数 */
  runs: number;
  /** Token 消耗 */
  tokens: number;
  /** 平均延迟（毫秒） */
  avg_latency_ms: number;
}

/**
 * 单次运行记录。
 */
export interface RunRecord {
  /** 运行唯一 ID */
  run_id: string;
  /** Agent ID */
  agent_id: string;
  /** 线程 ID（可选） */
  thread_id?: string;
  /** 运行状态 */
  status: 'running' | 'completed' | 'failed';
  /** 开始时间 */
  started_at: string;
  /** 结束时间（可选） */
  finished_at?: string;
  /** 消息数量 */
  message_count: number;
  /** 工具调用次数 */
  tool_call_count: number;
  /** 总 Token 消耗（可选） */
  total_tokens?: number;
  /** 错误信息（可选） */
  error?: string;
}

/**
 * 对话线程。
 *
 * **Thread 兼容说明**:
 * 后端返回的 JSON 中同时包含 `thread_id` 和 `id` 字段，
 * 以便新旧组件均可正常工作。ApiClient 不做额外转换。
 *
 * @example
 * ```json
 * { "thread_id": "thr_xxx", "id": "thr_xxx", "title": "对话", ... }
 * ```
 */
export interface Thread {
  /** 线程唯一 ID */
  thread_id: string;
  /** 线程标题 */
  title: string;
  /** 创建时间（ISO 8601） */
  created_at: string;
  /** 最后更新时间（ISO 8601） */
  updated_at: string;
  /** 消息数量 */
  message_count: number;
  /** 状态 */
  status: 'active' | 'archived';
  /** 自定义元数据（可选） */
  metadata?: Record<string, unknown>;

  /**
   * thread_id 的别名，用于向后兼容旧版组件。
   * 后端返回的 JSON 中会同时包含此字段。
   * @deprecated 请使用 thread_id
   */
  id: string;
}

/**
 * 创建线程请求参数。
 */
export interface CreateThreadRequest {
  /** 线程标题（可选，后端可自动生成） */
  title?: string;
  /** 绑定的 Agent ID（可选） */
  agent_id?: string;
}

/**
 * 单条消息。
 */
export interface Message {
  /** 消息发送者角色 */
  role: 'user' | 'assistant' | 'tool' | 'system';
  /** 消息内容 */
  content: string;
  /** 工具调用列表（assistant 消息可能有） */
  tool_calls?: ToolCall[];
  /** 工具调用 ID（tool 角色消息回传） */
  tool_call_id?: string;
  /** 工具名称（tool 角色消息回传） */
  tool_name?: string;
  /** Base64 图片数据数组 */
  images?: string[];
  /** 文件附件列表 */
  files?: FileAttachment[];
  /** 消息创建时间（ISO 8601） */
  created_at: string;
}

/**
 * 文件附件（API 层，snake_case 字段）。
 */
export interface FileAttachment {
  /** 文件名 */
  name: string;
  /** MIME 类型 */
  mime_type: string;
  /** 文件大小（字节） */
  size_bytes: number;
  /** Base64 文件数据 */
  data: string;
}

/**
 * 工具调用信息。
 */
export interface ToolCall {
  /** 工具调用唯一 ID */
  id: string;
  /** 工具名称 */
  name: string;
  /** 调用参数 */
  arguments: Record<string, unknown>;
  /** 执行结果（可选） */
  result?: unknown;
  /** 状态 */
  status: 'pending' | 'running' | 'completed' | 'failed';
  /** 开始时间（可选） */
  started_at?: string;
  /** 结束时间（可选） */
  finished_at?: string;
}

/**
 * Agent 配置。
 */
export interface AgentConfig {
  /** Agent 唯一 ID */
  agent_id: string;
  /** 模型名称 */
  model: string;
  /** 模型提供商 */
  provider: string;
  /** 系统提示词（可选） */
  system_prompt?: string;
  /** 温度参数（可选） */
  temperature?: number;
  /** 最大 Token 数（可选） */
  max_tokens?: number;
  /** 启用的工具 ID 列表（可选） */
  tools?: string[];
  /** 多模态配置（可选） */
  multimodal?: MultimodalConfig;
}

/**
 * 多模态配置。
 */
export interface MultimodalConfig {
  /** 视觉模型提供商 */
  vision_provider?: 'openai' | 'anthropic' | 'ollama';
  /** 视觉模型名称 */
  vision_model?: string;
  /** 视觉模型 API Key */
  vision_api_key?: string;
  /** 视觉模型 Base URL */
  vision_base_url?: string;
  /** 最大图片大小（MB） */
  max_image_size_mb?: number;
}

/**
 * SSE 事件类型（AG-UI 协议）。
 *
 * 服务端通过 Server-Sent Events 推送的流式事件。
 */
export type SSEEvent =
  /** 文本 Token（流式输出） */
  | { type: 'token'; data: { text: string } }
  /** 工具调用开始 */
  | { type: 'tool_call_start'; data: { id: string; name: string; arguments: Record<string, unknown> } }
  /** 工具调用结束 */
  | { type: 'tool_call_end'; data: { id: string; result: unknown } }
  /** 消息结束（完整消息元信息） */
  | { type: 'message_end'; data: { role: string; content: string } }
  /** 错误事件 */
  | { type: 'error'; data: { message: string; code?: string } }
  /** 流结束 */
  | { type: 'done' }
  /** 无法识别的事件 */
  | { type: 'unknown'; raw: string };
