import { useRef, useEffect, useState, useMemo } from 'react';
import { Plus, Search, Trash2, MessageSquare } from 'lucide-react';
import type { Thread } from '../../types/api';

interface ThreadListProps {
  threads: Thread[];
  currentThreadId: string | null;
  onSelect: (id: string) => void;
  onNew: () => void;
  onDelete: (id: string) => void;
  loading?: boolean;
}

export function ThreadList({
  threads,
  currentThreadId,
  onSelect,
  onNew,
  onDelete,
  loading,
}: ThreadListProps) {
  const [search, setSearch] = useState('');
  const activeRef = useRef<HTMLButtonElement>(null);

  // 自动滚动当前线程到可见区域
  useEffect(() => {
    activeRef.current?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  }, [currentThreadId]);

  // 前端过滤
  const filtered = useMemo(() => {
    if (!search.trim()) return threads;
    const q = search.toLowerCase();
    return threads.filter((t) => {
      const title = t.title || '';
      return title.toLowerCase().includes(q) || t.thread_id.toLowerCase().includes(q);
    });
  }, [threads, search]);

  return (
    <div className="flex flex-col h-full">
      {/* 头部 */}
      <div className="flex items-center justify-between px-4 py-3 border-b border-white/10 dark:border-gray-700/30">
        <h2 className="text-sm font-semibold text-gray-700 dark:text-gray-300 tracking-tight">
          对话
        </h2>
        <span className="text-[11px] text-gray-400 dark:text-gray-500 bg-white/30 dark:bg-gray-800/30 px-2 py-0.5 rounded-full">
          {threads.length}
        </span>
      </div>

      {/* 新建按钮 */}
      <div className="px-3 pt-3 pb-2">
        <button
          onClick={onNew}
          disabled={loading}
          className="w-full flex items-center justify-center gap-2 px-3 py-2.5 rounded-xl text-sm font-medium
            bg-gradient-to-r from-indigo-500 to-purple-500 text-white shadow-lg shadow-indigo-500/20
            hover:from-indigo-600 hover:to-purple-600 active:scale-[0.98]
            disabled:opacity-50 disabled:cursor-not-allowed
            transition-all duration-200"
        >
          <Plus className="w-4 h-4" />
          新建对话
        </button>
      </div>

      {/* 搜索框 */}
      <div className="px-3 pb-2">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400 dark:text-gray-500" />
          <input
            type="text"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="搜索对话..."
            className="w-full pl-8 pr-3 py-1.5 text-xs rounded-lg bg-white/30 dark:bg-gray-800/30 border border-white/20 dark:border-gray-700/30 text-gray-700 dark:text-gray-300 placeholder:text-gray-400 dark:placeholder:text-gray-500 outline-none focus:ring-1 focus:ring-indigo-500/40 focus:border-indigo-500/30 transition-all"
          />
        </div>
      </div>

      {/* 列表 */}
      <div className="flex-1 overflow-y-auto px-3 pb-3 space-y-1">
        {loading && threads.length === 0 && (
          <div className="flex items-center justify-center py-12">
            <div className="flex gap-1">
              <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '0ms' }} />
              <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '200ms' }} />
              <span className="w-2 h-2 rounded-full bg-indigo-400/40 animate-bounce" style={{ animationDelay: '400ms' }} />
            </div>
          </div>
        )}

        {!loading && filtered.length === 0 && (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <MessageSquare className="w-8 h-8 text-gray-300 dark:text-gray-600 mb-2" />
            <p className="text-xs text-gray-400 dark:text-gray-500">暂无对话</p>
            <p className="text-[10px] text-gray-300 dark:text-gray-600 mt-1">
              {search ? '尝试其他关键词' : '点击上方按钮新建'}
            </p>
          </div>
        )}

        {filtered.map((thread) => {
          const isActive = thread.thread_id === currentThreadId;
          const createdDate = thread.created_at ? new Date(thread.created_at) : null;
          const displayName = thread.title || `对话 ${thread.thread_id.slice(0, 8)}...`;

          return (
            <button
              key={thread.thread_id}
              ref={isActive ? activeRef : undefined}
              onClick={() => onSelect(thread.thread_id)}
              className={`
                w-full text-left px-3 py-2.5 rounded-xl transition-all duration-200 group relative
                ${isActive
                  ? 'glass shadow-md ring-1 ring-indigo-500/20'
                  : 'hover:bg-white/40 dark:hover:bg-gray-800/40'
                }
              `}
            >
              <div className="flex items-start gap-2.5">
                <div className={`mt-0.5 flex-shrink-0 w-8 h-8 rounded-lg flex items-center justify-center ${
                  isActive
                    ? 'bg-gradient-to-br from-indigo-500 to-purple-600 text-white'
                    : 'bg-gray-100 dark:bg-gray-800 text-gray-400 dark:text-gray-500'
                }`}>
                  <MessageSquare className="w-4 h-4" />
                </div>
                <div className="flex-1 min-w-0">
                  <p
                    className={`text-sm truncate ${
                      isActive
                        ? 'font-semibold text-gray-800 dark:text-gray-100'
                        : 'font-medium text-gray-600 dark:text-gray-400'
                    }`}
                  >
                    {displayName}
                  </p>
                  <div className="flex items-center gap-2 mt-0.5">
                    {createdDate && (
                      <p className="text-[10px] text-gray-400 dark:text-gray-500">
                        {createdDate.toLocaleDateString('zh-CN', {
                          month: 'short',
                          day: 'numeric',
                          hour: '2-digit',
                          minute: '2-digit',
                        })}
                      </p>
                    )}
                    {thread.message_count > 0 && (
                      <span className="text-[10px] text-gray-400 dark:text-gray-500">
                        {thread.message_count} 条消息
                      </span>
                    )}
                  </div>
                </div>

                {/* 删除按钮 */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDelete(thread.thread_id);
                  }}
                  className="opacity-0 group-hover:opacity-100 transition-opacity duration-200 p-1 rounded-lg hover:bg-red-100 dark:hover:bg-red-900/30 text-gray-400 hover:text-red-500"
                  title="删除"
                >
                  <Trash2 className="w-3.5 h-3.5" />
                </button>
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}
