import { useState, useCallback } from 'react';
import { Eye, EyeOff, Loader2, AlertCircle } from 'lucide-react';
import { useAuth } from '../hooks/use-auth';
import { useNavigate } from 'react-router-dom';

/* ===================================================================
   LoginPage — 管理员登录页
   科技感暗黑风 · 毛玻璃卡片 · 渐变品牌色 · 响应式
   =================================================================== */

export function LoginPage() {
  const navigate = useNavigate();
  const { login, loading, error } = useAuth();

  const [tokenInput, setTokenInput] = useState('');
  const [showToken, setShowToken] = useState(false);
  const [btnScale, setBtnScale] = useState(1);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!tokenInput.trim() || loading) return;
      const ok = await login(tokenInput.trim());
      if (ok) {
        navigate('/admin/chat', { replace: true });
      }
    },
    [tokenInput, loading, login, navigate],
  );

  return (
    <div className="relative min-h-screen w-full flex items-center justify-center overflow-hidden bg-gray-950">
      {/* ===== 动态渐变背景层 ===== */}
      <div className="absolute inset-0 bg-gradient-to-br from-gray-950 via-indigo-950/40 to-purple-950/30" />
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_top_right,rgba(99,102,241,0.12),transparent_70%)]" />
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_bottom_left,rgba(139,92,246,0.08),transparent_70%)]" />

      {/* ===== 背景装饰光晕 ===== */}
      <div className="absolute top-1/4 left-1/4 w-96 h-96 rounded-full bg-indigo-500/10 blur-[120px]" />
      <div className="absolute bottom-1/4 right-1/4 w-96 h-96 rounded-full bg-purple-500/10 blur-[120px]" />

      {/* ===== 网格纹理叠加 ===== */}
      <div
        className="absolute inset-0 opacity-[0.03]"
        style={{
          backgroundImage:
            'linear-gradient(rgba(255,255,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(255,255,255,0.1) 1px, transparent 1px)',
          backgroundSize: '40px 40px',
        }}
      />

      {/* ===== 登录卡片 ===== */}
      <div className="relative z-10 w-full max-w-[420px] px-4 sm:px-0">
        <div
          className="
            w-full rounded-2xl
            bg-white/[0.06]
            backdrop-blur-2xl
            border border-white/[0.08]
            shadow-[0_8px_32px_rgba(0,0,0,0.4)]
            p-8 sm:p-10
          "
        >
          {/* ===== Logo + 标题 ===== */}
          <div className="flex flex-col items-center mb-8">
            {/* Logo icon */}
            <div className="relative mb-4">
              <div className="w-14 h-14 rounded-2xl bg-gradient-to-br from-indigo-400 to-purple-600 flex items-center justify-center shadow-lg shadow-indigo-500/30">
                <span className="text-white font-bold text-xl">R</span>
              </div>
              {/* 状态指示光点 */}
              <div className="absolute -top-1 -right-1 w-3 h-3 rounded-full bg-emerald-400 animate-pulse shadow-lg shadow-emerald-400/60" />
            </div>

            {/* 标题 */}
            <h1 className="text-2xl font-bold text-gray-100 tracking-tight">
              Remo AI
            </h1>
            <p className="mt-1 text-sm text-gray-500">管理后台</p>
          </div>

          {/* ===== 登录表单 ===== */}
          <form onSubmit={handleSubmit} className="space-y-5">
            {/* Token 输入框 */}
            <div>
              <label
                htmlFor="admin-token"
                className="block text-xs font-medium text-gray-400 mb-1.5 tracking-wide uppercase"
              >
                管理员 Token
              </label>
              <div className="relative">
                <input
                  id="admin-token"
                  type={showToken ? 'text' : 'password'}
                  value={tokenInput}
                  onChange={(e) => setTokenInput(e.target.value)}
                  placeholder="请输入您的访问令牌"
                  autoFocus
                  disabled={loading}
                  className="
                    w-full h-11 pl-3.5 pr-10
                    bg-white/[0.04]
                    border border-white/[0.08]
                    rounded-xl
                    text-sm text-gray-100 placeholder-gray-600
                    outline-none
                    transition-all duration-200
                    focus:border-indigo-500/60 focus:bg-white/[0.06]
                    focus:shadow-[0_0_0_3px_rgba(99,102,241,0.15)]
                    disabled:opacity-50 disabled:cursor-not-allowed
                  "
                />
                {/* 显/隐切换按钮 */}
                <button
                  type="button"
                  onClick={() => setShowToken((v) => !v)}
                  disabled={loading}
                  className="
                    absolute right-2.5 top-1/2 -translate-y-1/2
                    w-7 h-7 flex items-center justify-center
                    rounded-lg text-gray-500 hover:text-gray-300
                    transition-colors duration-150
                    disabled:opacity-50
                  "
                  tabIndex={-1}
                  aria-label={showToken ? '隐藏 Token' : '显示 Token'}
                >
                  {showToken ? (
                    <EyeOff className="w-4 h-4" />
                  ) : (
                    <Eye className="w-4 h-4" />
                  )}
                </button>
              </div>
            </div>

            {/* 错误提示 */}
            {error && (
              <div
                className="
                  flex items-start gap-2.5 px-3.5 py-2.5
                  rounded-xl bg-red-500/10 border border-red-500/20
                  animate-[fadeIn_0.25s_ease-out]
                "
              >
                <AlertCircle className="w-4 h-4 text-red-400 mt-0.5 shrink-0" />
                <span className="text-xs text-red-300 leading-relaxed">
                  {error}
                </span>
              </div>
            )}

            {/* 登录按钮 */}
            <button
              type="submit"
              disabled={loading || !tokenInput.trim()}
              onClick={(e) => {
                // 点击缩放反馈
                setBtnScale(0.96);
                setTimeout(() => setBtnScale(1), 150);
              }}
              className="
                relative w-full h-11
                bg-gradient-to-r from-indigo-500 to-purple-600
                hover:from-indigo-400 hover:to-purple-500
                rounded-xl
                text-sm font-medium text-white
                shadow-lg shadow-indigo-500/25
                transition-all duration-200
                hover:brightness-110
                active:brightness-90
                disabled:opacity-40 disabled:cursor-not-allowed
                disabled:hover:brightness-100
                flex items-center justify-center gap-2
                overflow-hidden
              "
              style={{ transform: `scale(${btnScale})` }}
            >
              {/* 按钮悬浮光晕 */}
              <div className="absolute inset-0 bg-white/0 hover:bg-white/[0.06] transition-colors duration-200" />

              {loading ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  <span>验证中…</span>
                </>
              ) : (
                <span>登 录</span>
              )}
            </button>
          </form>

          {/* ===== 底部提示 ===== */}
          <p className="mt-6 text-center text-xs text-gray-600">
            请输入管理员 Token 以登录
          </p>
        </div>

        {/* ===== 版本号 ===== */}
        <p className="mt-4 text-center text-[11px] text-gray-700 font-mono tracking-wider">
          Remo AI · v0.1.0
        </p>
      </div>
    </div>
  );
}
