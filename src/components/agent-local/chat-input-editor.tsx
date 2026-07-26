/**
 * CodeMirror 6 chat editor.
 *
 * Renders the chips for slash tokens and forwards every keydown to the parent
 * so it can implement Enter (send), Escape (stop) and slash-dropdown navigation.
 */

import { memo } from "react";
import { useCodemirrorChat } from "@/hooks/use-codemirror-chat";
import { activeSkillsInText } from "@/lib/skill-text";
import type { SkillInfo } from "@/types/agent";
import type { SkillChipConfig } from "./skill-chip-extension";

const BUILT_IN_NAMES = ["compress"];

interface ChatInputEditorProps {
  value: string;
  placeholder: string;
  readOnly: boolean;
  activeSkills: SkillInfo[];
  onTextChange: (value: string, cursorPos: number) => void;
  onKeyEvent: (event: KeyboardEvent) => boolean | void;
}

function ChatInputEditorImpl({
  value,
  placeholder,
  readOnly,
  activeSkills,
  onTextChange,
  onKeyEvent,
}: ChatInputEditorProps) {
  const skillNames = activeSkillsInText(value, activeSkills).map((s) => s.name);
  const chipConfig: SkillChipConfig = {
    skillNames,
    builtInNames: BUILT_IN_NAMES,
  };

  const { hostRef } = useCodemirrorChat({
    value,
    placeholder,
    readOnly,
    chipConfig,
    onChange: onTextChange,
    onKeyEvent: onKeyEvent,
  });

  return (
    <div className="chat-textarea-shell">
      <div className="chat-cm-host" ref={hostRef} data-keyboard-scope="local" />
    </div>
  );
}

export const ChatInputEditor = memo(ChatInputEditorImpl);
