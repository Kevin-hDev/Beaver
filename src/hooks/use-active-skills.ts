import { useCallback } from "react";
import type { SkillInfo } from "@/types/agent";
import type { useSlashCommands } from "@/hooks/use-slash-commands";
import type { SlashItem } from "@/hooks/use-slash-commands";
import { activeSkillsInText, replaceSlashToken } from "@/lib/skill-text";
import type { ComposerDraftSkill } from "@/hooks/use-composer-draft";

export interface ActiveSkillsState {
  activeSkills: SkillInfo[];
  handleSelectSkill: (item: SlashItem) => Promise<void>;
  getSkillsPayload: () => { name: string; content: string }[] | undefined;
}

export function useActiveSkills(
  slash: ReturnType<typeof useSlashCommands>,
  text: string,
  setText: (v: string) => void,
  draftSkills: ComposerDraftSkill[],
  rememberSkill: (skill: SkillInfo, content: string) => void,
): ActiveSkillsState {
  const activeSkills = draftSkills.map((entry) => entry.info);

  const handleSelectSkill = useCallback(async (item: SlashItem) => {
    const result = await slash.selectItem(item);
    if (!result) return;

    if ("builtIn" in result) {
      setText("/" + result.builtIn.name);
      return;
    }

    const { skill, content } = result;
    rememberSkill(skill, content);
    setText(replaceSlashToken(text, skill.command));
  }, [slash, text, setText, rememberSkill]);

  const getSkillsPayload = useCallback(() => {
    const visibleSkills = activeSkillsInText(text, activeSkills);
    if (visibleSkills.length === 0) return undefined;
    return visibleSkills.map((s) => ({
      name: s.command,
      content: draftSkills.find((entry) => entry.info.id === s.id)?.content ?? "",
    }));
  }, [activeSkills, draftSkills, text]);

  return {
    activeSkills,
    handleSelectSkill,
    getSkillsPayload,
  };
}
