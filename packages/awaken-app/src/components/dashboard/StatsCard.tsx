import { type LucideIcon } from 'lucide-react';

export type StatsCardColor = 'green' | 'blue' | 'purple' | 'orange';

interface StatsCardProps {
  label: string;
  value: string | number;
  icon: LucideIcon;
  color: StatsCardColor;
  trend?: string;
}

const COLOR_STYLES: Record<StatsCardColor, {
  bg: string;
  text: string;
  glow: string;
  ring: string;
}> = {
  green: {
    bg: 'bg-emerald-500/10',
    text: 'text-emerald-400',
    glow: 'shadow-emerald-500/8',
    ring: 'ring-emerald-500/20',
  },
  blue: {
    bg: 'bg-sky-500/10',
    text: 'text-sky-400',
    glow: 'shadow-sky-500/8',
    ring: 'ring-sky-500/20',
  },
  purple: {
    bg: 'bg-violet-500/10',
    text: 'text-violet-400',
    glow: 'shadow-violet-500/8',
    ring: 'ring-violet-500/20',
  },
  orange: {
    bg: 'bg-orange-500/10',
    text: 'text-orange-400',
    glow: 'shadow-orange-500/8',
    ring: 'ring-orange-500/20',
  },
};

export function StatsCard({ label, value, icon: Icon, color, trend }: StatsCardProps) {
  const c = COLOR_STYLES[color];

  return (
    <div
      className={`
        relative overflow-hidden rounded-2xl border border-white/[0.06]
        bg-gradient-to-br from-gray-900 to-gray-900/80 p-5
        transition-all duration-300 hover:border-white/[0.12]
        hover:shadow-lg ${c.glow} group
      `}
    >
      {/* 微光晕背景 */}
      <div
        className={`
          absolute -top-10 -right-10 w-28 h-28 rounded-full blur-3xl
          opacity-40 group-hover:opacity-60 transition-opacity duration-500
          ${c.bg}
        `}
      />

      <div className="relative flex items-start justify-between">
        <div className="space-y-1.5">
          <p className="text-xs font-medium text-gray-500 uppercase tracking-wider">
            {label}
          </p>
          <p className="text-3xl font-bold text-gray-100 tabular-nums tracking-tight">
            {value}
          </p>
          {trend && (
            <p className="text-xs text-gray-500">{trend}</p>
          )}
        </div>

        <div
          className={`
            w-10 h-10 rounded-xl flex items-center justify-center shrink-0
            ${c.bg} ring-1 ${c.ring}
          `}
        >
          <Icon className={`w-5 h-5 ${c.text}`} />
        </div>
      </div>
    </div>
  );
}
