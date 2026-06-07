import React, { useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface ThinkingRevealProps {
  /** The thinking/reasoning content to display. */
  content: string;
  /** Label shown in the collapsed header. Default "Thinking". */
  label?: string;
  /** Whether the thinking block is still being streamed. */
  streaming?: boolean;
  /** Whether the content is initially expanded. Default false. */
  defaultExpanded?: boolean;
  /** CSS class name for the outer container. */
  className?: string;
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const styles: Record<string, React.CSSProperties> = {
  container: {
    margin: '4px 0',
    borderRadius: 8,
    border: '1px dashed #cbd5e1',
    overflow: 'hidden',
    fontSize: 13,
  },
  toggle: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    padding: '6px 10px',
    cursor: 'pointer',
    userSelect: 'none',
    background: '#f1f5f9',
    color: '#475569',
    fontSize: 12,
    fontWeight: 500,
  },
  icon: {
    transition: 'transform 0.15s',
    fontSize: 10,
  },
  body: {
    padding: '8px 10px',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    color: '#64748b',
    fontStyle: 'italic',
    lineHeight: 1.6,
    maxHeight: 300,
    overflow: 'auto',
    background: '#fafbfc',
  },
  pulse: {
    display: 'inline-block',
    width: 6,
    height: 6,
    borderRadius: '50%',
    background: '#3b82f6',
    animation: 'thinking-pulse 1.4s ease-in-out infinite',
  },
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * Collapsible "thinking" block — shows the LLM's reasoning process.
 *
 * Starts collapsed; click to expand. When `streaming` is true, a pulsing
 * indicator shows that content is still arriving.
 *
 * @example
 * ```tsx
 * <ThinkingReveal
 *   content="Let me break down this problem..."
 *   streaming={isThinking}
 * />
 * ```
 */
export function ThinkingReveal({
  content,
  label = 'Thinking',
  streaming = false,
  defaultExpanded = false,
  className,
}: ThinkingRevealProps) {
  const [expanded, setExpanded] = useState(defaultExpanded);

  return (
    <div className={className} style={styles.container}>
      <div
        style={styles.toggle}
        onClick={() => setExpanded((e) => !e)}
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            setExpanded((e) => !e);
          }
        }}
      >
        <span
          style={{
            ...styles.icon,
            transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
          }}
        >
          ▶
        </span>
        <span>{label}</span>
        {streaming && <span style={styles.pulse} />}
      </div>

      {expanded && content && <div style={styles.body}>{content}</div>}

      {streaming && (
        <style>{`
          @keyframes thinking-pulse {
            0%, 100% { opacity: 1; transform: scale(1); }
            50% { opacity: 0.4; transform: scale(0.8); }
          }
        `}</style>
      )}
    </div>
  );
}
