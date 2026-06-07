import { useState, useEffect } from 'react';
import type { HealthResponse } from '../../types/api';
import { getHealth } from '../../lib/api-client';
import { Activity, Clock, Cpu } from 'lucide-react';

/** 格式化秒数为 天/时/分 */
function fmtUptime(totalSecs: number): string {
  const d = Math.floor(totalSecs / 86400);
  const h = Math.floor((totalSecs % 86400) / 3600);
  const m = Math.floor((totalSecs % 3600) / 60);
  const s = Math.floor(totalSecs % 60);

  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  if (m > 0) return `${m}m ${s}s`;
  return `${s}s`;
}

export function HealthCard() {
  const [health, setHealth] = useState<HealthResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetch = async () => {
      try {
        setHealth(await getHealth());
        setError(null);
      } catch (err) {
        setError(err instanceof Error ? err.message : '连接失败');
      }
    };
    fetch();
    const interval = setInterval(fetch, 15_000);
    return () => clearInterval(interval);
  }, []);

  const isOk = !error && health?.status === 'ok';

  return (
    <div className="rounded-2xl border border-white/[0.06] bg-gradient-to-br from-gray-900 to-gray-900/80 p-5 transition-all duration-300 hover:border-white/[0.12]">
      {/* 标题 + 状态灯 */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-sm font-semibold text-gray-300">系统健康</h3>
        <div className="flex items-center gap-2">
          {/* 状态指示灯 */}
          <span className="relative flex h-2.5 w-2.5">
            <span
              className={`
                animate-ping absolute inline-flex h-full w-full rounded-full opacity-75
                ${isOk ? 'bg-emerald-400' : 'bg-red-500'}
              `}
            />
            <span
              className={`
                relative inline-flex rounded-full h-2.5 w-2.5
                ${isOk ? 'bg-emerald-400' : 'bg-red-500'}
              `}
            />
          </span>
          <span
            className={`text-xs font-medium ${
              isOk ? 'text-emerald-400' : 'text-red-400'
            }`}
          >
            {isOk ? '运行中' : error ?? '未知'}
          </span>
        </div>
      </div>

      {/* 数据行 */}
      <div className="space-y-3">
        {health?.version && (
          <Row icon={Cpu} label="版本" value={health.version} mono />
        )}
        {health?.uptime_seconds !== undefined && (
          <Row icon={Clock} label="运行时间" value={fmtUptime(health.uptime_seconds)} />
        )}
        {health?.agents_running !== undefined && (
          <Row icon={Activity} label="当前 Agent" value={String(health.agents_running)} />
        )}
      </div>

      {/* 加载中 */}
      {!health && !error && (
        <div className="flex items-center gap-2 text-sm text-gray-500 py-2">
          <div className="w-4 h-4 border-2 border-gray-600 border-t-transparent rounded-full animate-spin" />
          连接中...
        </div>
      )}
    </div>
  );
}

/* -------- 内部辅助组件 -------- */

function Row({
  icon: Icon,
  label,
  value,
  mono = false,
}: {
  icon: typeof Activity;
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex items-center gap-3 text-sm">
      <Icon className="w-4 h-4 text-gray-500 shrink-0" />
      <span className="text-gray-400">{label}</span>
      <span
        className={`ml-auto ${
          mono ? 'font-mono' : ''
        } text-gray-200`}
      >
        {value}
      </span>
    </div>
  );
}
