/**
 * CodeMirror 6 extension that renders slash tokens ("/skill-name" and
 * built-in commands like "/compress") as atomic chips.
 *
 * The actual "/name" text is replaced by a widget showing an icon plus the
 * bare name (no slash, no background). The replaced range is declared atomic
 * via `EditorView.atomicRanges` so the caret, clicks, selection and backspace
 * treat it as a single block.
 */

import { Decoration, EditorView, WidgetType } from "@codemirror/view";
import type { DecorationSet } from "@codemirror/view";
import { Extension, RangeSetBuilder } from "@codemirror/state";

import {
  findSkillTokenRanges,
  type SkillTokenSource,
} from "./skill-chip-ranges";
import { buildChipIconSvg, CLOCK_PATH, MAGIC_WAND_PATH } from "./skill-chip-icons";

/**
 * Configuration for the chip extension. The names are kept in a closure so
 * they can be reconfigured at runtime by rebuilding the extension (cheap:
 * decoration only, no editor remount).
 */
export interface SkillChipConfig {
  skillNames: string[];
  builtInNames: string[];
}

class SkillChipWidget extends WidgetType {
  constructor(
    readonly name: string,
    readonly source: SkillTokenSource,
  ) {
    super();
  }

  eq(other: SkillChipWidget): boolean {
    return other.name === this.name && other.source === this.source;
  }

  toDOM(): HTMLElement {
    const wrap = document.createElement("span");
    wrap.className = "skill-chip";
    if (this.source === "built-in") wrap.classList.add("skill-chip-built-in");

    const path = this.source === "built-in" ? CLOCK_PATH : MAGIC_WAND_PATH;
    wrap.appendChild(buildChipIconSvg(path));

    const name = document.createElement("span");
    name.className = "skill-chip-name";
    name.textContent = this.name;
    wrap.appendChild(name);
    return wrap;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

function buildReplaceDecorations(
  text: string,
  config: SkillChipConfig,
): DecorationSet {
  const ranges = findSkillTokenRanges(text, config.skillNames, config.builtInNames);
  if (ranges.length === 0) return Decoration.none;

  const builder = new RangeSetBuilder<Decoration>();
  for (const range of ranges) {
    builder.add(
      range.from,
      range.to,
      Decoration.replace({ widget: new SkillChipWidget(range.name, range.source) }),
    );
  }
  return builder.finish();
}

/**
 * Build an atomic range set marking every chip range. The value type is
 * `Decoration` to satisfy `RangeSetBuilder`'s `RangeValue` constraint; the
 * actual value is irrelevant — only the ranges matter to `atomicRanges`.
 */
function buildAtomicRanges(
  text: string,
  config: SkillChipConfig,
): DecorationSet {
  const ranges = findSkillTokenRanges(text, config.skillNames, config.builtInNames);
  if (ranges.length === 0) return Decoration.none;
  const builder = new RangeSetBuilder<Decoration>();
  for (const range of ranges) {
    builder.add(range.from, range.to, Decoration.mark({}));
  }
  return builder.finish();
}

/**
 * Build the chip extension for the given names.
 *
 * Two facets are wired:
 *  - `EditorView.decorations` renders the chips (replace the /name text)
 *  - `EditorView.atomicRanges` declares the same ranges atomic so the caret,
 *    clicks and selection jump over the whole chip in one step.
 *
 * Both facets derive from the document text, so they recompute automatically
 * on every transaction that changes the doc.
 */
export function skillChipExtension(config: SkillChipConfig): Extension {
  return [
    EditorView.decorations.of((view) =>
      buildReplaceDecorations(view.state.doc.toString(), config),
    ),
    EditorView.atomicRanges.of((view) =>
      buildAtomicRanges(view.state.doc.toString(), config),
    ),
  ];
}
