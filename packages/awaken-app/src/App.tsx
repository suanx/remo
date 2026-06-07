import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { Layout } from './components/layout/Layout';
import { LoginPage } from './pages/LoginPage';
import { ChatPage } from './pages/ChatPage';
import { DashboardPage } from './pages/DashboardPage';
import { SettingsPage } from './pages/SettingsPage';
import { type ReactNode } from 'react';

const AUTH_KEY = 'remo-admin-token';

function AuthGuard({ children }: { children: ReactNode }) {
  const token = localStorage.getItem(AUTH_KEY);
  if (!token) {
    return <Navigate to="/admin/login" replace />;
  }
  return <>{children}</>;
}

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        {/* 公开路由：登录页（无 Layout） */}
        <Route path="/admin/login" element={<LoginPage />} />

        {/* 受保护路由（有 Layout + AuthGuard） */}
        <Route
          path="/admin"
          element={
            <AuthGuard>
              <Layout />
            </AuthGuard>
          }
        >
          <Route index element={<Navigate to="/admin/chat" replace />} />
          <Route path="chat" element={<ChatPage />} />
          <Route path="dashboard" element={<DashboardPage />} />
          <Route path="settings" element={<SettingsPage />} />
        </Route>

        {/* 根路径重定向 */}
        <Route path="/" element={<Navigate to="/admin/chat" replace />} />
        <Route path="*" element={<Navigate to="/admin/chat" replace />} />
      </Routes>
    </BrowserRouter>
  );
}
