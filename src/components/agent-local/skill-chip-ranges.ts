/**
 * Pure detection of slash token ranges in a document.
 *
 * Shared between the CodeMirror extension (chat input) and the static renderer
 * (chat messages). Both rely on `splitSkillText` for the actual parsing.
 */

import { splitSkillText } from "@/lib/skill-text";

export type SkillTokenSource = "skill" | "built-in";

export interface SkillTokenRange {
  /** Absolute start offset (inclusive) of the "/name" token in the document. */
  from: number;
  /** Absolute end offset (exclusive). */
  to: number;
  /** Skill name without the leading slash. */
  name: string;
  /** Full "/name" text as it appears in the document. */
  raw: string;
  source: SkillTokenSource;
}

/**
 * Scan `text` and return every slash token (skill or built-in) with its
 * absolute range. Built-in names take precedence over skills when both match
 * (a built-in can never be shadowed by a user skill of the same name).
 */
export function findSkillTokenRanges(
  text: string,
  skillNames: string[],
  builtInNames: string[],
): SkillTokenRange[] {
  const all = [...skillNames, ...builtInNames];
  if (all.length === 0) return [];

  const builtInSet = new Set(builtInNames);
  const parts = splitSkillText(text, all);
  const ranges: SkillTokenRange[] = [];

  let offset = 0;
  for (const part of parts) {
    if (part.kind === "skill") {
      const name = part.text.startsWith("/") ? part.text.slice(1) : part.text;
      const source: SkillTokenSource = builtInSet.has(name) ? "built-in" : "skill";
      ranges.push({
        from: offset,
        to: offset + part.text.length,
        name,
        raw: part.text,
        source,
      });
    }
    offset += part.text.length;
  }
  return ranges;
}
