import { useRef, useEffect, useState } from 'react';
import { Bot, Loader2 } from 'lucide-react';
import type { ChatMessage } from '../../types/chat';
import { ToolCallCard } from './ToolCallCard';
import { ImagePreviewModal } from './ImagePreview';

interface MessageListProps {
  messages: ChatMessage[];
  streaming?: boolean;
}

/* ===== 打字光标 ===== */
function TypewriterCursor() {
  return (
    <span className="inline-block w-[2px] h-[1em] bg-indigo-400 dark:bg-indigo-300 ml-0.5 align-middle animate-pulse" />
  );
}

/* ===== 消息中的图片显示 ===== */
function InlineImages({ images }: { images: string[] }) {
  const [previewSrc, setPreviewSrc] = useState<string | null>(null);

  return (
    <>
      <div className="flex flex-wrap gap-2 mb-2">
        {images.map((b64, idx) => {
          // 判断是否已带 data: 前缀
          const src = b64.startsWith('data:') ? b64 : `data:image/jpeg;base64,${b64}`;
          return (
            <img
              key={idx}
              src={src}
              alt={`图片 ${idx + 1}`}
              className="max-h-40 rounded-lg object-cover cursor-pointer border border-white/10 hover:opacity-90 transition-opacity"
              onClick={() => setPreviewSrc(src)}
            />
          );
        })}
      </div>
      {previewSrc && (
        <ImagePreviewModal
          src={previewSrc}
          onClose={() => setPreviewSrc(null)}
        />
      )}
    </>
  );
}

/* ===== 单个消息气泡 ===== */
function MessageBubble({ message }: { message: ChatMessage }) {
  const isUser = message.role === 'user';
  const isAssistant = message.role === 'assistant';
  const isSystem = message.role === 'system';
  const isTool = message.role === 'tool';
  const isStreamingMsg = message.isStreaming;

  // System: 居中灰色小字
  if (isSystem) {
    return (
      <div className="flex justify-center py-2 animate-fade-in">
        <span className="text-xs text-gray-400 dark:text-gray-500 bg-gray-100/50 dark:bg-gray-800/30 px-3 py-1 rounded-full backdrop-blur-sm">
          {message.content}
        </span>
      </div>
    );
  }

  // Tool: 可折叠卡片
  if (isTool) {
    return (
      <div className="flex justify-center py-1 animate-fade-in">
        {message.toolCalls?.map((tc, idx) => (
          <ToolCallCard key={tc.id} toolCall={tc} index={idx} />
        ))}
      </div>
    );
  }

  return (
    <div
      className={`flex gap-3 animate-fade-in ${isUser ? 'flex-row-reverse' : ''}`}
    >
      {/* 头像 */}
      <div
        className={`
          flex-shrink-0 w-8 h-8 rounded-xl flex items-center justify-center
          ${isUser
            ? 'bg-gradient-to-br from-indigo-500 to-blue-600 text-white shadow-lg shadow-indigo-500/20'
            : 'bg-gradient-to-br from-purple-500 to-indigo-600 text-white shadow-lg shadow-purple-500/20'
          }
        `}
      >
        {isUser ? (
          <span className="text-sm">🧑</span>
        ) : (
          <Bot className="w-4 h-4" />
        )}
      </div>

      {/* 气泡 */}
      <div className={`flex flex-col max-w-[80%] ${isUser ? 'items-end' : 'items-start'}`}>
        {/* 角色标签 */}
        <span className="text-[10px] text-gray-400 dark:text-gray-500 mb-1 px-1">
          {isUser ? '你' : 'Remo AI'}
        </span>

        <div
          className={`
            rounded-2xl px-4 py-3 text-sm leading-relaxed shadow-sm
            ${isUser
              ? 'bg-gradient-to-br from-indigo-500 to-blue-600 text-white rounded-tr-md'
              : 'glass rounded-tl-md text-gray-800 dark:text-gray-200 border border-white/10 dark:border-white/5'
            }
          `}
        >
          {/* 图片（仅用户消息可能带图片） */}
          {isUser && message.images && message.images.length > 0 && (
            <InlineImages images={message.images} />
          )}

          {/* 文本内容 */}
          <div className="whitespace-pre-wrap break-words">
            {message.content || (isStreamingMsg ? '' : '(空)')}
            {isStreamingMsg && <TypewriterCursor />}
          </div>

          {/* 工具调用卡片（assistant 消息） */}
          {isAssistant && message.toolCalls && message.toolCalls.length > 0 && (
            <div className="mt-3 space-y-2 border-t border-gray-200/30 dark:border-gray-600/30 pt-3">
              {message.toolCalls.map((tc, idx) => (
                <ToolCallCard key={tc.id} toolCall={tc} index={idx} />
              ))}
            </div>
          )}
        </div>

        {/* 时间戳 */}
        <span className="text-[9px] text-gray-400 dark:text-gray-600 mt-1 px-1 opacity-0 group-hover:opacity-100 transition-opacity">
          {new Date(message.timestamp).toLocaleTimeString('zh-CN', {
            hour: '2-digit',
            minute: '2-digit',
          })}
        </span>
      </div>
    </div>
  );
}

/* ===== 消息列表 ===== */
export function MessageList({ messages, streaming }: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  // 自动滚动到底部
  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth', block: 'end' });
  }, [messages, messages.length]);

  // 空状态
  if (messages.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center">
        <div className="text-center max-w-md px-8 animate-fade-in">
          <div className="w-16 h-16 mx-auto mb-4 rounded-2xl bg-gradient-to-br from-indigo-500/20 to-purple-500/20 flex items-center justify-center">
            <Bot className="w-8 h-8 text-indigo-400" />
          </div>
          <h2 className="text-lg font-semibold text-gray-700 dark:text-gray-300 mb-2">
            开始与 Remo AI 对话...
          </h2>
          <p className="text-sm text-gray-400 dark:text-gray-500 leading-relaxed">
            选择左侧对话或新建一个，开始与 AI 助手交流。
            <br />
            支持文字、图片和工具调用。
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex-1 overflow-y-auto px-4 py-6 space-y-4">
      {messages.map((msg) => (
        <div key={msg.id} className="group">
          <MessageBubble message={msg} />
        </div>
      ))}

      {/* 流式加载指示 */}
      {streaming && messages.length > 0 && (
        <div className="flex items-center gap-2 text-xs text-gray-400 dark:text-gray-500 pl-2 animate-fade-in">
          <Loader2 className="w-3.5 h-3.5 text-indigo-400 animate-spin" />
          <span>AI 正在思考...</span>
        </div>
      )}

      <div ref={bottomRef} />
    </div>
  );
}
