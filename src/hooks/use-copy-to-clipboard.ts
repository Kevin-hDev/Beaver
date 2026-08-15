import { useCallback, useEffect, useRef, useState } from "react";

/* Durée pendant laquelle une confirmation de copie reste lisible. Assez pour
   être vue, assez court pour que la commande redevienne disponible sans geste. */
const FEEDBACK_MS = 2000;

export type CopyState = "idle" | "copied" | "error";

/* Copie un texte et expose l'issue de l'opération. L'échec est un état à part
   entière, jamais avalé : le presse-papiers peut être refusé par le système, et
   annoncer « copié » dans ce cas ferait coller autre chose à l'utilisateur. */
export function useCopyToClipboard() {
  const [state, setState] = useState<CopyState>("idle");
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  useEffect(() => () => clearTimeout(timerRef.current), []);

  const reset = useCallback(() => {
    clearTimeout(timerRef.current);
    setState("idle");
  }, []);

  const copy = useCallback(async (text: string) => {
    clearTimeout(timerRef.current);
    try {
      await navigator.clipboard.writeText(text);
      setState("copied");
    } catch {
      setState("error");
    }
    timerRef.current = setTimeout(() => setState("idle"), FEEDBACK_MS);
  }, []);

  return { state, copy, reset };
}
