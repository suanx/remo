import { useState } from 'react';
import { ChevronDown, Check, Loader2, AlertTriangle } from 'lucide-react';
import type { ToolCallState } from '../../types/chat';

interface ToolCallCardProps {
  toolCall: ToolCallState;
  index: number;
}

const STATUS_ICONS = {
  pending: Loader2,
  running: Loader2,
  completed: Check,
  failed: AlertTriangle,
} as const;

const STATUS_COLORS = {
  pending: 'text-yellow-500',
  running: 'text-indigo-400',
  completed: 'text-green-500',
  failed: 'text-red-500',
} as const;

const STATUS_LABELS = {
  pending: '等待中',
  running: '运行中...',
  completed: '完成',
  failed: '失败',
} as const;

function truncate(str: string, max = 200): string {
  if (str.length <= max) return str;
  return str.slice(0, max) + '...';
}

export function ToolCallCard({ toolCall, index }: ToolCallCardProps) {
  const [expanded, setExpanded] = useState(false);
  const StatusIcon = STATUS_ICONS[toolCall.status] ?? Loader2;
  const isRunning = toolCall.status === 'running';

  const argsJSON = JSON.stringify(toolCall.arguments, null, 2);
  const resultStr =
    toolCall.result !== undefined
      ? typeof toolCall.result === 'string'
        ? toolCall.result
        : JSON.stringify(toolCall.result, null, 2)
      : undefined;

  return (
    <div
      className="group rounded-xl border border-gray-200/50 dark:border-gray-700/30 bg-white/40 dark:bg-gray-800/30 backdrop-blur-sm overflow-hidden transition-all duration-200 animate-fade-in"
      style={{ animationDelay: `${index * 60}ms` }}
    >
      {/* 头部 */}
      <button
        onClick={() => setExpanded((v) => !v)}
        className="w-full flex items-center gap-2.5 px-3.5 py-2.5 text-left hover:bg-white/30 dark:hover:bg-gray-800/30 transition-colors"
      >
        <StatusIcon
          className={`w-4 h-4 flex-shrink-0 ${STATUS_COLORS[toolCall.status]} ${isRunning ? 'animate-spin' : ''}`}
        />
        <span className="flex-1 min-w-0">
          <span className="text-xs font-mono font-semibold text-gray-700 dark:text-gray-300 truncate block">
            {toolCall.name}
          </span>
          <span className={`text-[10px] ${STATUS_COLORS[toolCall.status]}`}>
            {STATUS_LABELS[toolCall.status]}
          </span>
        </span>
        <ChevronDown
          className={`w-3.5 h-3.5 text-gray-400 transition-transform duration-200 ${expanded ? 'rotate-180' : ''}`}
        />
      </button>

      {/* 展开内容 */}
      {expanded && (
        <div className="px-3.5 pb-3 space-y-2 animate-fade-in">
          {/* 参数 */}
          <div>
            <p className="text-[10px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1">
              参数
            </p>
            <pre className="text-[11px] font-mono bg-white/40 dark:bg-gray-950/40 rounded-lg p-2 overflow-x-auto text-gray-600 dark:text-gray-400 border border-gray-100/50 dark:border-gray-800/50 max-h-32 overflow-y-auto">
              {argsJSON}
            </pre>
          </div>

          {/* 结果 */}
          {resultStr !== undefined && (
            <div>
              <p className="text-[10px] font-medium text-gray-400 dark:text-gray-500 uppercase tracking-wider mb-1">
                结果
              </p>
              <pre className="text-[11px] font-mono bg-white/40 dark:bg-gray-950/40 rounded-lg p-2 overflow-x-auto text-gray-600 dark:text-gray-400 border border-gray-100/50 dark:border-gray-800/50 max-h-32 overflow-y-auto">
                {truncate(resultStr)}
              </pre>
            </div>
          )}

          {/* 运行中指示 */}
          {isRunning && (
            <div className="flex items-center gap-2 text-[10px] text-indigo-500">
              <span className="w-1.5 h-1.5 rounded-full bg-indigo-400 animate-pulse" />
              等待结果...
            </div>
          )}
        </div>
      )}
    </div>
  );
}
