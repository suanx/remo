import { useState, useCallback } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A single message in a conversation thread. */
export interface ThreadMessage {
  /** Unique message ID. */
  id: string;
  /** Message role: "user", "assistant", "system", or "tool". */
  role: 'user' | 'assistant' | 'system' | 'tool';
  /** Text content of the message. */
  content: string;
  /** ISO-8601 timestamp. */
  timestamp: string;
  /** Optional tool calls attached to this message (assistant role). */
  toolCalls?: ToolCallInfo[];
  /** Optional metadata key-value pairs. */
  metadata?: Record<string, unknown>;
}

/** Tool call information attached to an assistant message. */
export interface ToolCallInfo {
  /** Unique call ID. */
  id: string;
  /** Tool name. */
  name: string;
  /** Serialized arguments. */
  arguments: string;
  /** Tool result content, if already executed. */
  result?: string;
}

/** A conversation thread with metadata. */
export interface Thread {
  /** Unique thread ID. */
  id: string;
  /** Thread display name. */
  name: string;
  /** Messages in chronological order. */
  messages: ThreadMessage[];
  /** ISO-8601 creation timestamp. */
  createdAt: string;
  /** ISO-8601 last-update timestamp. */
  updatedAt: string;
}

/** Configuration options for the `useThread` hook. */
export interface UseThreadOptions {
  /** Base URL of the Awaken server API. */
  baseUrl: string;
  /** Optional auth headers. */
  headers?: Record<string, string>;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Manage a conversation thread: create, send messages, append assistant responses.
 *
 * @example
 * ```tsx
 * const { thread, sendMessage, loading } = useThread({
 *   baseUrl: '/api',
 * });
 *
 * return (
 *   <div>
 *     {thread.messages.map((m) => (
 *       <div key={m.id}>{m.content}</div>
 *     ))}
 *     <button onClick={() => sendMessage('Hello')} disabled={loading}>
 *       Send
 *     </button>
 *   </div>
 * );
 * ```
 */
export function useThread(options: UseThreadOptions) {
  const { baseUrl, headers = {} } = options;

  const [thread, setThread] = useState<Thread>(() => ({
    id: crypto.randomUUID(),
    name: 'New Thread',
    messages: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
  }));
  const [loading, setLoading] = useState(false);

  const addUserMessage = useCallback((content: string): ThreadMessage => {
    const msg: ThreadMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content,
      timestamp: new Date().toISOString(),
    };
    setThread((prev) => ({
      ...prev,
      messages: [...prev.messages, msg],
      updatedAt: new Date().toISOString(),
    }));
    return msg;
  }, []);

  const addAssistantMessage = useCallback(
    (content: string, toolCalls?: ToolCallInfo[]): ThreadMessage => {
      const msg: ThreadMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content,
        timestamp: new Date().toISOString(),
        toolCalls,
      };
      setThread((prev) => ({
        ...prev,
        messages: [...prev.messages, msg],
        updatedAt: new Date().toISOString(),
      }));
      return msg;
    },
    [],
  );

  /**
   * Send a user message and stream the assistant response.
   * Appends both the user message and the streamed assistant response to the thread.
   */
  const sendMessage = useCallback(
    async (content: string) => {
      addUserMessage(content);
      setLoading(true);

      try {
        const response = await fetch(`${baseUrl}/threads/${thread.id}/messages`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', ...headers },
          body: JSON.stringify({ role: 'user', content }),
        });

        if (!response.ok) {
          throw new Error(`Failed to send message: ${response.status}`);
        }

        const data = await response.json();
        if (data.content) {
          addAssistantMessage(data.content, data.tool_calls);
        }
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        addAssistantMessage(`[Error: ${errorMsg}]`);
      } finally {
        setLoading(false);
      }
    },
    [baseUrl, headers, thread.id, addUserMessage, addAssistantMessage],
  );

  /** Clear all messages in the thread. */
  const clearThread = useCallback(() => {
    setThread((prev) => ({
      ...prev,
      messages: [],
      updatedAt: new Date().toISOString(),
    }));
  }, []);

  /** Replace the entire thread state. */
  const setThreadState = useCallback((newThread: Thread) => {
    setThread(newThread);
  }, []);

  return {
    thread,
    loading,
    sendMessage,
    addUserMessage,
    addAssistantMessage,
    clearThread,
    setThread: setThreadState,
  };
}
