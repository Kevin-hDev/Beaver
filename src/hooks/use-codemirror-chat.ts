/**
 * CodeMirror 6 hook for the chat input.
 *
 * Controlled editor: React owns the value, CM6 owns the caret/selection.
 * A guard prevents the React→CM sync from echoing CM's own updates back.
 *
 * Three Compartments allow runtime reconfiguration without remounting:
 *  - `readOnlyComp`   : toggles EditorState.readOnly / EditorView.editable
 *  - `placeholderComp` : swaps the placeholder facet
 *  - `chipComp`        : rebuilds the skill-chip extension when names change
 *
 * Auto-resize is built in: a ResizeObserver measures the content scroller and
 * clamps the host height to `maxHeight`.
 *
 * Keyboard behaviour is delegated to `onKeyEvent` so the parent decides
 * Enter (send), Escape (stop), and arrow navigation for the slash dropdown.
 * IME composition is tracked so Enter never fires mid-composition.
 */

import { useCallback, useEffect, useRef, useState } from "react";
import { EditorView, keymap, placeholder as cmPlaceholder } from "@codemirror/view";
import {
  Annotation,
  Compartment,
  EditorSelection,
  EditorState,
} from "@codemirror/state";
import { history, defaultKeymap, historyKeymap } from "@codemirror/commands";

import { skillChipExtension, type SkillChipConfig } from "@/components/agent-local/skill-chip-extension";
import { markdownInputExtension } from "@/components/agent-local/markdown-input-extension";
import {
  chatEditorTheme,
  createChatDomEventHandlers,
} from "./chat-editor-behavior";

const REACT_VALUE_SYNC = Annotation.define<boolean>();

interface UseCodemirrorChatOptions {
  value: string;
  placeholder: string;
  readOnly: boolean;
  chipConfig: SkillChipConfig;
  /** Called whenever the document text or selection changes from inside CM. */
  onChange: (value: string, cursorPos: number) => void;
  /** Raw keydown forwarded from CM. Return `true` to stop CM's own handling. */
  onKeyEvent?: (event: KeyboardEvent) => boolean | void;
  /** Max editor height before internal scroll kicks in. */
  maxHeight?: number;
}

export function useCodemirrorChat({
  value,
  placeholder,
  readOnly,
  chipConfig,
  onChange,
  onKeyEvent,
  maxHeight = 200,
}: UseCodemirrorChatOptions) {
  const hostRef = useRef<HTMLDivElement | null>(null);
  const viewRef = useRef<EditorView | null>(null);

  // Compartments allow runtime reconfiguration without remount. They are
  // created once (useState initializer, never re-created) and read inside
  // effects/handlers, so React 19's strict render-time ref rules are honoured.
  const [readOnlyComp] = useState(() => new Compartment());
  const [placeholderComp] = useState(() => new Compartment());
  const [chipComp] = useState(() => new Compartment());

  // Live refs so handlers always see the latest props.
  // Updated in an effect (not during render) to comply with React 19's
  // strict ref-mutation rules.
  const onChangeRef = useRef(onChange);
  const onKeyEventRef = useRef(onKeyEvent);
  useEffect(() => {
    onChangeRef.current = onChange;
    onKeyEventRef.current = onKeyEvent;
  }, [onChange, onKeyEvent]);

  // IME composition guard: Enter must not send while composing.
  const composingRef = useRef(false);

  // Auto-resize: clamp the host height to the natural content height.
  // The scroller only reports its own content; the host adds vertical padding
  // (border-box), so we must add it back to size the host correctly.
  const resize = useCallback(() => {
    const host = hostRef.current;
    const view = viewRef.current;
    if (!host || !view) return;
    const scroller = view.scrollDOM;
    if (!scroller) return;
    host.style.height = "auto";
    const style = getComputedStyle(host);
    const padTop = parseFloat(style.paddingTop) || 0;
    const padBottom = parseFloat(style.paddingBottom) || 0;
    const natural = scroller.scrollHeight + padTop + padBottom;
    host.style.height = `${Math.min(natural, maxHeight)}px`;
  }, [maxHeight]);
  const resizeRef = useRef(resize);
  useEffect(() => {
    resizeRef.current = resize;
  }, [resize]);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    const keyHandler = createChatDomEventHandlers(onKeyEventRef, composingRef);

    const view = new EditorView({
      state: EditorState.create({
        doc: value,
        extensions: [
          EditorView.lineWrapping,
          markdownInputExtension(),
          history(),
          keymap.of([...defaultKeymap, ...historyKeymap]),
          placeholderComp.of(cmPlaceholder(placeholder)),
          chipComp.of(skillChipExtension(chipConfig)),
          keyHandler,
          readOnlyComp.of([
            EditorState.readOnly.of(readOnly),
            EditorView.editable.of(!readOnly),
          ]),
          EditorView.updateListener.of((update) => {
            if (!update.docChanged && !update.selectionSet) return;
            const isReactSync = update.transactions.some(
              (transaction) => transaction.annotation(REACT_VALUE_SYNC),
            );
            if (!isReactSync) {
              onChangeRef.current(
                update.state.doc.toString(),
                update.state.selection.main.head,
              );
            }
            requestAnimationFrame(() => resizeRef.current());
          }),
          chatEditorTheme,
        ],
      }),
      parent: host,
    });

    viewRef.current = view;
    requestAnimationFrame(() => resizeRef.current());

    // ResizeObserver catches wrapping changes (viewport width, bubble width).
    const observer = new ResizeObserver(() => resizeRef.current());
    observer.observe(host);

    return () => {
      observer.disconnect();
      view.destroy();
      viewRef.current = null;
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Sync React value → CM (skip if identical to avoid clobbering caret).
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    const current = view.state.doc.toString();
    if (current === value) return;
    view.dispatch({
      changes: { from: 0, to: current.length, insert: value },
      annotations: REACT_VALUE_SYNC.of(true),
    });
    requestAnimationFrame(resize);
  }, [value, resize]);

  // Reconfigure placeholder without remounting.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({ effects: placeholderComp.reconfigure(cmPlaceholder(placeholder)) });
  }, [placeholder, placeholderComp]);

  // Toggle readOnly without remounting.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: readOnlyComp.reconfigure([
        EditorState.readOnly.of(readOnly),
        EditorView.editable.of(!readOnly),
      ]),
    });
  }, [readOnly, readOnlyComp]);

  // Reconfigure chips when skill/built-in names change.
  useEffect(() => {
    const view = viewRef.current;
    if (!view) return;
    view.dispatch({
      effects: chipComp.reconfigure(skillChipExtension(chipConfig)),
    });
  }, [chipConfig, chipComp]);

  return {
    hostRef,
    viewRef,
    composingRef,
    focus: () => {
      const view = viewRef.current;
      if (!view) return;
      view.focus();
      const end = view.state.doc.length;
      view.dispatch({ selection: EditorSelection.cursor(end) });
    },
  };
}
