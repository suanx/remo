import { useState, useEffect, useCallback } from 'react';
import {
  RefreshCw,
  PlayCircle,
  Activity,
  Zap,
  Calendar,
  ToggleLeft,
  ToggleRight,
} from 'lucide-react';
import { listRuns, listThreads } from '../../lib/api-client';
import type { RunRecord, Thread } from '../../types/api';
import { StatsCard } from './StatsCard';
import type { StatsCardColor } from './StatsCard';
import { HealthCard } from './HealthCard';
import { RunList } from './RunList';

/* ===== DashboardView ===== */
export function DashboardView() {
  const [runs, setRuns] = useState<RunRecord[]>([]);
  const [threads, setThreads] = useState<Thread[]>([]);
  const [loading, setLoading] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(true);

  /* ---- 数据抓取 ---- */
  const fetchData = useCallback(async () => {
    try {
      const [r, t] = await Promise.all([listRuns(), listThreads()]);
      setRuns(r);
      setThreads(t);
    } catch {
      // 静默失败
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  /* 自动刷新（每 15s） */
  useEffect(() => {
    if (!autoRefresh) return;
    const interval = setInterval(fetchData, 15_000);
    return () => clearInterval(interval);
  }, [autoRefresh, fetchData]);

  /* ---- 派生统计 ---- */
  const totalRuns = runs.length;
  const activeThreads = threads.length;
  const tokenConsumption = '—'; // API 暂未提供
  const todayRuns = runs.filter((r) => {
    const d = new Date(r.started_at);
    const now = new Date();
    return d.toDateString() === now.toDateString();
  }).length;

  /* ---- 统计卡片配置 ---- */
  const statsCards: Array<{
    label: string;
    value: string | number;
    icon: typeof PlayCircle;
    color: StatsCardColor;
    trend?: string;
  }> = [
    {
      label: '总运行次数',
      value: loading ? '…' : totalRuns,
      icon: PlayCircle,
      color: 'green',
      trend: totalRuns > 0 ? `共 ${totalRuns} 次运行` : undefined,
    },
    {
      label: '活跃线程数',
      value: loading ? '…' : activeThreads,
      icon: Activity,
      color: 'blue',
      trend: activeThreads > 0 ? `${activeThreads} 个线程` : undefined,
    },
    {
      label: 'Token 消耗',
      value: tokenConsumption,
      icon: Zap,
      color: 'purple',
    },
    {
      label: '今日运行',
      value: loading ? '…' : todayRuns,
      icon: Calendar,
      color: 'orange',
      trend: todayRuns > 0 ? `今日 ${todayRuns} 次` : undefined,
    },
  ];

  return (
    <div className="flex-1 overflow-y-auto p-6 lg:p-8 space-y-6">
      {/* ===== 顶栏 ===== */}
      <div className="flex items-center justify-between flex-wrap gap-4">
        <div>
          <h1 className="text-xl font-bold text-gray-100 tracking-tight">
            仪表盘
          </h1>
          <p className="text-sm text-gray-500 mt-1">
            Remo AI Agent 运行状态概览
          </p>
        </div>

        <div className="flex items-center gap-3">
          {/* 自动刷新开关 */}
          <button
            onClick={() => setAutoRefresh((v) => !v)}
            className={`
              flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium
              transition-all duration-200 border
              ${
                autoRefresh
                  ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/15'
                  : 'bg-gray-800/50 text-gray-500 border-white/[0.06] hover:text-gray-300'
              }
            `}
          >
            {autoRefresh ? (
              <ToggleRight className="w-3.5 h-3.5" />
            ) : (
              <ToggleLeft className="w-3.5 h-3.5" />
            )}
            <span>自动刷新</span>
          </button>

          {/* 刷新按钮 */}
          <button
            onClick={fetchData}
            disabled={loading}
            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium bg-white/[0.06] text-gray-400 border border-white/[0.06] hover:bg-white/[0.1] hover:text-gray-200 transition-all duration-200 disabled:opacity-50"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${loading ? 'animate-spin' : ''}`} />
            <span>刷新</span>
          </button>
        </div>
      </div>

      {/* ===== 统计卡片 2×2 网格 ===== */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
        {statsCards.map((card) => (
          <StatsCard
            key={card.label}
            label={card.label}
            value={card.value}
            icon={card.icon}
            color={card.color}
            trend={card.trend}
          />
        ))}
      </div>

      {/* ===== 健康状态 + 运行记录 ===== */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <HealthCard />
        <RunList />
      </div>

      {/* ===== 底部提示 ===== */}
      <div className="rounded-2xl border border-white/[0.06] bg-gradient-to-br from-gray-900/60 to-gray-900/40 p-5 border-l-[3px] border-l-amber-500/50">
        <div className="flex items-start gap-3">
          <span className="text-lg shrink-0 mt-0.5 opacity-60">💡</span>
          <div>
            <h3 className="text-sm font-medium text-gray-300">
              通过 API 配置 Agent
            </h3>
            <p className="text-xs text-gray-500 mt-1 leading-relaxed">
              Remo 启动时为空运行。使用{' '}
              <code className="bg-amber-500/10 text-amber-400 px-1 rounded text-[10px] font-mono">
                POST /v1/agents
              </code>{' '}
              动态注册 Agent。
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
