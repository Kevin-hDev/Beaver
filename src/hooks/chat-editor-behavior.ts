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
  /* Pas de hauteur imposée : l'éditeur grandit avec son texte jusqu'au plafond
     posé sur `.cm-editor` dans chat-input-textarea.css. */
  "&": { backgroundColor: "transparent" },
  ".cm-scroller": {
    overflowX: "hidden",
    overflowY: "auto",
    overscrollBehavior: "contain",
  },
});

/* Le reste de l'apparence — marges intérieures, curseur, contour au focus —
   vit dans chat-input-textarea.css. Déclaré ici en plus, il entrerait en
   concurrence avec ces règles, et laquelle l'emporte dépendrait de l'ordre de
   chargement des feuilles. */
