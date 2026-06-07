/* ===================================================================
 * SSE 流式读取器
 *
 * 基于 fetch + ReadableStream 解析 SSE（Server-Sent Events），
 * 支持手动取消（abort），返回异步生成器供消费方逐事件处理。
 * =================================================================== */

import type { SSEEvent } from '../types/api';

/** SSE 流式读取器返回类型 */
export interface SSEStreamResult {
  /** 异步生成器，逐条产出解析后的 SSEEvent */
  stream: AsyncGenerator<SSEEvent, void, void>;
  /** 取消请求 */
  abort: () => void;
}

/** 默认请求超时（毫秒） */
const DEFAULT_TIMEOUT_MS = 30_000;

/**
 * 创建 SSE 流式连接，使用 fetch + ReadableStream 解析事件流。
 *
 * @param url         - 请求 URL
 * @param body        - POST JSON 请求体（会被 JSON.stringify）
 * @param timeoutMs   - 超时时间（毫秒），默认 30_000，传 0 表示不设超时
 * @param extraHeaders - 额外请求头（可选）
 * @returns `{ stream, abort }` —— stream 为异步生成器，abort 用于取消
 *
 * @example
 * ```ts
 * const { stream, abort } = createSSEStream('/api/chat', {
 *   messages: [{ role: 'user', content: 'Hello' }],
 * });
 *
 * try {
 *   for await (const event of stream) {
 *     if (event.type === 'token') console.log(event.data.text);
 *     if (event.type === 'done') break;
 *   }
 * } finally {
 *   abort();
 * }
 * ```
 */
export function createSSEStream(
  url: string,
  body: unknown,
  timeoutMs: number = DEFAULT_TIMEOUT_MS,
  extraHeaders?: Record<string, string>,
): SSEStreamResult {
  const abortController = new AbortController();
  let aborted = false;

  /** 取消请求 */
  function abort(): void {
    aborted = true;
    abortController.abort();
  }

  /** 实际的异步生成器 */
  async function* generate(): AsyncGenerator<SSEEvent, void, void> {
    // 超时计时器
    let timeoutId: ReturnType<typeof setTimeout> | undefined;
    if (timeoutMs > 0) {
      timeoutId = setTimeout(() => {
        if (!aborted) {
          abortController.abort(new Error(`SSE 请求超时（${timeoutMs}ms）`));
        }
      }, timeoutMs);
    }

    try {
      const response = await fetch(url, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Accept: 'text/event-stream',
          'Cache-Control': 'no-cache',
          ...extraHeaders,
        },
        body: JSON.stringify(body),
        signal: abortController.signal,
      });

      if (!response.ok) {
        const errBody = await response.text().catch(() => '');
        throw new Error(`SSE 请求失败 [${response.status}]: ${errBody || response.statusText}`);
      }

      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('响应体不可读（body 为空）');
      }

      const decoder = new TextDecoder();
      let buffer = '';

      while (!aborted) {
        const { done, value } = await reader.read();
        if (done) break;

        buffer += decoder.decode(value, { stream: true });

        // SSE 协议：事件以 \n\n 分隔
        const parts = buffer.split('\n\n');
        // 保留最后一段（可能不完整）
        buffer = parts.pop() || '';

        for (const part of parts) {
          if (aborted) break;

          const event = parseSSEPart(part);
          if (event) {
            yield event;
          }
        }
      }

      // 处理缓冲区中剩余的数据
      if (buffer.trim() && !aborted) {
        const event = parseSSEPart(buffer);
        if (event) {
          yield event;
        }
      }
    } catch (error: unknown) {
      if (aborted) return; // 手动取消不抛出

      if (error instanceof DOMException && error.name === 'AbortError') {
        return; // 取消操作，静默退出
      }

      // 将错误包装为 SSEEvent 产出
      yield {
        type: 'error',
        data: {
          message: error instanceof Error ? error.message : String(error),
          code: 'STREAM_ERROR',
        },
      };
    } finally {
      if (timeoutId !== undefined) {
        clearTimeout(timeoutId);
      }
    }
  }

  return { stream: generate(), abort };
}

/* ---- 内部解析 ---- */

/**
 * 解析一段 SSE 数据块（以 \n 分隔的行）。
 *
 * SSE 标准格式：
 * ```
 * event: <type>
 * data: <JSON 或纯文本>
 * ```
 * 或简写为：
 * ```
 * data: <JSON 包含 type 字段>
 * ```
 *
 * @param part - 以 \n 连接的原始文本块
 * @returns 解析后的 SSEEvent，无法解析时返回 null
 */
function parseSSEPart(part: string): SSEEvent | null {
  const lines = part.split('\n').map((l) => l.trim()).filter(Boolean);
  if (lines.length === 0) return null;

  let eventType: string | null = null;
  let dataPayload: string | null = null;

  for (const line of lines) {
    if (line.startsWith('event: ')) {
      eventType = line.slice(7).trim();
    } else if (line.startsWith('data: ')) {
      dataPayload = line.slice(6).trim();
    } else if (line.startsWith('data:')) {
      dataPayload = line.slice(5).trim();
    } else if (line === 'data: [DONE]') {
      return { type: 'done' };
    }
  }

  // 没有 data 字段则忽略
  if (dataPayload === null) return null;

  // 结束标记
  if (dataPayload === '[DONE]') {
    return { type: 'done' };
  }

  // 尝试解析 JSON
  let parsed: Record<string, unknown>;
  try {
    parsed = JSON.parse(dataPayload) as Record<string, unknown>;
  } catch {
    // 非 JSON 数据 —— 按未知事件处理
    return { type: 'unknown', raw: dataPayload };
  }

  // 如果明确指定了 event 类型，优先使用
  const type = eventType || (parsed.type as string) || (parsed.event as string) || '';

  switch (type) {
    case 'token':
    case 'text':
      return {
        type: 'token',
        data: {
          text: (parsed.data as { text?: string })?.text
            || (parsed as { text?: string }).text
            || (parsed.content as string)
            || '',
        },
      };

    case 'tool_call_start':
    case 'tool_call': {
      const toolData = parsed.data as Record<string, unknown> || parsed;
      return {
        type: 'tool_call_start',
        data: {
          id: (toolData.id as string) || '',
          name: (toolData.name as string) || '',
          arguments: (toolData.arguments as Record<string, unknown>)
            || (toolData.args as Record<string, unknown>)
            || (toolData.input as Record<string, unknown>)
            || {},
        },
      };
    }

    case 'tool_call_end':
    case 'tool_result': {
      const resultData = parsed.data as Record<string, unknown> || parsed;
      return {
        type: 'tool_call_end',
        data: {
          id: (resultData.id as string) || '',
          result: resultData.result ?? resultData.content ?? resultData.output,
        },
      };
    }

    case 'message_end':
    case 'finish': {
      const msgData = parsed.data as Record<string, unknown> || parsed;
      return {
        type: 'message_end',
        data: {
          role: (msgData.role as string) || 'assistant',
          content: (msgData.content as string) || '',
        },
      };
    }

    case 'error':
      return {
        type: 'error',
        data: {
          message: ((parsed.data as { message?: string })?.message
            || (parsed as { message?: string }).message
            || '未知 SSE 错误') as string,
          code: ((parsed.data as { code?: string })?.code
            || (parsed as { code?: string }).code) as string | undefined,
        },
      };

    case 'done':
    case 'finish':
      return { type: 'done' };

    case 'heartbeat':
      // 心跳事件 —— 忽略
      return null;

    default:
      // 未知事件类型，保留原始数据
      return { type: 'unknown', raw: dataPayload };
  }
}

/* ---- 遗留兼容导出（旧版 streamChat） ---- */


/**
 * @deprecated 请使用 createSSEStream 替代。
 * 保持向后兼容的 streamChat 包装。
 */
export function streamChat(
  threadId: string,
  agentId: string,
  message: string,
  onEvent: (event: SSEEvent) => void,
  onDone: () => void,
  onError?: (err: Error) => void,
  _opts?: Record<string, unknown>,
): () => void {
  const url = `/v1/ag-ui/threads/${threadId}/runs`;
  const body = {
    agent_id: agentId,
    thread_id: threadId,
    messages: [{ role: 'user', content: message }],
    stream: true,
  };

  let aborted = false;
  const run = async () => {
    const { stream, abort: cancelStream } = createSSEStream(url, body);
    try {
      for await (const event of stream) {
        if (aborted) break;

        // 将 SSEEvent 转换为旧的 AgUiEvent 并回调
        switch (event.type) {
          case 'token':
            onEvent({ type: 'text' as unknown as never, content: event.data.text });
            break;
          case 'tool_call_start':
            onEvent({
              type: 'tool_call' as unknown as never,
              id: event.data.id,
              name: event.data.name,
              args: event.data.arguments,
            });
            break;
          case 'tool_call_end':
            onEvent({
              type: 'tool_result' as unknown as never,
              id: event.data.id,
              name: '',
              result: event.data.result,
            });
            break;
          case 'message_end':
            onEvent({ type: 'done' as unknown as never });
            onDone();
            return;
          case 'error':
            onError?.(new Error(event.data.message));
            return;
          case 'done':
            onDone();
            return;
          default:
            break;
        }
      }
      onDone();
    } catch (err) {
      if (!aborted) {
        onError?.(err instanceof Error ? err : new Error(String(err)));
      }
    }
  };

  run();

  return () => {
    aborted = true;
  };
}
