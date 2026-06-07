/* ===================================================================
 * useThreads —— 线程管理 Hook
 *
 * 提供线程的 CRUD 操作与选择状态管理。
 * 依赖 ApiClient 进行后端通信。
 * =================================================================== */

import { useState, useEffect, useCallback } from 'react';
import type { Thread } from '../types/api';
import { apiClient } from '../lib/api-client';

/**
 * useThreads —— 线程管理 Hook。
 *
 * @returns 线程列表、当前选中线程 ID、以及各种操作方法
 *
 * @example
 * ```tsx
 * const { threads, activeThreadId, createThread, selectThread, deleteThread, refreshThreads } = useThreads();
 *
 * // 创建线程
 * const newThread = await createThread('新对话');
 *
 * // 切换线程
 * selectThread(newThread.thread_id);
 *
 * // 删除线程
 * await deleteThread('thread-xxx');
 * ```
 */
export function useThreads() {
  const [threads, setThreads] = useState<Thread[]>([]);
  const [activeThreadId, setActiveThreadId] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  /* ---- 刷新线程列表 ---- */

  /**
   * 从后端重新加载线程列表。
   * 自动处理加载状态和错误。
   */
  const refreshThreads = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      const data = await apiClient.listThreads();
      setThreads(data);

      // 如果当前选中的线程不再存在于列表中，清空选择
      setActiveThreadId((prev) => {
        if (prev && !data.some((t) => t.thread_id === prev)) {
          return null;
        }
        return prev;
      });
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : '获取线程列表失败';
      setError(message);
    } finally {
      setLoading(false);
    }
  }, []);

  // 组件挂载时自动加载
  useEffect(() => {
    refreshThreads();
  }, [refreshThreads]);

  /* ---- 创建线程 ---- */

  /**
   * 创建新线程。
   *
   * @param title - 线程标题（可选）
   * @returns 创建的 Thread 对象
   * @throws 创建失败时抛出 Error
   */
  const createThread = useCallback(
    async (title?: string): Promise<Thread> => {
      setError(null);

      try {
        const thread = await apiClient.createThread(
          title ? { title } : undefined,
        );

        // 将新线程插入列表头部
        setThreads((prev) => [thread, ...prev]);

        // 自动选中新创建的线程
        setActiveThreadId(thread.thread_id);

        return thread;
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : '创建线程失败';
        setError(message);
        throw err;
      }
    },
    [],
  );

  /* ---- 选择线程 ---- */

  /**
   * 切换当前活跃线程。
   *
   * @param id - 要选中的线程 ID
   */
  const selectThread = useCallback((id: string) => {
    setActiveThreadId(id);
    setError(null);
  }, []);

  /* ---- 删除线程 ---- */

  /**
   * 删除指定线程。
   *
   * @param id - 要删除的线程 ID
   * @throws 删除失败时抛出 Error
   */
  const deleteThread = useCallback(
    async (id: string): Promise<void> => {
      setError(null);

      try {
        await apiClient.deleteThread(id);

        // 从列表中移除
        setThreads((prev) => prev.filter((t) => t.thread_id !== id));

        // 如果删除的是当前选中线程，清空选择
        setActiveThreadId((prev) => (prev === id ? null : prev));
      } catch (err: unknown) {
        const message = err instanceof Error ? err.message : '删除线程失败';
        setError(message);
        throw err;
      }
    },
    [],
  );

  return {
    /* 新 API（符合任务规格） */
    /** 线程列表 */
    threads,
    /** 当前选中的线程 ID */
    activeThreadId,
    /** 是否正在加载 */
    loading,
    /** 错误信息 */
    error,
    /** 创建新线程（自动选中） */
    createThread,
    /** 切换选中线程 */
    selectThread,
    /** 删除线程 */
    deleteThread,
    /** 刷新线程列表 */
    refreshThreads,

    /* 旧 API（向后兼容） */
    /** @deprecated 请使用 createThread */
    addThread: createThread,
    /** @deprecated 请使用 deleteThread */
    removeThread: deleteThread,
  } as const;
}
