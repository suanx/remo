import React, { useEffect, useRef, useState } from 'react';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface StreamingTextProps {
  /** The full text content to render. */
  text: string;
  /** Whether the stream is still active (cursor shown when true). */
  streaming?: boolean;
  /** Milliseconds per character for the typewriter effect. Default 20. */
  speed?: number;
  /** CSS class name for the outer container. */
  className?: string;
  /** Enable typewriter character-by-character reveal. Default true when streaming. */
  typewriter?: boolean;
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

/**
 * Renders text with an optional typewriter streaming animation.
 *
 * When `streaming` is true, a blinking cursor is appended and new characters
 * are revealed at the configured `speed`. When the stream completes, the full
 * text is shown instantly.
 *
 * @example
 * ```tsx
 * <StreamingText text={delta} streaming={isStreaming} />
 * ```
 */
export function StreamingText({
  text,
  streaming = false,
  speed = 20,
  className,
  typewriter = true,
}: StreamingTextProps) {
  const [displayed, setDisplayed] = useState(text);
  const [cursorVisible, setCursorVisible] = useState(true);
  const prevTextRef = useRef(text);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Typewriter effect: reveal new characters progressively.
  useEffect(() => {
    if (!typewriter || !streaming) {
      setDisplayed(text);
      prevTextRef.current = text;
      return;
    }

    const prevText = prevTextRef.current;

    // If text shrank (reset), show immediately.
    if (text.length < prevText.length) {
      setDisplayed(text);
      prevTextRef.current = text;
      return;
    }

    // Reveal from where we left off.
    let currentIndex = prevText.length;
    const target = text;

    const tick = () => {
      if (currentIndex < target.length) {
        currentIndex++;
        setDisplayed(target.slice(0, currentIndex));
        timerRef.current = setTimeout(tick, speed);
      } else {
        prevTextRef.current = target;
      }
    };

    timerRef.current = setTimeout(tick, speed);

    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [text, streaming, speed, typewriter]);

  // Blinking cursor.
  useEffect(() => {
    if (!streaming) {
      setCursorVisible(false);
      return;
    }
    const interval = setInterval(() => {
      setCursorVisible((v) => !v);
    }, 530);
    return () => clearInterval(interval);
  }, [streaming]);

  return (
    <span className={className} style={{ fontFamily: 'inherit' }}>
      {displayed}
      {streaming && (
        <span
          style={{
            opacity: cursorVisible ? 1 : 0,
            transition: 'opacity 0.1s',
            marginLeft: 1,
            borderRight: '2px solid currentColor',
          }}
          aria-hidden="true"
        />
      )}
    </span>
  );
}
