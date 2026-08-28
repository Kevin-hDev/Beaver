/**
 * CodeMirror 6 chat editor.
 *
 * Controlled editor: React owns the value, CM6 owns the caret/selection.
 * A guard prevents the React→CM sync from echoing CM's own updates back.
 *
 * Three Compartments allow runtime reconfiguration without remounting:
 *  - `readOnlyComp`   : toggles EditorState.readOnly / EditorView.editable
 *  - `placeholderComp` : swaps the placeholder facet
 *  - `chipComp`        : rebuilds the skill-chip extension when names change
 *
 * La hauteur n'est pas calculée ici : l'éditeur grandit avec son texte et
 * s'arrête au plafond posé en CSS (`.cm-editor { max-height }`), qui en est
 * l'unique autorité. Une mesure en JavaScript devait pour mesurer remettre la
 * hauteur à « auto », et ce passage transitoire écrasait la position de
 * défilement : la dernière ligne écrite restait sous le bord du champ.
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
}

export function useCodemirrorChat({
  value,
  placeholder,
  readOnly,
  chipConfig,
  onChange,
  onKeyEvent,
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
            if (isReactSync) return;
            onChangeRef.current(
              update.state.doc.toString(),
              update.state.selection.main.head,
            );
          }),
          chatEditorTheme,
        ],
      }),
      parent: host,
    });

    viewRef.current = view;

    return () => {
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
  }, [value]);

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

  const focus = useCallback(() => {
    const view = viewRef.current;
    if (!view) return;
    view.focus();
    const end = view.state.doc.length;
    view.dispatch({ selection: EditorSelection.cursor(end) });
  }, []);

  return { hostRef, viewRef, composingRef, focus };
}
