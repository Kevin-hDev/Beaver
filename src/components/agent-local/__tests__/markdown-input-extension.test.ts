import { describe, expect, it } from "vitest";
import { EditorState } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import {
  findMarkdownBulletRanges,
  markdownInputExtension,
} from "../markdown-input-extension";

describe("markdownInputExtension", () => {
  it("détecte seulement les marqueurs de listes Markdown", () => {
    const content = [
      "- Premier",
      "* Second",
      "+ Troisième",
      "1. Numéroté",
      "---",
      "# Titre",
      "```md",
      "- Exemple de code",
      "```",
    ].join("\n");

    const markers = findMarkdownBulletRanges(content)
      .map(({ from, to }) => content.slice(from, to));

    expect(markers).toEqual(["-", "*", "+"]);
  });

  it("ne modifie jamais le texte Markdown original", () => {
    const content = "- Élément\n\n---";

    findMarkdownBulletRanges(content);

    expect(content).toBe("- Élément\n\n---");
  });

  it("affiche les points sans remplacer les données de CodeMirror", () => {
    const content = "- Premier\n- Second\n\n---";
    const parent = document.createElement("div");
    const view = new EditorView({
      parent,
      state: EditorState.create({
        doc: content,
        extensions: [markdownInputExtension()],
      }),
    });

    expect(parent.querySelectorAll(".chat-md-input-bullet")).toHaveLength(2);
    expect(view.state.doc.toString()).toBe(content);

    view.destroy();
  });
});
