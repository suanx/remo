import { useState, useEffect, useCallback, useRef } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A single event received from the SSE stream. */
export interface StreamEvent {
  /** Event type emitted by the Awaken server (e.g. "text_delta", "tool_call", "thinking"). */
  type: string;
  /** Event payload — shape varies by event type. */
  data: unknown;
  /** ISO-8601 timestamp of the event. */
  timestamp?: string;
}

/** Reactive state of an active stream subscription. */
export interface StreamState {
  /** Whether the SSE connection is open and receiving. */
  connected: boolean;
  /** Accumulated events in order. */
  events: StreamEvent[];
  /** The most recent error, if any. */
  error: Error | null;
  /** Whether the stream has completed (server sent `done` or connection closed). */
  done: boolean;
}

/** Configuration options for the `useStream` hook. */
export interface UseStreamOptions {
  /** SSE endpoint URL (absolute or relative). */
  url: string;
  /** Optional headers sent with the SSE request (e.g. auth tokens). */
  headers?: Record<string, string>;
  /** Called for each parsed event. */
  onEvent?: (event: StreamEvent) => void;
  /** Called when the stream completes. */
  onDone?: () => void;
  /** Called on error. */
  onError?: (error: Error) => void;
  /** If true, start streaming immediately on mount. Default `false`. */
  autoConnect?: boolean;
}

// ---------------------------------------------------------------------------
// Hook
// ---------------------------------------------------------------------------

/**
 * Subscribe to an Awaken SSE streaming endpoint.
 *
 * Returns reactive state and control functions (`connect`, `disconnect`).
 *
 * @example
 * ```tsx
 * const { events, connected, connect } = useStream({
 *   url: '/api/stream',
 *   onEvent: (e) => console.log('event', e),
 * });
 *
 * return (
 *   <button onClick={connect} disabled={connected}>
 *     Start
 *   </button>
 * );
 * ```
 */
export function useStream(options: UseStreamOptions) {
  const { url, headers, onEvent, onDone, onError, autoConnect = false } = options;

  const [state, setState] = useState<StreamState>({
    connected: false,
    events: [],
    error: null,
    done: false,
  });

  const abortRef = useRef<AbortController | null>(null);
  const callbacksRef = useRef({ onEvent, onDone, onError });
  callbacksRef.current = { onEvent, onDone, onError };

  const reset = useCallback(() => {
    setState({ connected: false, events: [], error: null, done: false });
  }, []);

  const disconnect = useCallback(() => {
    abortRef.current?.abort();
    abortRef.current = null;
    setState((prev) => ({ ...prev, connected: false }));
  }, []);

  const connect = useCallback(() => {
    disconnect();
    reset();

    const controller = new AbortController();
    abortRef.current = controller;

    (async () => {
      try {
        const response = await fetch(url, {
          headers: { Accept: 'text/event-stream', ...headers },
          signal: controller.signal,
        });

        if (!response.ok) {
          throw new Error(`SSE connection failed: ${response.status} ${response.statusText}`);
        }

        setState((prev) => ({ ...prev, connected: true }));

        const reader = response.body?.getReader();
        if (!reader) {
          throw new Error('Response body is not readable');
        }

        const decoder = new TextDecoder();
        let buffer = '';

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          buffer += decoder.decode(value, { stream: true });
          const lines = buffer.split('\n');
          buffer = lines.pop() ?? '';

          let currentType = '';
          let currentData = '';

          for (const line of lines) {
            if (line.startsWith('event:')) {
              currentType = line.slice(6).trim();
            } else if (line.startsWith('data:')) {
              currentData = line.slice(5).trim();
            } else if (line === '') {
              if (currentType === 'done' || currentData === '[DONE]') {
                setState((prev) => ({ ...prev, done: true, connected: false }));
                callbacksRef.current.onDone?.();
                return;
              }

              if (currentType || currentData) {
                let parsedData: unknown = currentData;
                try {
                  parsedData = JSON.parse(currentData);
                } catch {
                  // Keep as string
                }

                const event: StreamEvent = {
                  type: currentType || 'message',
                  data: parsedData,
                  timestamp: new Date().toISOString(),
                };

                setState((prev) => ({ ...prev, events: [...prev.events, event] }));
                callbacksRef.current.onEvent?.(event);

                currentType = '';
                currentData = '';
              }
            }
          }
        }

        setState((prev) => ({ ...prev, done: true, connected: false }));
        callbacksRef.current.onDone?.();
      } catch (err) {
        if ((err as Error).name === 'AbortError') return;
        const error = err instanceof Error ? err : new Error(String(err));
        setState((prev) => ({ ...prev, error, connected: false }));
        callbacksRef.current.onError?.(error);
      }
    })();
  }, [url, headers, disconnect, reset]);

  // Auto-connect on mount when requested.
  useEffect(() => {
    if (autoConnect) {
      connect();
    }
    return disconnect;
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [autoConnect]);

  return {
    ...state,
    connect,
    disconnect,
    reset,
  };
}
