import { markdown, markdownLanguage } from "@codemirror/lang-markdown";
import { syntaxTree } from "@codemirror/language";
import { RangeSetBuilder, type EditorState, type Extension } from "@codemirror/state";
import {
  Decoration,
  EditorView,
  WidgetType,
  type DecorationSet,
} from "@codemirror/view";

const BULLET_MARK_PATTERN = /^[-+*]$/;

interface MarkdownBulletRange {
  from: number;
  to: number;
}

class MarkdownBulletWidget extends WidgetType {
  eq(): boolean {
    return true;
  }

  toDOM(): HTMLElement {
    const bullet = document.createElement("span");
    bullet.className = "chat-md-input-bullet";
    bullet.textContent = "•";
    return bullet;
  }

  ignoreEvent(): boolean {
    return false;
  }
}

type MarkdownTree = ReturnType<typeof markdownLanguage.parser.parse>;

function collectBulletRanges(
  tree: MarkdownTree,
  sliceText: (from: number, to: number) => string,
): MarkdownBulletRange[] {
  const ranges: MarkdownBulletRange[] = [];
  tree.iterate({
    enter(node) {
      if (node.name !== "ListMark") return;
      const marker = sliceText(node.from, node.to);
      if (!BULLET_MARK_PATTERN.test(marker)) return;
      ranges.push({ from: node.from, to: node.to });
    },
  });
  return ranges;
}

export function findMarkdownBulletRanges(text: string): MarkdownBulletRange[] {
  return collectBulletRanges(
    markdownLanguage.parser.parse(text),
    (from, to) => text.slice(from, to),
  );
}

function buildBulletDecorations(state: EditorState): DecorationSet {
  const builder = new RangeSetBuilder<Decoration>();
  const ranges = collectBulletRanges(
    syntaxTree(state),
    (from, to) => state.doc.sliceString(from, to),
  );
  for (const range of ranges) {
    builder.add(
      range.from,
      range.to,
      Decoration.replace({ widget: new MarkdownBulletWidget() }),
    );
  }
  return builder.finish();
}

const bulletDecorations = EditorView.decorations.compute(
  ["doc"],
  buildBulletDecorations,
);

export function markdownInputExtension(): Extension {
  return [
    markdown({
      base: markdownLanguage,
      addKeymap: false,
      completeHTMLTags: false,
      pasteURLAsLink: false,
    }),
    bulletDecorations,
  ];
}
