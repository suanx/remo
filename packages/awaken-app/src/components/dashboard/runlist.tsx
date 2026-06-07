import { useState, useEffect } from 'react';
import type { RunRecord } from '../../types/api';
import { listRuns } from '../../lib/api-client';
import { Eye, RefreshCw } from 'lucide-react';

/* ===== 状态 → 徽章样式 ===== */
const STATUS_BADGE: Record<string, string> = {
  running:    'bg-sky-500/15 text-sky-400 border-sky-500/20',
  completed:  'bg-emerald-500/15 text-emerald-400 border-emerald-500/20',
  failed:     'bg-red-500/15 text-red-400 border-red-500/20',
  suspended:  'bg-amber-500/15 text-amber-400 border-amber-500/20',
  cancelled:  'bg-gray-500/15 text-gray-400 border-gray-500/20',
  pending:    'bg-gray-500/15 text-gray-400 border-gray-500/20',
};

/** 截断 ID 显示 */
function truncateId(id: string, len = 12): string {
  return id.length > len ? `${id.slice(0, len)}…` : id;
}

/** 计算用时（秒） */
function calcDuration(run: RunRecord): string {
  if (run.finished_at && run.started_at) {
    const ms = new Date(run.finished_at).getTime() - new Date(run.started_at).getTime();
    return `${(ms / 1000).toFixed(1)}s`;
  }
  return '—';
}

/* ===== RunList 组件 ===== */
export function RunList() {
  const [runs, setRuns] = useState<RunRecord[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetch = async () => {
      try {
        setRuns(await listRuns());
      } catch {
        // 静默失败
      } finally {
        setLoading(false);
      }
    };
    fetch();
    const interval = setInterval(fetch, 5_000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="rounded-2xl border border-white/[0.06] bg-gradient-to-br from-gray-900 to-gray-900/80 p-5 transition-all duration-300 hover:border-white/[0.12]">
      {/* 标题栏 */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-gray-300">运行记录</h3>
        <RefreshCw
          className={`w-4 h-4 text-gray-500 ${loading ? 'animate-spin' : ''}`}
        />
      </div>

      {/* 表格 */}
      {loading && runs.length === 0 ? (
        <div className="space-y-3">
          {[1, 2, 3].map((i) => (
            <div key={i} className="h-9 bg-white/[0.04] rounded-lg animate-pulse" />
          ))}
        </div>
      ) : runs.length === 0 ? (
        <div className="flex flex-col items-center justify-center py-10 text-gray-500">
          <span className="text-2xl mb-2 opacity-30">🏃</span>
          <p className="text-sm">暂无运行记录</p>
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-white/[0.06] text-xs text-gray-500 uppercase tracking-wider">
                <th className="text-left font-medium px-2 py-2">Run ID</th>
                <th className="text-left font-medium px-2 py-2">Agent</th>
                <th className="text-left font-medium px-2 py-2">状态</th>
                <th className="text-right font-medium px-2 py-2">消息</th>
                <th className="text-right font-medium px-2 py-2">工具调用</th>
                <th className="text-right font-medium px-2 py-2">用时</th>
                <th className="text-right font-medium px-2 py-2">操作</th>
              </tr>
            </thead>
            <tbody>
              {runs.slice(0, 30).map((run) => (
                <tr
                  key={run.run_id}
                  className="border-b border-white/[0.04] hover:bg-white/[0.02] transition-colors"
                >
                  {/* Run ID */}
                  <td className="px-2 py-2.5 font-mono text-xs text-gray-400" title={run.run_id}>
                    {truncateId(run.run_id)}
                  </td>

                  {/* Agent ID */}
                  <td className="px-2 py-2.5 text-gray-300 max-w-[100px] truncate" title={run.agent_id}>
                    {run.agent_id}
                  </td>

                  {/* 状态 Badge */}
                  <td className="px-2 py-2.5">
                    <span
                      className={`
                        inline-block px-2 py-0.5 rounded-full text-[10px] font-medium
                        border ${STATUS_BADGE[run.status] || STATUS_BADGE.pending}
                      `}
                    >
                      {statusLabel(run.status)}
                    </span>
                  </td>

                  {/* 消息数 */}
                  <td className="px-2 py-2.5 text-right font-mono text-xs text-gray-400">
                    {run.message_count ?? '—'}
                  </td>

                  {/* 工具调用数 */}
                  <td className="px-2 py-2.5 text-right font-mono text-xs text-gray-400">
                    {run.tool_call_count ?? '—'}
                  </td>

                  {/* 用时 */}
                  <td className="px-2 py-2.5 text-right font-mono text-xs text-gray-400">
                    {calcDuration(run)}
                  </td>

                  {/* 操作 */}
                  <td className="px-2 py-2.5 text-right">
                    <button
                      className="inline-flex items-center gap-1 px-2 py-1 rounded-lg text-[10px] font-medium text-gray-500 hover:text-gray-200 hover:bg-white/[0.06] transition-colors"
                      title="查看详情"
                    >
                      <Eye className="w-3.5 h-3.5" />
                      <span className="hidden sm:inline">详情</span>
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

/** 中文本地化状态 */
function statusLabel(status: string): string {
  const map: Record<string, string> = {
    running: '运行中',
    completed: '已完成',
    failed: '失败',
    suspended: '已暂停',
    cancelled: '已取消',
    pending: '等待中',
  };
  return map[status] ?? status;
}
