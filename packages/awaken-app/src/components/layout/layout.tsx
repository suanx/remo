import { useState } from 'react';
import { NavLink, Outlet } from 'react-router-dom';
import { MessageSquare, LayoutDashboard, Settings, Menu, X } from 'lucide-react';

const NAV_ITEMS = [
  { path: '/admin/chat', label: '对话', icon: MessageSquare },
  { path: '/admin/dashboard', label: '仪表盘', icon: LayoutDashboard },
  { path: '/admin/settings', label: '设置', icon: Settings },
];

export function Layout() {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <div className="h-screen w-screen flex overflow-hidden bg-gray-950 text-gray-100 selection:bg-indigo-500/20">
      {/* ===== 移动端遮罩 ===== */}
      {sidebarOpen && (
        <div
          className="fixed inset-0 bg-black/60 backdrop-blur-sm z-20 lg:hidden"
          onClick={() => setSidebarOpen(false)}
        />
      )}

      {/* ===== 侧边栏 ===== */}
      <aside
        className={`
          fixed lg:static inset-y-0 left-0 z-30
          flex flex-col w-64
          bg-gradient-to-b from-gray-900 via-gray-900/95 to-gray-950
          border-r border-white/[0.06] backdrop-blur-xl
          transform transition-transform duration-300 ease-out
          ${sidebarOpen ? 'translate-x-0' : '-translate-x-full lg:translate-x-0'}
        `}
      >
        {/* Logo 区 */}
        <div className="flex items-center h-16 px-6 border-b border-white/[0.06] shrink-0">
          <div className="flex items-center gap-3">
            <div className="relative">
              <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-indigo-400 to-purple-600 flex items-center justify-center text-white font-bold text-sm shadow-lg shadow-indigo-500/30">
                R
              </div>
              {/* 发光点 */}
              <div className="absolute -top-0.5 -right-0.5 w-2.5 h-2.5 rounded-full bg-emerald-400 animate-pulse shadow-lg shadow-emerald-400/60" />
            </div>
            <span className="font-semibold text-gray-100 text-sm tracking-tight">
              Remo AI
            </span>
          </div>
        </div>

        {/* 导航 */}
        <nav className="flex-1 px-3 py-5 space-y-1 overflow-y-auto">
          {NAV_ITEMS.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === '/admin'}
              onClick={() => setSidebarOpen(false)}
              className={({ isActive }) => `
                flex items-center gap-3 px-3 py-2.5 rounded-xl text-sm
                transition-all duration-200
                ${
                  isActive
                    ? 'bg-white/[0.08] text-white font-medium shadow-sm'
                    : 'text-gray-400 hover:bg-white/[0.04] hover:text-gray-200'
                }
              `}
            >
              <item.icon className="w-4 h-4 shrink-0" />
              <span>{item.label}</span>
            </NavLink>
          ))}
        </nav>

        {/* 底部：版本号 + 状态指示器 */}
        <div className="px-4 py-4 border-t border-white/[0.06] shrink-0">
          <div className="flex items-center justify-between text-xs text-gray-500">
            <span className="font-mono">v0.1.0</span>
            <div className="flex items-center gap-1.5">
              <span className="relative flex h-2 w-2">
                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75" />
                <span className="relative inline-flex rounded-full h-2 w-2 bg-emerald-400" />
              </span>
              <span>正常</span>
            </div>
          </div>
        </div>
      </aside>

      {/* ===== 移动端汉堡菜单按钮 ===== */}
      <button
        onClick={() => setSidebarOpen((v) => !v)}
        className="fixed top-4 left-4 z-40 lg:hidden w-9 h-9 rounded-lg bg-gray-800/80 border border-white/[0.08] backdrop-blur-sm flex items-center justify-center text-gray-400 hover:text-gray-200 transition-colors"
        aria-label={sidebarOpen ? '关闭侧边栏' : '打开侧边栏'}
      >
        {sidebarOpen ? <X className="w-4 h-4" /> : <Menu className="w-4 h-4" />}
      </button>

      {/* ===== 主内容区：使用 Outlet 渲染子路由 ===== */}
      <main className="flex-1 flex flex-col min-w-0 overflow-hidden bg-gray-950">
        <Outlet />
      </main>
    </div>
  );
}
