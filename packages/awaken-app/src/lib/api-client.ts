/* ===================================================================
 * ApiClient —— REST API 客户端
 *
 * 封装后端 REST API 调用，统一错误处理。
 * 支持健康检查、统计、线程管理、Agent 配置等。
 * =================================================================== */

import type {
  HealthResponse,
  StatsResponse,
  RunRecord,
  Thread,
  CreateThreadRequest,
  Message,
  AgentConfig,
  FileAttachment,
} from '../types/api';

/**
 * ApiClient —— 后端 REST API 客户端。
 *
 * 所有方法均会处理 HTTP 错误并抛出包含状态码和响应体的 Error。
 *
 * @example
 * ```ts
 * const api = new ApiClient('/api');
 * const health = await api.health();
 * ```
 */
export class ApiClient {
  /** API 基础路径 */
  private baseUrl: string;

  /**
   * @param baseUrl - API 基础路径，默认 '/api'
   */
  constructor(baseUrl = '/api') {
    this.baseUrl = baseUrl.replace(/\/+$/, '');
  }

  /* ========== 内部工具 ========== */

  /**
   * 发起 JSON 请求。
   *
   * @param path   - 请求路径（相对于 baseUrl）
   * @param init   - fetch 配置项（可选）
   * @returns 解析后的响应数据
   * @throws 当 HTTP 状态码非 2xx 时抛出 Error
   */
  private async request<T>(path: string, init?: RequestInit): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const res = await fetch(url, {
      headers: {
        'Content-Type': 'application/json',
        Accept: 'application/json',
        ...(init?.headers as Record<string, string>),
      },
      ...init,
    });

    if (!res.ok) {
      const body = await res.text().catch(() => '');
      throw new Error(`[ApiClient] HTTP ${res.status} ${res.statusText}${body ? `: ${body}` : ''}`);
    }

    // 204 No Content 或空响应
    const contentType = res.headers.get('content-type') || '';
    if (res.status === 204 || contentType.includes('text/')) {
      return undefined as T;
    }

    return res.json();
  }

  /* ========== 健康检查 ========== */

  /**
   * 健康检查。
   *
   * @returns 服务器状态信息
   */
  async health(): Promise<HealthResponse> {
    return this.request<HealthResponse>('/health');
  }

  /* ========== 统计 ========== */

  /**
   * 获取运行统计。
   *
   * @returns 统计数据
   */
  async stats(): Promise<StatsResponse> {
    return this.request<StatsResponse>('/stats');
  }

  /* ========== 运行记录 ========== */

  /**
   * 获取运行记录列表。
   *
   * @param limit - 限制返回数量（可选）
   * @returns 运行记录数组
   */
  async getRuns(limit?: number): Promise<RunRecord[]> {
    const query = limit != null ? `?limit=${limit}` : '';
    return this.request<RunRecord[]>(`/v1/runs${query}`);
  }

  /**
   * 获取单条运行记录详情。
   *
   * @param runId - 运行 ID
   * @returns 运行记录
   */
  async getRun(runId: string): Promise<RunRecord> {
    return this.request<RunRecord>(`/v1/runs/${encodeURIComponent(runId)}`);
  }

  /* ========== 线程管理 ========== */

  /**
   * 获取线程列表。
   *
   * @returns 线程数组
   */
  async listThreads(): Promise<Thread[]> {
    return this.request<Thread[]>('/v1/threads');
  }

  /**
   * 创建新线程。
   *
   * @param req - 创建线程请求参数（可选）
   * @returns 创建的线程
   */
  async createThread(req?: CreateThreadRequest): Promise<Thread> {
    return this.request<Thread>('/v1/threads', {
      method: 'POST',
      body: JSON.stringify(req ?? {}),
    });
  }

  /**
   * 删除指定线程。
   *
   * @param threadId - 线程 ID
   */
  async deleteThread(threadId: string): Promise<void> {
    return this.request<void>(`/v1/threads/${encodeURIComponent(threadId)}`, {
      method: 'DELETE',
    });
  }

  /**
   * 获取线程消息列表。
   *
   * @param threadId - 线程 ID
   * @returns 消息数组
   */
  async getThreadMessages(threadId: string): Promise<Message[]> {
    return this.request<Message[]>(`/v1/threads/${encodeURIComponent(threadId)}/messages`);
  }

  /* ========== Agent 配置 ========== */

  /**
   * 获取 Agent 配置列表。
   *
   * @returns Agent 配置数组
   */
  async getAgents(): Promise<AgentConfig[]> {
    return this.request<AgentConfig[]>('/v1/agents').catch(() => []);
  }

  /**
   * 更新 Agent 配置。
   *
   * @param config - Agent 配置
   * @returns 更新后的 Agent 配置
   */
  async updateAgent(config: AgentConfig): Promise<AgentConfig> {
    return this.request<AgentConfig>(
      `/v1/agents/${encodeURIComponent(config.agent_id)}`,
      {
        method: 'PATCH',
        body: JSON.stringify(config),
      },
    );
  }

  /* ========== 发送消息（SSE 流式） ========== */

  /**
   * 发送消息到线程并返回 SSE EventSource 用于流式接收回复。
   *
   * **注意**: 本方法使用原生 EventSource (GET)，若后端仅支持 POST 请使用
   * `createSSEStream` 配合 `fetch`。此处为兼容性保留 `EventSource` 签名。
   *
   * @param threadId - 线程 ID
   * @param content  - 消息文本内容
   * @param images   - Base64 图片数据数组（可选）
   * @param files    - 文件附件列表（可选）
   * @returns EventSource 实例
   *
   * @example
   * ```ts
   * const es = await api.sendMessage('thread-123', 'Hello!');
   * es.onmessage = (e) => console.log(e.data);
   * ```
   */
  async sendMessage(
    threadId: string,
    content: string,
    images?: string[],
    files?: FileAttachment[],
  ): Promise<EventSource> {
    const params = new URLSearchParams({ content, stream: 'true' });
    if (images?.length) {
      params.set('images', JSON.stringify(images));
    }
    if (files?.length) {
      params.set('files', JSON.stringify(files));
    }

    const url = `${this.baseUrl}/v1/ag-ui/threads/${encodeURIComponent(threadId)}/runs/stream?${params}`;
    return new EventSource(url);
  }
}

/**
 * 默认 ApiClient 实例（baseUrl = '/api'）。
 *
 * 可直接导入使用，也支持传入自定义 baseUrl 实例化。
 */
export const apiClient = new ApiClient('/api');

/* ===================================================================
 * 以下为向后兼容的旧版函数导出。
 * 新代码请直接使用 `ApiClient` 类或 `apiClient` 实例。
 * =================================================================== */
/** @deprecated 请使用 apiClient.health() */
export const getHealth = (): Promise<HealthResponse> => apiClient.health();
/** @deprecated 请使用 apiClient.stats() */
export const getStats = (): Promise<StatsResponse> => apiClient.stats();
/** @deprecated 请使用 apiClient.getRuns() */
export const listRuns = (limit?: number): Promise<RunRecord[]> => apiClient.getRuns(limit);
/** @deprecated 请使用 apiClient.listThreads() */
export const listThreads = (): Promise<Thread[]> => apiClient.listThreads();
/** @deprecated 请使用 apiClient.createThread() */
export const createThread = (data?: CreateThreadRequest): Promise<Thread> =>
  apiClient.createThread(data);
/** @deprecated 请使用 apiClient.deleteThread() */
export const deleteThread = (threadId: string): Promise<void> =>
  apiClient.deleteThread(threadId);
/** @deprecated 请使用 apiClient.getThreadMessages() */
export const getThreadMessages = (threadId: string): Promise<Message[]> =>
  apiClient.getThreadMessages(threadId);
/** @deprecated 请使用 apiClient.listAgents() */
export const listAgents = (): Promise<AgentConfig[]> => apiClient.getAgents();
/** @deprecated 请使用 apiClient.getRun() */
export const listThreadRuns = (threadId: string): Promise<RunRecord[]> =>
  apiClient.getRuns().then((runs) => runs.filter((r) => r.thread_id === threadId));
