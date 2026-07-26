import type React from "react";
import type { SkillInfo } from "@/types/agent";
import type { SkillTokenSource } from "@/components/agent-local/skill-chip-ranges";
import { MagicWandIcon } from "@/components/agent-local/skill-icons";

export function replaceSlashToken(text: string, skillName: string): string {
  const lastSlash = text.lastIndexOf("/");
  if (lastSlash < 0) return `${text}/${skillName}`;
  const before = text.slice(0, lastSlash);
  const after = text.slice(lastSlash);
  const trailing = after.match(/^\/[^\s/]*/)?.[0] ?? "";
  return `${before}/${skillName}${after.slice(trailing.length)}`;
}

export function activeSkillsInText(
  text: string,
  skills: SkillInfo[],
): SkillInfo[] {
  const activeCommands = new Set(
    splitSkillText(text, skills.map((skill) => skill.command))
      .filter((part) => part.kind === "skill")
      .map((part) => part.text.slice(1)),
  );
  return skills.filter((skill) => activeCommands.has(skill.command));
}

export interface ChatChipOptions {
  /** Optional built-in command names rendered with a distinct icon. */
  builtInNames?: string[];
}

/**
 * Render skill (and optionally built-in) chips inside the chat messages.
 *
 * Pure React render: no caret to align, so each "/name" token becomes an
 * inline chip directly.
 */
export function highlightSkillNodes(
  nodes: React.ReactNode[],
  skillNames: string[] | undefined,
  options?: ChatChipOptions,
): React.ReactNode[] {
  const all = collectChipNames(skillNames, options);
  if (all.length < 1) return nodes;

  const builtInSet = new Set(options?.builtInNames ?? []);
  const highlighted: React.ReactNode[] = [];
  nodes.forEach((node, nodeIndex) => {
    if (typeof node !== "string") {
      highlighted.push(node);
      return;
    }
    splitSkillText(node, all).forEach((part, partIndex) => {
      if (part.kind !== "skill") {
        highlighted.push(part.text);
        return;
      }
      const name = part.text.startsWith("/") ? part.text.slice(1) : part.text;
      const source: SkillTokenSource = builtInSet.has(name) ? "built-in" : "skill";
      highlighted.push(renderChatChip(name, source, `${nodeIndex}-${partIndex}`));
    });
  });
  return highlighted;
}

function collectChipNames(
  skillNames: string[] | undefined,
  options?: ChatChipOptions,
): string[] {
  const names = [...(skillNames ?? []), ...(options?.builtInNames ?? [])];
  return names.filter(Boolean);
}

function renderChatChip(name: string, source: SkillTokenSource, key: React.Key) {
  const Icon = source === "built-in" ? ClockIcon : MagicWandIcon;
  return (
    <span key={key} className={`skill-chip${source === "built-in" ? " skill-chip-built-in" : ""}`}>
      <Icon className="skill-chip-icon" />
      <span className="skill-chip-name">{name}</span>
    </span>
  );
}

function ClockIcon({ className }: { className?: string }) {
  return (
    <svg className={className} viewBox="0 0 256 256" fill="currentColor" aria-hidden="true">
      <path d="M128,24A104,104,0,1,0,232,128,104.11,104.11,0,0,0,128,24Zm0,192a88,88,0,1,1,88-88A88.1,88.1,0,0,1,128,216Zm64-88a8,8,0,0,1-8,8H128a8,8,0,0,1-8-8V72a8,8,0,0,1,16,0v48h48A8,8,0,0,1,192,128Z" />
    </svg>
  );
}

export function splitSkillText(text: string, skillNames: string[]) {
  const names = skillNames.filter(Boolean).sort((a, b) => b.length - a.length);
  if (names.length < 1) return [{ kind: "text" as const, text }];

  const parts: Array<{ kind: "text" | "skill"; text: string }> = [];
  let lastIndex = 0;

  for (let index = 0; index < text.length; index += 1) {
    if (text[index] !== "/" || !isTokenStart(text, index)) continue;
    const name = matchingSkillName(text, index + 1, names);
    if (!name) continue;

    const end = index + 1 + name.length;
    if (!isTokenEnd(text, end)) continue;

    if (index > lastIndex) {
      parts.push({ kind: "text", text: text.slice(lastIndex, index) });
    }
    parts.push({ kind: "skill", text: text.slice(index, end) });
    lastIndex = end;
    index = end - 1;
  }

  if (lastIndex < text.length) {
    parts.push({ kind: "text", text: text.slice(lastIndex) });
  }
  return parts.length > 0 ? parts : [{ kind: "text", text }];
}

function matchingSkillName(text: string, start: number, skillNames: string[]): string | null {
  for (const name of skillNames) {
    if (text.startsWith(name, start)) return name;
  }
  return null;
}

function isTokenStart(text: string, index: number): boolean {
  return index === 0 || isWhitespace(text[index - 1]);
}

function isTokenEnd(text: string, index: number): boolean {
  if (index >= text.length) return true;
  const char = text[index];
  return isWhitespace(char) || char === "." || char === "," || char === ";"
    || char === ":" || char === "!" || char === "?";
}

function isWhitespace(char: string | undefined): boolean {
  return char === " " || char === "\n" || char === "\r" || char === "\t";
}
