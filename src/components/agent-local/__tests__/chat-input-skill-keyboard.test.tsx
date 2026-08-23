import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { EditorSelection } from "@codemirror/state";
import { EditorView } from "@codemirror/view";
import { afterAll, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import type { SkillInfo } from "@/types/agent";
import { ChatInput } from "../chat-input";
import { clearComposerDraft } from "@/hooks/use-composer-draft";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: () => <div data-testid="chat-input-actions-row" />,
}));

vi.mock("../file-thumbnail", () => ({
  FileThumbnail: () => <div data-testid="file-thumbnail" />,
}));

class TestResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

const originalResizeObserver = globalThis.ResizeObserver;
const originalScrollIntoView = Object.getOwnPropertyDescriptor(
  Element.prototype,
  "scrollIntoView",
);

const skills: SkillInfo[] = [
  {
    id: "local:skill:alpha",
    name: "alpha-skill",
    command: "alpha-skill",
    description: "Premier skill",
    path: "/skills/alpha/SKILL.md",
    source: "local",
    source_name: "CL-GO-DASH",
  },
  {
    id: "local:skill:beta",
    name: "beta-skill",
    command: "beta-skill",
    description: "Second skill",
    path: "/skills/beta/SKILL.md",
    source: "local",
    source_name: "CL-GO-DASH",
  },
];

const baseProps = {
  draftKey: "skill-keyboard",
  modelName: "test-model",
  providerName: "test-provider",
  isStreaming: false,
  fastModeEnabled: false,
  fastModePending: false,
  contextUsed: 0,
  contextMax: 8000,
  permissionMode: "chat" as PermissionMode,
  onPermissionModeChange: vi.fn(),
  onFileImport: vi.fn(),
  onModelChange: vi.fn(),
  onReasoningModeChange: vi.fn(),
  onFastModeChange: vi.fn(),
  onSend: vi.fn(),
  onStop: vi.fn(),
};

function editorView(container: HTMLElement): EditorView {
  const content = container.querySelector(".cm-content");
  if (!(content instanceof HTMLElement)) throw new Error("Éditeur absent");
  const view = EditorView.findFromDOM(content);
  if (!view) throw new Error("Vue CodeMirror absente");
  return view;
}

function typeSlash(view: EditorView) {
  act(() => {
    view.dispatch({
      changes: { from: 0, insert: "/" },
      selection: EditorSelection.cursor(1),
      userEvent: "input.type",
    });
  });
}

beforeAll(() => {
  globalThis.ResizeObserver = TestResizeObserver;
  Object.defineProperty(Element.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
});

afterAll(() => {
  globalThis.ResizeObserver = originalResizeObserver;
  if (originalScrollIntoView) {
    Object.defineProperty(
      Element.prototype,
      "scrollIntoView",
      originalScrollIntoView,
    );
  } else {
    Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  }
});

beforeEach(() => {
  vi.clearAllMocks();
  clearComposerDraft(baseProps.draftKey);
  vi.mocked(invoke).mockImplementation((command) => {
    if (command === "list_skills") return Promise.resolve(skills);
    if (command === "load_skill") return Promise.resolve("# Skill");
    return Promise.resolve(undefined);
  });
});

describe("navigation clavier des skills", () => {
  it("navigue, boucle puis valide le choix actif avec Entrée", async () => {
    const { container } = render(<ChatInput {...baseProps} />);
    const view = editorView(container);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("list_skills"));
    typeSlash(view);
    await screen.findByText("beta-skill");

    const content = view.contentDOM;
    expect(screen.getByText("compress").closest(".slash-item")).toHaveClass("active");

    fireEvent.keyDown(content, { key: "ArrowDown" });
    expect(screen.getByText("alpha-skill").closest(".slash-item")).toHaveClass("active");

    fireEvent.keyDown(content, { key: "ArrowDown" });
    expect(screen.getByText("beta-skill").closest(".slash-item")).toHaveClass("active");

    fireEvent.keyDown(content, { key: "ArrowDown" });
    expect(screen.getByText("compress").closest(".slash-item")).toHaveClass("active");

    fireEvent.keyDown(content, { key: "ArrowUp" });
    expect(screen.getByText("beta-skill").closest(".slash-item")).toHaveClass("active");

    fireEvent.keyDown(content, { key: "Enter" });

    await waitFor(() => expect(view.state.doc.toString()).toBe("/beta-skill"));
    expect(container.querySelector(".slash-dropdown")).toBeNull();
    expect(view.state.doc.toString()).toBe("/beta-skill");
    expect(container.querySelector(".skill-chip-name")).toHaveTextContent("beta-skill");
  });

  it("laisse Maj + Entrée créer une nouvelle ligne", async () => {
    const { container } = render(<ChatInput {...baseProps} />);
    const view = editorView(container);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("list_skills"));
    typeSlash(view);
    await screen.findByText("alpha-skill");

    fireEvent.keyDown(view.contentDOM, { key: "Enter", shiftKey: true });

    expect(view.state.doc.toString()).toBe("/\n");
    expect(invoke).not.toHaveBeenCalledWith(
      "load_skill",
      expect.anything(),
    );
  });

  it("ferme la liste avec Échap sans modifier le texte", async () => {
    const { container } = render(<ChatInput {...baseProps} />);
    const view = editorView(container);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("list_skills"));
    typeSlash(view);
    await screen.findByText("alpha-skill");

    fireEvent.keyDown(view.contentDOM, { key: "Escape" });

    expect(container.querySelector(".slash-dropdown")).toBeNull();
    expect(view.state.doc.toString()).toBe("/");
  });

  it("ne bloque aucune touche de saisie ordinaire", () => {
    const { container } = render(<ChatInput {...baseProps} />);
    const view = editorView(container);
    const letterEvent = new KeyboardEvent("keydown", {
      key: "a",
      bubbles: true,
      cancelable: true,
    });

    view.contentDOM.dispatchEvent(letterEvent);

    expect(letterEvent.defaultPrevented).toBe(false);
  });

  it("restaure un skill sélectionné avec son contenu après remontage", async () => {
    const onSend = vi.fn();
    const first = render(<ChatInput {...baseProps} onSend={onSend} />);
    const firstView = editorView(first.container);

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("list_skills"));
    typeSlash(firstView);
    await screen.findByText("beta-skill");
    fireEvent.keyDown(firstView.contentDOM, { key: "ArrowUp" });
    fireEvent.keyDown(firstView.contentDOM, { key: "Enter" });
    await waitFor(() => expect(firstView.state.doc.toString()).toBe("/beta-skill"));
    first.unmount();

    const second = render(<ChatInput {...baseProps} onSend={onSend} />);
    expect(second.container.querySelector(".skill-chip-name")).toHaveTextContent("beta-skill");
    fireEvent.keyDown(editorView(second.container).contentDOM, { key: "Enter" });

    expect(onSend).toHaveBeenCalledWith("/beta-skill", undefined, [
      { name: "beta-skill", content: "# Skill" },
    ]);
  });
});
