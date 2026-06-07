import React, { useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface ToolCallCardProps {
  /** Tool name. */
  toolName: string;
  /** Serialized tool arguments (JSON string or object). */
  args: string | Record<string, unknown>;
  /** Tool result content, if available. */
  result?: string;
  /** Execution status. */
  status?: 'pending' | 'success' | 'error';
  /** Whether the result section is initially expanded. Default false. */
  defaultExpanded?: boolean;
  /** CSS class name for the outer container. */
  className?: string;
}

// ---------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------

const styles: Record<string, React.CSSProperties> = {
  card: {
    border: '1px solid #e2e8f0',
    borderRadius: 8,
    padding: '8px 12px',
    margin: '4px 0',
    fontFamily: 'ui-monospace, monospace',
    fontSize: 13,
    lineHeight: 1.5,
  },
  header: {
    display: 'flex',
    alignItems: 'center',
    gap: 6,
    cursor: 'pointer',
    userSelect: 'none',
  },
  badge: {
    display: 'inline-block',
    padding: '1px 6px',
    borderRadius: 4,
    fontSize: 11,
    fontWeight: 600,
    lineHeight: '18px',
  },
  chevron: {
    transition: 'transform 0.15s',
    fontSize: 10,
    color: '#94a3b8',
  },
  section: {
    marginTop: 6,
    padding: 8,
    background: '#f8fafc',
    borderRadius: 4,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
    fontSize: 12,
    maxHeight: 200,
    overflow: 'auto',
  },
};

const statusColors: Record<string, { bg: string; text: string }> = {
  pending: { bg: '#fef3c7', text: '#92400e' },
  success: { bg: '#d1fae5', text: '#065f46' },
  error: { bg: '#fee2e2', text: '#991b1b' },
};

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * Visual card for a single tool call — shows tool name, arguments, and result.
 *
 * Click the header to expand/collapse the argument and result sections.
 *
 * @example
 * ```tsx
 * <ToolCallCard
 *   toolName="calculator"
 *   args={{ expression: "2 + 2" }}
 *   result="4"
 *   status="success"
 * />
 * ```
 */
export function ToolCallCard({
  toolName,
  args,
  result,
  status = 'success',
  defaultExpanded = false,
  className,
}: ToolCallCardProps) {
  const [expanded, setExpanded] = useState(defaultExpanded);

  const argsStr =
    typeof args === 'string'
      ? args
      : JSON.stringify(args, null, 2);

  const colors = statusColors[status] ?? statusColors.pending;

  return (
    <div className={className} style={styles.card}>
      <div
        style={styles.header}
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
            ...styles.badge,
            background: colors.bg,
            color: colors.text,
          }}
        >
          {status === 'pending' ? '⏳' : status === 'success' ? '✓' : '✗'}
        </span>
        <strong style={{ fontSize: 12 }}>{toolName}</strong>
        <span
          style={{
            ...styles.chevron,
            transform: expanded ? 'rotate(90deg)' : 'rotate(0deg)',
          }}
        >
          ▶
        </span>
      </div>

      {expanded && (
        <>
          <div style={styles.section}>
            <div style={{ fontWeight: 600, marginBottom: 4, fontSize: 11, color: '#64748b' }}>
              Arguments
            </div>
            {argsStr}
          </div>
          {result != null && (
            <div style={styles.section}>
              <div style={{ fontWeight: 600, marginBottom: 4, fontSize: 11, color: '#64748b' }}>
                Result
              </div>
              {result}
            </div>
          )}
        </>
      )}
    </div>
  );
}
