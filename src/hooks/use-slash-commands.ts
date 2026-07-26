import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SkillInfo } from "@/types/agent";
import { useFsEvent } from "@/hooks/use-fs-event";

export interface BuiltInCommand {
  name: string;
  description: string;
  source: "built-in";
  path: string;
}

/**
 * Find the index of the slash that starts an in-progress slash command at or
 * before the cursor. Returns -1 when none qualifies.
 *
 * Rules:
 *  - The slash itself may be at the cursor (just typed "/") or earlier when
 *    the cursor sits inside the filter part (e.g. "/hk|" or "/hk-d|ev").
 *  - The slash must be at the start of the text or preceded by whitespace so
 *    we don't treat a "/" inside a path or word as a command trigger.
 *  - Between the slash and the cursor there may be only command characters
 *    (no spaces, no other slash) — otherwise the command is already closed.
 */
function findSlashBeforeCursor(text: string, cursorPos: number): number {
  // Search backwards from the cursor for a candidate slash.
  for (let i = Math.min(cursorPos - 1, text.length - 1); i >= 0; i -= 1) {
    const ch = text[i];
    if (ch === " " || ch === "\n" || ch === "\r" || ch === "\t") return -1;
    if (ch !== "/") continue;
    // Found a slash. It is a command starter only if at start of text or
    // preceded by whitespace.
    const before = i === 0 ? "" : text[i - 1];
    const isWordStart = before === "" || before === " " || before === "\n" || before === "\r" || before === "\t";
    return isWordStart ? i : -1;
  }
  return -1;
}

const BUILT_IN_COMMANDS: BuiltInCommand[] = [
  {
    name: "compress",
    description: "Compress conversation context manually",
    source: "built-in",
    path: "__built-in__/compress",
  },
];

export type SlashItem = SkillInfo | BuiltInCommand;

export function isBuiltIn(item: SlashItem): item is BuiltInCommand {
  return item.source === "built-in";
}

export function useSlashCommands() {
  const [skills, setSkills] = useState<SkillInfo[]>([]);
  const [showDropdown, setShowDropdown] = useState(false);
  const [filter, setFilter] = useState("");
  const [activeIndex, setActiveIndex] = useState(0);

  const loadSkills = useCallback(() => {
    invoke<SkillInfo[]>("list_skills")
      .then(setSkills)
      .catch(() => console.warn("Erreur chargement skills"));
  }, []);

  useEffect(() => { loadSkills(); }, [loadSkills]);

  useFsEvent("fs:skills-changed", loadSkills);

  const allItems: SlashItem[] = filter
    ? [
        ...BUILT_IN_COMMANDS.filter(
          (c) =>
            c.name.toLowerCase().includes(filter.toLowerCase()) ||
            c.description.toLowerCase().includes(filter.toLowerCase()),
        ),
        ...skills.filter(
          (s) =>
            s.name.toLowerCase().includes(filter.toLowerCase()) ||
            s.source_name.toLowerCase().includes(filter.toLowerCase()) ||
            s.description.toLowerCase().includes(filter.toLowerCase()),
        ),
      ]
    : [...BUILT_IN_COMMANDS, ...skills];

  /**
   * Detect slash command at the cursor position.
   *
   * `cursorPos` (0-based, in the same units as `text.length`) lets us detect a
   * slash BEFORE existing text (e.g. typing "/" in front of "hello"). When
   * omitted, the legacy fallback uses the last slash in the whole text, which
   * misfires when the slash precedes unrelated text.
   */
  const handleInput = useCallback((text: string, cursorPos?: number) => {
    const pos = cursorPos ?? text.length;
    // Walk backwards from the cursor to find an unambiguous slash command.
    // The slash must be at `pos` (just typed) or earlier if the cursor is
    // still inside the token (typing the filter).
    const slashIndex = findSlashBeforeCursor(text, pos);
    if (slashIndex < 0) {
      setShowDropdown(false);
      setFilter("");
      return;
    }
    const afterSlash = text.slice(slashIndex + 1, pos);
    if (afterSlash.includes(" ") || afterSlash.includes("/")) {
      setShowDropdown(false);
      setFilter("");
      return;
    }
    setShowDropdown(true);
    setFilter(afterSlash);
    setActiveIndex(0);
  }, []);

  const selectSkill = useCallback(async (skill: SkillInfo) => {
    setShowDropdown(false);
    setFilter("");
    setActiveIndex(0);
    try {
      const content = await invoke<string>("load_skill", { skillId: skill.id });
      return { content, skill };
    } catch {
      console.warn("Erreur chargement skill");
      return null;
    }
  }, []);

  const selectItem = useCallback(
    async (item: SlashItem): Promise<{ content: string; skill: SkillInfo } | { builtIn: BuiltInCommand } | null> => {
      setShowDropdown(false);
      setFilter("");
      setActiveIndex(0);
      if (isBuiltIn(item)) {
        return { builtIn: item };
      }
      try {
        const content = await invoke<string>("load_skill", { skillId: item.id });
        return { content, skill: item };
      } catch {
        console.warn("Erreur chargement skill");
        return null;
      }
    },
    [],
  );

  const moveUp = useCallback(() => {
    setActiveIndex((i) => (i > 0 ? i - 1 : allItems.length - 1));
  }, [allItems.length]);

  const moveDown = useCallback(() => {
    setActiveIndex((i) => (i < allItems.length - 1 ? i + 1 : 0));
  }, [allItems.length]);

  const close = useCallback(() => {
    setShowDropdown(false);
    setFilter("");
    setActiveIndex(0);
  }, []);

  return {
    skills: allItems,
    showDropdown,
    activeIndex,
    handleInput,
    selectSkill,
    selectItem,
    moveUp,
    moveDown,
    close,
  };
}
