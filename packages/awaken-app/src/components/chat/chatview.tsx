import { PanelLeft, Bot } from 'lucide-react';
import type { Thread } from '../../types/api';
import { useChat } from '../../hooks/use-chat';
import { ThreadList } from './ThreadList';
import { MessageList } from './MessageList';
import { ChatInput } from './ChatInput';

interface ChatViewProps {
  threads: Thread[];
  currentThreadId: string | null;
  onSelectThread: (id: string) => void;
  onNewThread: () => void;
  onDeleteThread: (id: string) => void;
  loading?: boolean;
}

export function ChatView({
  threads,
  currentThreadId,
  onSelectThread,
  onNewThread,
  onDeleteThread,
  loading,
}: ChatViewProps) {
  const {
    messages,
    isStreaming,
    error,
    sendMessage,
    cancelStream,
  } = useChat(currentThreadId);

  // 当前线程标题
  const threadTitle = currentThreadId
    ? threads.find((t) => t.thread_id === currentThreadId)?.title || '对话中...'
    : 'Remo AI Agent';

  return (
    <div className="flex-1 flex h-full overflow-hidden bg-gray-50 dark:bg-gray-950">
      {/* ===== 左侧边栏 ===== */}
      <div className="flex-shrink-0 w-72 border-r border-gray-200/60 dark:border-gray-800/60">
        <div className="w-72 h-full glass rounded-r-2xl">
          <ThreadList
            threads={threads}
            currentThreadId={currentThreadId}
            onSelect={onSelectThread}
            onNew={onNewThread}
            onDelete={onDeleteThread}
            loading={loading}
          />
        </div>
      </div>

      {/* ===== 主聊天区域 ===== */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* 顶栏 */}
        <div className="flex items-center gap-3 px-4 h-14 border-b border-gray-200/60 dark:border-gray-800/60 glass flex-shrink-0">
          <div className="flex items-center gap-2 flex-1 min-w-0">
            <Bot className="w-5 h-5 text-indigo-400 flex-shrink-0" />
            <h1 className="text-sm font-semibold text-gray-700 dark:text-gray-300 truncate">
              {threadTitle}
            </h1>
          </div>

          {/* 错误提示 */}
          {error && (
            <span className="text-xs text-red-400 bg-red-50/50 dark:bg-red-950/30 px-2 py-1 rounded-lg">
              {error}
            </span>
          )}

          {/* 状态指示 */}
          {isStreaming && (
            <div className="flex items-center gap-1.5 text-xs text-indigo-500">
              <span className="w-2 h-2 rounded-full bg-indigo-400 animate-pulse" />
              响应中
            </div>
          )}
        </div>

        {/* 消息列表（flex-1 自动填满） */}
        <MessageList messages={messages} streaming={isStreaming} />

        {/* 输入区域（底部固定） */}
        <ChatInput
          onSend={sendMessage}
          onCancel={cancelStream}
          isStreaming={isStreaming}
          disabled={!currentThreadId}
        />
      </div>

      {/* ===== 右侧工具面板（预留） ===== */}
      <div className="hidden lg:flex flex-shrink-0 w-72 border-l border-gray-200/60 dark:border-gray-800/60">
        <div className="flex-1 glass rounded-l-2xl p-4">
          <div className="text-center py-12">
            <div className="w-12 h-12 mx-auto mb-3 rounded-xl bg-gradient-to-br from-indigo-500/10 to-purple-500/10 flex items-center justify-center">
              <Bot className="w-6 h-6 text-indigo-400/50" />
            </div>
            <p className="text-xs text-gray-400 dark:text-gray-500">工具面板</p>
            <p className="text-[10px] text-gray-300 dark:text-gray-600 mt-1">扩展能力即将上线</p>
          </div>
        </div>
      </div>
    </div>
  );
}
