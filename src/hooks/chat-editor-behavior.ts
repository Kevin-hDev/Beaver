import { Prec } from "@codemirror/state";
import { EditorView } from "@codemirror/view";

interface ValueRef<T> {
  current: T;
}

type KeyEventHandler = ((event: KeyboardEvent) => boolean | void) | undefined;

function isControlledChatKey(key: string): boolean {
  return key === "ArrowUp"
    || key === "ArrowDown"
    || key === "Enter"
    || key === "Escape";
}

export function createChatDomEventHandlers(
  onKeyEventRef: ValueRef<KeyEventHandler>,
  composingRef: ValueRef<boolean>,
) {
  return Prec.highest(EditorView.domEventHandlers({
    keydown: (event: KeyboardEvent) => {
      if (
        !isControlledChatKey(event.key)
        || composingRef.current
        || event.isComposing
      ) {
        return false;
      }
      return onKeyEventRef.current?.(event) ?? false;
    },
    compositionstart: () => {
      composingRef.current = true;
    },
    compositionend: () => {
      composingRef.current = false;
    },
  }));
}

export const chatEditorTheme = EditorView.theme({
  "&": { backgroundColor: "transparent", height: "100%" },
  ".cm-scroller": {
    overflowX: "hidden",
    overflowY: "auto",
    overscrollBehavior: "contain",
  },
  ".cm-content": { padding: 0, caretColor: "var(--ink)" },
  "&.cm-focused": { outline: "none" },
});
