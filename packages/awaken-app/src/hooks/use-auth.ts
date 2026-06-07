import { useState, useCallback } from 'react';

const AUTH_KEY = 'remo-admin-token';

interface AuthState {
  isAuthenticated: boolean;
  token: string | null;
  login: (token: string) => Promise<boolean>;
  logout: () => void;
  loading: boolean;
  error: string | null;
}

export function useAuth(): AuthState {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem(AUTH_KEY));
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isAuthenticated = !!token;

  const login = useCallback(async (inputToken: string): Promise<boolean> => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch('/api/admin/login', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token: inputToken }),
      });
      if (!res.ok) {
        const err = await res.json().catch(() => ({ error: '登录失败' }));
        throw new Error(err.error || 'Token 验证失败');
      }
      localStorage.setItem(AUTH_KEY, inputToken);
      setToken(inputToken);
      return true;
    } catch (e) {
      const msg = e instanceof Error ? e.message : '登录失败';
      setError(msg);
      return false;
    } finally {
      setLoading(false);
    }
  }, []);

  const logout = useCallback(() => {
    localStorage.removeItem(AUTH_KEY);
    setToken(null);
    setError(null);
  }, []);

  return { isAuthenticated, token, login, logout, loading, error };
}

/// 获取存储的 token（用于 API 请求头）
export function getAuthToken(): string | null {
  return localStorage.getItem(AUTH_KEY);
}

/// 生成 Authorization 请求头
export function authHeaders(): Record<string, string> {
  const token = getAuthToken();
  return token ? { Authorization: `Bearer ${token}` } : {};
}
