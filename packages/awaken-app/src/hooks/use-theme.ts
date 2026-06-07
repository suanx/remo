import { useState, useEffect, useCallback } from 'react';

type Theme = 'light' | 'dark';
const STORAGE_KEY = 'awaken-theme';

function getSystemTheme(): Theme {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function getStoredTheme(): Theme | null {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === 'light' || stored === 'dark') return stored;
  } catch {
    // localStorage 不可用
  }
  return null;
}

function resolveTheme(): Theme {
  return getStoredTheme() ?? getSystemTheme();
}

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(resolveTheme);

  // 应用到 <html>
  const applyTheme = useCallback((t: Theme) => {
    document.documentElement.classList.toggle('dark', t === 'dark');
  }, []);

  // 切换
  const toggleTheme = useCallback(() => {
    setThemeState((prev) => {
      const next = prev === 'dark' ? 'light' : 'dark';
      try {
        localStorage.setItem(STORAGE_KEY, next);
      } catch { /* ignore */ }
      applyTheme(next);
      return next;
    });
  }, [applyTheme]);

  // 设置指定主题
  const setTheme = useCallback(
    (t: Theme) => {
      try {
        localStorage.setItem(STORAGE_KEY, t);
      } catch { /* ignore */ }
      applyTheme(t);
      setThemeState(t);
    },
    [applyTheme],
  );

  // 初始化 & 监听系统偏好变化
  useEffect(() => {
    const initial = resolveTheme();
    applyTheme(initial);
    setThemeState(initial);

    const mql = window.matchMedia('(prefers-color-scheme: dark)');
    const handler = (e: MediaQueryListEvent) => {
      // 仅在用户没有手动存储偏好时跟随系统
      if (!getStoredTheme()) {
        const next = e.matches ? 'dark' : 'light';
        applyTheme(next);
        setThemeState(next);
      }
    };
    mql.addEventListener('change', handler);
    return () => mql.removeEventListener('change', handler);
  }, [applyTheme]);

  return { theme, toggleTheme, setTheme, isDark: theme === 'dark' };
}
