/* ===================================================================
 * useChat —— 聊天状态管理 Hook
 *
 * 管理消息列表、流式发送/接收、图片上传 & base64 编码、文件附件。
 * 依赖 createSSEStream 进行 SSE 流式通信。
 * =================================================================== */

import { useState, useRef, useCallback } from 'react';
import type { ChatMessage, FileAttachment as ChatFileAttachment, ToolCallState } from '../types/chat';
import type { SSEEvent } from '../types/api';
import { createSSEStream } from '../lib/sse';

/* ========== 常量 ========== */

/** 消息 ID 前缀 */
const MSG_PREFIX = 'msg';

/** 生成唯一 ID */
function uid(): string {
  return `${MSG_PREFIX}_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
}

/* ========== 工具函数 ========== */

/**
 * 将 File 对象读取为 Base64 字符串。
 *
 * @param file - 要读取的文件
 * @returns Base64 编码的字符串（含 data URI 前缀）
 */
function readFileAsBase64(file: File): Promise<string> {
  return new Promise<string>((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = () => reject(new Error(`读取文件失败: ${file.name}`));
    reader.readAsDataURL(file);
  });
}

/**
 * 将 File 对象转换为前端的 FileAttachment。
 *
 * @param file     - 源文件
 * @param withData - 是否同时读取 base64 数据（默认 true）
 * @returns 前端 FileAttachment
 */
async function fileToAttachment(file: File, withData = true): Promise<ChatFileAttachment> {
  const id = `file_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
  let data = '';
  let previewUrl: string | undefined;

  if (withData) {
    const base64 = await readFileAsBase64(file);
    // 去除 data URI 前缀，只保留 base64 部分
    const commaIndex = base64.indexOf(',');
    data = commaIndex >= 0 ? base64.slice(commaIndex + 1) : base64;

    // 图片生成预览 URL
    if (file.type.startsWith('image/')) {
      previewUrl = base64;
    }
  }

  return {
    id,
    name: file.name,
    mimeType: file.type,
    sizeBytes: file.size,
    data,
    previewUrl,
  };
}

/* ========== Hook ========== */

/**
 * useChat —— 聊天状态管理 Hook。
 *
 * @param threadId - 当前活跃的线程 ID，传 null 表示未选择线程
 * @returns 聊天状态与操作方法
 *
 * @example
 * ```tsx
 * const { messages, isStreaming, sendMessage, cancelStream, clearMessages } = useChat('thread-123');
 *
 * // 发送纯文本
 * sendMessage('Hello!');
 *
 * // 发送图片
 * sendMessage('看图', ['base64data...']);
 *
 * // 发送文件
 * sendMessage('附件', undefined, [{ id: 'f1', name: 'doc.pdf', mimeType: 'application/pdf', sizeBytes: 1000, data: '...' }]);
 * ```
 */
export function useChat(threadId: string | null) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isStreaming, setIsStreaming] = useState(false);
  const [error, setError] = useState<string | undefined>(undefined);
  const [images, setImages] = useState<string[]>([]);
  const abortRef = useRef<(() => void) | null>(null);

  // 切换线程时清空状态
  const doReset = useCallback(() => {
    setMessages([]);
    setIsStreaming(false);
    setError(undefined);
    setImages([]);
    abortRef.current = null;
  }, []);

  // threadId 变化时自动重置
  const lastThreadRef = useRef<string | null>(null);
  if (lastThreadRef.current !== threadId) {
    lastThreadRef.current = threadId;
    doReset();
  }

  /* ---- 发送消息 ---- */

  /**
   * 发送消息到当前线程，并开启流式接收 AI 回复。
   *
   * @param text   - 消息文本
   * @param images - Base64 图片数据数组（可选）
   * @param files  - 文件附件列表（可选）
   */
  const sendMessage = useCallback(
    async (
      text: string,
      images?: string[],
      files?: ChatFileAttachment[],
    ): Promise<void> => {
      if (!threadId) {
        setError('未选择线程，请先创建或选择一个线程');
        return;
      }

      if (!text.trim() && (!images || images.length === 0) && (!files || files.length === 0)) {
        return; // 空消息不发送
      }

      // 已有流在进行中
      if (isStreaming) {
        setError('请等待当前消息发送完成');
        return;
      }

      setError(undefined);

      // 构造用户消息
      const userMsg: ChatMessage = {
        id: uid(),
        role: 'user',
        content: text,
        images,
        files,
        timestamp: new Date(),
        isStreaming: false,
      };

      // 构造占位 assistant 消息
      const assistantMsg: ChatMessage = {
        id: uid(),
        role: 'assistant',
        content: '',
        timestamp: new Date(),
        isStreaming: true,
      };

      // 同步更新消息列表
      setMessages((prev) => [...prev, userMsg, assistantMsg]);
      setIsStreaming(true);

      try {
        // 通过 SSE 发送消息并接收流式回复
        const { stream, abort } = createSSEStream(
          `/api/v1/ag-ui/threads/${encodeURIComponent(threadId)}/runs`,
          {
            messages: [
              {
                role: 'user',
                content: text,
                ...(images && images.length > 0 ? { images } : {}),
                ...(files && files.length > 0
                  ? {
                      files: files.map((f) => ({
                        name: f.name,
                        mime_type: f.mimeType,
                        size_bytes: f.sizeBytes,
                        data: f.data,
                      })),
                    }
                  : {}),
              },
            ],
            stream: true,
          },
        );

        abortRef.current = abort;

        for await (const event of stream) {
          applySSEEvent(event, setMessages);
        }

        // 流正常结束 —— 标记 assistant 消息完成
        setMessages((prev) => {
          const updated = [...prev];
          const last = updated[updated.length - 1];
          if (last?.role === 'assistant') {
            updated[updated.length - 1] = { ...last, isStreaming: false };
          }
          return updated;
        });
      } catch (err: unknown) {
        const errMsg = err instanceof Error ? err.message : String(err);
        setError(errMsg);

        // 在 assistant 消息中追加错误提示
        setMessages((prev) => {
          const updated = [...prev];
          const last = updated[updated.length - 1];
          if (last?.role === 'assistant') {
            updated[updated.length - 1] = {
              ...last,
              content: last.content + `\n\n❌ ${errMsg}`,
              isStreaming: false,
            };
          }
          return updated;
        });
      } finally {
        setIsStreaming(false);
        abortRef.current = null;
      }
    },
    [threadId, isStreaming],
  );

  /* ---- 取消流 ---- */

  /**
   * 取消正在进行的流式传输。
   */
  const cancelStream = useCallback(() => {
    abortRef.current?.();
    abortRef.current = null;
    setIsStreaming(false);

    // 将未完成的 assistant 消息标记为完成
    setMessages((prev) => {
      const updated = [...prev];
      const last = updated[updated.length - 1];
      if (last?.isStreaming) {
        updated[updated.length - 1] = {
          ...last,
          content: last.content + '\n\n_（已取消）_',
          isStreaming: false,
        };
      }
      return updated;
    });
  }, []);

  /* ---- 清空消息 ---- */

  /**
   * 清空当前消息列表。
   */
  const clearMessages = useCallback(() => {
    setMessages([]);
    setError(undefined);
  }, []);

  /* ---- 图片上传 ---- */

  /**
   * 添加待上传图片（File 对象 → base64 编码）。
   *
   * @param file - 图片文件
   * @returns 图片 base64 数据
   */
  const addImage = useCallback(async (file: File): Promise<string> => {
    const base64 = await readFileAsBase64(file);
    const commaIndex = base64.indexOf(',');
    const rawData = commaIndex >= 0 ? base64.slice(commaIndex + 1) : base64;
    setImages((prev) => [...prev, rawData]);
    return rawData;
  }, []);

  /**
   * 移除待上传图片。
   *
   * @param data - 要移除的图片 base64 数据
   */
  const removeImage = useCallback((data: string) => {
    setImages((prev) => prev.filter((i) => i !== data));
  }, []);

  /**
   * 清空待上传图片列表。
   */
  const clearImages = useCallback(() => {
    setImages([]);
  }, []);

  /* ---- 消息加载 ---- */

  /**
   * 从后端重新加载当前线程的消息列表。
   * 若 threadId 为 null，则清空本地消息。
   */
  const loadMessages = useCallback(async () => {
    if (!threadId) {
      setMessages([]);
      return;
    }
    // NOTE: 实际项目中可从 apiClient.getThreadMessages 加载历史消息
    // 当前实现保持与旧版兼容但仅清空本地状态
    setMessages([]);
  }, [threadId]);

  return {
    /* 新 API（符合任务规格） */
    /** 消息列表 */
    messages,
    /** 是否正在流式传输 */
    isStreaming,
    /** 错误信息 */
    error,
    /** 发送消息（支持文本、图片、文件） */
    sendMessage,
    /** 取消当前流 */
    cancelStream,
    /** 清空消息列表 */
    clearMessages,

    /* 旧 API（向后兼容） */
    /** @deprecated 请使用 isStreaming */
    sending: isStreaming,
    /** @deprecated 请使用 cancelStream */
    cancel: cancelStream,
    /** 待上传的图片 base64 列表 */
    images,
    /** 上传图片文件 → base64 */
    addImage,
    /** 移除待上传图片 */
    removeImage,
    /** 清空待上传图片 */
    clearImages,
    /** 加载历史消息 */
    loadMessages,
  } as const;
}

/* ========== SSE 事件应用 ========== */

/**
 * 将 SSE 事件应用到消息列表中（不可变更新）。
 *
 * @param event      - SSE 事件
 * @param setMessages - React state setter
 */
function applySSEEvent(
  event: SSEEvent,
  setMessages: React.Dispatch<React.SetStateAction<ChatMessage[]>>,
): void {
  switch (event.type) {
    case 'token':
      // 追加 token 到最后一条 assistant 消息
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant') {
          updated[updated.length - 1] = {
            ...last,
            content: last.content + event.data.text,
          };
        }
        return updated;
      });
      break;

    case 'tool_call_start':
      // 添加工具调用状态到最后一条 assistant 消息
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant') {
          const newToolCall: ToolCallState = {
            id: event.data.id,
            name: event.data.name,
            arguments: event.data.arguments,
            status: 'running',
          };
          updated[updated.length - 1] = {
            ...last,
            toolCalls: [...(last.toolCalls ?? []), newToolCall],
          };
        }
        return updated;
      });
      break;

    case 'tool_call_end':
      // 更新对应工具调用的结果与状态
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant' && last.toolCalls) {
          updated[updated.length - 1] = {
            ...last,
            toolCalls: last.toolCalls.map((tc) =>
              tc.id === event.data.id
                ? { ...tc, result: event.data.result, status: 'completed' as const }
                : tc,
            ),
          };
        }
        return updated;
      });
      break;

    case 'message_end': {
      // 最终消息 —— 更新 role 和完整 content，标记完成
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant') {
          updated[updated.length - 1] = {
            ...last,
            role: event.data.role as ChatMessage['role'],
            content: event.data.content || last.content,
            isStreaming: false,
          };
        }
        return updated;
      });
      break;
    }

    case 'error':
      // 错误事件 —— 设置错误信息并标记 assistant 消息
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant') {
          updated[updated.length - 1] = {
            ...last,
            content: last.content + `\n\n❌ ${event.data.message}`,
            isStreaming: false,
          };
        }
        return updated;
      });
      break;

    case 'done':
      // 完成事件 —— 标记 assistant 消息完成
      setMessages((prev) => {
        const updated = [...prev];
        const last = updated[updated.length - 1];
        if (last?.role === 'assistant') {
          updated[updated.length - 1] = { ...last, isStreaming: false };
        }
        return updated;
      });
      break;

    case 'unknown':
      // 未知事件 —— 忽略
      break;
  }
}
