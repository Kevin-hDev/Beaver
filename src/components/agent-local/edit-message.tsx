import { useCallback, useEffect, useLayoutEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  USER_MESSAGE_EDIT_MIN_LINES,
  USER_MESSAGE_MAX_LINES,
  userMessageHeightForLines,
} from "@/lib/user-message-layout";
import "./edit-message.css";

interface EditMessageProps {
  initialContent: string;
  onSave: (content: string) => void;
  onCancel: () => void;
}

export function EditMessage({ initialContent, onSave, onCancel }: EditMessageProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(initialContent);
  const [textHeight, setTextHeight] = useState<number>();
  const [overflowing, setOverflowing] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const scrollFrameRef = useRef<number | null>(null);

  const measureHeight = useCallback(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    const followsEnd = textarea.selectionStart === textarea.value.length
      && textarea.selectionEnd === textarea.value.length;
    textarea.style.height = "auto";
    const naturalHeight = textarea.scrollHeight;
    const minHeight = userMessageHeightForLines(textarea, USER_MESSAGE_EDIT_MIN_LINES);
    const maxHeight = userMessageHeightForLines(textarea, USER_MESSAGE_MAX_LINES);
    const nextHeight = Math.max(minHeight, Math.min(naturalHeight, maxHeight));
    const nextOverflowing = naturalHeight > maxHeight;

    textarea.style.height = `${nextHeight}px`;
    setTextHeight((current) => current === nextHeight ? current : nextHeight);
    setOverflowing((current) => current === nextOverflowing ? current : nextOverflowing);

    if (nextOverflowing && followsEnd) {
      if (scrollFrameRef.current !== null) cancelAnimationFrame(scrollFrameRef.current);
      scrollFrameRef.current = requestAnimationFrame(() => {
        textarea.scrollTop = textarea.scrollHeight;
        scrollFrameRef.current = null;
      });
    }
  }, []);

  useLayoutEffect(() => {
    measureHeight();
  }, [content, measureHeight]);

  useLayoutEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;

    let previousWidth = textarea.getBoundingClientRect().width;
    if (typeof ResizeObserver === "undefined") {
      window.addEventListener("resize", measureHeight);
      return () => window.removeEventListener("resize", measureHeight);
    }

    const observer = new ResizeObserver(([entry]) => {
      const width = entry?.contentRect.width ?? 0;
      if (width === previousWidth) return;
      previousWidth = width;
      measureHeight();
    });
    observer.observe(textarea);
    return () => observer.disconnect();
  }, [measureHeight]);

  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    const end = textarea.value.length;
    textarea.focus();
    textarea.setSelectionRange(end, end);
    measureHeight();
    return () => {
      if (scrollFrameRef.current !== null) cancelAnimationFrame(scrollFrameRef.current);
    };
  }, [measureHeight]);

  const handleKeyDown = useCallback((event: React.KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onCancel();
      return;
    }
    if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
      event.preventDefault();
      onSave(content);
    }
  }, [onCancel, onSave, content]);

  return (
    <div className="chat-column-surface em-root">
      <div className="em-surface">
        <textarea
          ref={textareaRef}
          className={`em-textarea${overflowing ? " em-textarea-scroll" : ""}`}
          value={content}
          rows={USER_MESSAGE_EDIT_MIN_LINES}
          aria-label={t("agentLocal.editMessage")}
          onChange={(event) => setContent(event.target.value)}
          onKeyDown={handleKeyDown}
          style={textHeight ? { height: `${textHeight}px` } : undefined}
        />
        <div className="em-actions">
          <button className="btn btn-sm btn-ghost em-cancel" type="button" onClick={onCancel}>
            {t("agentLocal.cancel")}
          </button>
          <button className="btn btn-sm btn-primary" type="button" onClick={() => onSave(content)}>
            {t("agentLocal.send")}
          </button>
        </div>
      </div>
    </div>
  );
}
