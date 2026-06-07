/* ===================================================================
 * 前端聊天状态类型
 *
 * 涵盖聊天消息、工具调用状态、文件附件，以及 reducer 动作。
 * =================================================================== */

import type { FileAttachment as ApiFileAttachment } from './api';

/* ---- 聊天消息 ---- */

/** 前端聊天消息 */
export interface ChatMessage {
  /** 消息唯一 ID */
  id: string;
  /** 消息角色 */
  role: 'user' | 'assistant' | 'system' | 'tool';
  /** 消息文本内容 */
  content: string;
  /** Base64 图片数据列表（可选） */
  images?: string[];
  /** 文件附件列表（可选） */
  files?: FileAttachment[];
  /** 工具调用状态列表（可选） */
  toolCalls?: ToolCallState[];
  /** 消息时间戳 */
  timestamp: Date;
  /** 是否正在流式传输中（可选） */
  isStreaming?: boolean;
}

/* ---- 工具调用状态 ---- */

/** 工具调用状态（前端展示用） */
export interface ToolCallState {
  /** 工具调用 ID */
  id: string;
  /** 工具名称 */
  name: string;
  /** 工具参数 */
  arguments: Record<string, unknown>;
  /** 工具执行结果（可选） */
  result?: unknown;
  /** 调用状态 */
  status: 'pending' | 'running' | 'completed' | 'failed';
}

/* ---- 文件附件（前端版） ---- */

/** 文件附件（前端展示用，含预览 URL） */
export interface FileAttachment {
  /** 文件唯一 ID */
  id: string;
  /** 文件名 */
  name: string;
  /** MIME 类型 */
  mimeType: string;
  /** 文件大小（字节） */
  sizeBytes: number;
  /** Base64 编码的文件数据 */
  data: string;
  /** 本地预览 URL（可选，用于 Blob URL） */
  previewUrl?: string;
}

/* ---- 聊天状态 ---- */

/** 聊天状态 */
export interface ChatState {
  /** 消息列表 */
  messages: ChatMessage[];
  /** 是否正在流式传输 */
  isStreaming: boolean;
  /** 错误信息（可选） */
  error?: string;
}

/* ---- Reducer 动作 ---- */

/** 聊天状态 reducer 动作 */
export type ChatAction =
  | { type: 'ADD_MESSAGE'; payload: ChatMessage }
  | { type: 'UPDATE_LAST_MESSAGE'; payload: Partial<ChatMessage> }
  | { type: 'SET_STREAMING'; payload: boolean }
  | { type: 'SET_ERROR'; payload: string | undefined }
  | { type: 'CLEAR_MESSAGES' }
  | { type: 'APPEND_TOKEN'; payload: string };

/* ---- 工具函数 ---- */

/**
 * 将 API 端的 FileAttachment（snake_case）转换为前端版（camelCase）。
 *
 * @param api - API 返回的文件附件
 * @param id  - 前端唯一标识
 * @returns 前端 FileAttachment
 */
export function toChatFileAttachment(api: ApiFileAttachment, id: string): FileAttachment {
  return {
    id,
    name: api.name,
    mimeType: api.mime_type,
    sizeBytes: api.size_bytes,
    data: api.data,
    previewUrl: api.mime_type.startsWith('image/')
      ? `data:${api.mime_type};base64,${api.data}`
      : undefined,
  };
}

/**
 * 将前端 FileAttachment（camelCase）转换为 API 版（snake_case）。
 *
 * @param chat - 前端的文件附件
 * @returns API 端的 FileAttachment
 */
export function toApiFileAttachment(chat: FileAttachment): ApiFileAttachment {
  return {
    name: chat.name,
    mime_type: chat.mimeType,
    size_bytes: chat.sizeBytes,
    data: chat.data,
  };
}
