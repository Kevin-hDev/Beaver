import { useCallback, useSyncExternalStore } from "react";
import type { SkillInfo } from "@/types/agent";

const MAX_DRAFTS = 64;
const MAX_SKILLS_PER_DRAFT = 16;
const MAX_LISTENERS = 64;

export interface ComposerDraftSkill {
  info: SkillInfo;
  content: string;
}

interface ComposerDraft {
  text: string;
  skills: ComposerDraftSkill[];
}

const EMPTY_DRAFT: ComposerDraft = { text: "", skills: [] };

export const WELCOME_COMPOSER_DRAFT_KEY = "welcome";

/* Les brouillons restent volontairement dans la mémoire du renderer : ils
   survivent à la navigation sans écrire du texte potentiellement sensible sur
   disque. Les deux collections sont bornées pour garder ce cache prévisible. */
let drafts = new Map<string, ComposerDraft>();
let nextListenerId = 1;
const listeners = new Map<number, () => void>();

export function sessionComposerDraftKey(sessionId: string): string {
  return `session:${sessionId}`;
}

export function useComposerDraft(draftKey: string) {
  const subscribeToDrafts = useCallback(
    (listener: () => void) => subscribe(listener),
    [],
  );
  const readDraft = useCallback(() => drafts.get(draftKey) ?? EMPTY_DRAFT, [draftKey]);
  const draft = useSyncExternalStore(subscribeToDrafts, readDraft, readDraft);

  const setText = useCallback((next: string) => {
    updateComposerDraft(draftKey, (current) => ({
      text: next,
      skills: next.length === 0 ? [] : current.skills,
    }));
  }, [draftKey]);
  const rememberSkill = useCallback((info: SkillInfo, content: string) => {
    updateComposerDraft(draftKey, (current) => {
      const skills = current.skills.filter((entry) => entry.info.id !== info.id);
      skills.push({ info, content });
      return { ...current, skills: skills.slice(-MAX_SKILLS_PER_DRAFT) };
    });
  }, [draftKey]);
  const clear = useCallback(() => {
    clearComposerDraft(draftKey);
  }, [draftKey]);

  return { ...draft, setText, rememberSkill, clear };
}

export function clearComposerDraft(draftKey: string) {
  if (!drafts.has(draftKey)) return;
  const next = new Map(drafts);
  next.delete(draftKey);
  drafts = next;
  notify();
}

function updateComposerDraft(
  draftKey: string,
  update: (current: ComposerDraft) => ComposerDraft,
) {
  const draft = update(drafts.get(draftKey) ?? EMPTY_DRAFT);
  if (draft.text.length === 0 && draft.skills.length === 0) {
    clearComposerDraft(draftKey);
    return;
  }
  const next = new Map(drafts);
  next.delete(draftKey);
  next.set(draftKey, draft);
  while (next.size > MAX_DRAFTS) {
    const oldest = next.keys().next().value;
    if (oldest === undefined) break;
    next.delete(oldest);
  }
  drafts = next;
  notify();
}

function subscribe(listener: () => void): () => void {
  while (listeners.size >= MAX_LISTENERS) {
    const oldest = listeners.keys().next().value;
    if (oldest === undefined) break;
    listeners.delete(oldest);
  }
  const id = nextListenerId++;
  listeners.set(id, listener);
  return () => listeners.delete(id);
}

function notify() {
  for (const listener of listeners.values()) listener();
}
