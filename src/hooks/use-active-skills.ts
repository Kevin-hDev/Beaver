import { useState, useCallback, useRef } from "react";
import type { SkillInfo } from "@/types/agent";
import type { useSlashCommands } from "@/hooks/use-slash-commands";
import type { SlashItem } from "@/hooks/use-slash-commands";
import { activeSkillsInText, replaceSlashToken } from "@/lib/skill-text";

export interface ActiveSkillsState {
  activeSkills: SkillInfo[];
  skillContentsRef: React.RefObject<Map<string, string>>;
  handleSelectSkill: (item: SlashItem) => Promise<void>;
  getSkillsPayload: () => { name: string; content: string }[] | undefined;
  clearSkills: () => void;
}

export function useActiveSkills(
  slash: ReturnType<typeof useSlashCommands>,
  text: string,
  setText: (v: string) => void,
): ActiveSkillsState {
  const [activeSkills, setActiveSkills] = useState<SkillInfo[]>([]);
  const skillContentsRef = useRef<Map<string, string>>(new Map());

  const handleSelectSkill = useCallback(async (item: SlashItem) => {
    const result = await slash.selectItem(item);
    if (!result) return;

    if ("builtIn" in result) {
      setText("/" + result.builtIn.name);
      return;
    }

    const { skill, content } = result;
    if (!activeSkills.some((s) => s.id === skill.id)) {
      setActiveSkills((prev) => [...prev, skill]);
    }
    skillContentsRef.current.set(skill.id, content);
    setText(replaceSlashToken(text, skill.command));
  }, [slash, text, setText, activeSkills]);

  const getSkillsPayload = useCallback(() => {
    const visibleSkills = activeSkillsInText(text, activeSkills);
    if (visibleSkills.length === 0) return undefined;
    return visibleSkills.map((s) => ({
      name: s.command,
      content: skillContentsRef.current.get(s.id) ?? "",
    }));
  }, [activeSkills, text]);

  const clearSkills = useCallback(() => {
    setActiveSkills([]);
    skillContentsRef.current.clear();
  }, []);

  return {
    activeSkills,
    skillContentsRef,
    handleSelectSkill,
    getSkillsPayload,
    clearSkills,
  };
}
