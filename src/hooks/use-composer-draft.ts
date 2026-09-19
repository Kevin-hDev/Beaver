import { useCallback, useEffect, useSyncExternalStore } from "react";
import type { SkillInfo } from "@/types/agent";
import {
  MAX_SKILLS_PER_DRAFT,
  clearComposerDraft,
  openComposerDraft,
  readComposerDraft,
  subscribeComposerDrafts,
  updateComposerDraft,
  type ComposerDraftSnapshot,
  type ComposerDraftSkill,
} from "./composer-draft-store";

export type { ComposerDraftSkill };
export function sessionComposerDraftKey(sessionId: string): string { return `session:${sessionId}`; }

export function useComposerDraft(draftKey: string) {
  useEffect(() => { openComposerDraft(draftKey); }, [draftKey]);
  const read = useCallback(() => readComposerDraft(draftKey), [draftKey]);
  const draft = useSyncExternalStore(subscribeComposerDrafts, read, read);
  const setText = useCallback((text: string) => {
    updateComposerDraft(draftKey, (current) => ({
      ...current, text, skills: text.length === 0 ? [] : current.skills,
    }));
  }, [draftKey]);
  const rememberSkill = useCallback((info: SkillInfo, content: string) => {
    updateComposerDraft(draftKey, (current) => ({
      ...current,
      skills: [...current.skills.filter((entry) => entry.info.id !== info.id), { info, content }]
        .slice(-MAX_SKILLS_PER_DRAFT),
    }));
  }, [draftKey]);
  const consume = useCallback((expected: ComposerDraftSnapshot) => {
    updateComposerDraft(draftKey, (current) => ({
      ...current,
      text: current.text === expected.text ? "" : current.text,
      skills: sameSkills(current.skills, expected.skills) ? [] : current.skills,
    }));
  }, [draftKey]);
  const restore = useCallback((submitted: ComposerDraftSnapshot) => {
    updateComposerDraft(draftKey, (current) => ({
      ...current,
      text: current.text.length === 0 ? submitted.text : current.text,
      skills: current.skills.length === 0 ? submitted.skills : current.skills,
    }));
  }, [draftKey]);
  const clear = useCallback(() => clearComposerDraft(draftKey), [draftKey]);
  return { ...draft, setText, rememberSkill, consume, restore, clear };
}

function sameSkills(current: ComposerDraftSkill[], expected: ComposerDraftSkill[]): boolean {
  return current.length === expected.length && current.every((entry, index) => {
    const sent = expected[index];
    return sent?.info.id === entry.info.id && sent.content === entry.content;
  });
}

export { clearComposerDraft } from "./composer-draft-store";
export { WELCOME_COMPOSER_DRAFT_KEY } from "./composer-draft-store";
