import { useState, useCallback } from 'react';
import { useThreads } from '../hooks/use-threads';
import { ChatView } from '../components/chat/ChatView';
import { MessageSquarePlus } from 'lucide-react';

export function ChatPage() {
  const { threads, loading, addThread, removeThread } = useThreads();
  const [currentThreadId, setCurrentThreadId] = useState<string | null>(null);

  const handleNewThread = useCallback(async () => {
    try {
      const t = await addThread();
      setCurrentThreadId(t.id);
    } catch (e) {
      console.error('新建对话失败', e);
    }
  }, [addThread]);

  const handleSelectThread = useCallback((id: string) => {
    setCurrentThreadId(id);
  }, []);

  const handleDeleteThread = useCallback(
    async (id: string) => {
      try {
        await removeThread(id);
        if (currentThreadId === id) setCurrentThreadId(null);
      } catch (e) {
        console.error('删除对话失败', e);
      }
    },
    [removeThread, currentThreadId],
  );

  return (
    <div className="h-full flex flex-col">
      {/* 未选择线程时显示欢迎屏 */}
      {!currentThreadId && !loading && threads.length === 0 ? (
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center max-w-md px-6">
            <div className="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-gradient-to-br from-remo-400/20 to-remo-500/10 border border-remo-400/20 mb-6">
              <MessageSquarePlus className="w-8 h-8 text-remo-400" />
            </div>
            <h2 className="text-xl font-semibold text-gray-200 mb-2">
              选择一个对话或创建新对话
            </h2>
            <p className="text-sm text-gray-400 leading-relaxed">
              从左侧列表选择一个已有对话，或点击「新建对话」开始新的 AI 之旅
            </p>
            <button
              onClick={handleNewThread}
              className="mt-6 inline-flex items-center gap-2 px-5 py-2.5 rounded-xl text-sm font-medium
                bg-gradient-to-r from-remo-500 to-remo-400 text-white shadow-lg shadow-remo-500/20
                hover:from-remo-600 hover:to-remo-500 active:scale-[0.98]
                transition-all duration-200"
            >
              <MessageSquarePlus className="w-4 h-4" />
              新建对话
            </button>
          </div>
        </div>
      ) : (
        <ChatView
          threads={threads}
          currentThreadId={currentThreadId}
          onSelectThread={handleSelectThread}
          onNewThread={handleNewThread}
          onDeleteThread={handleDeleteThread}
          loading={loading}
        />
      )}
    </div>
  );
}
