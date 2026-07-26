import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { ChatInput } from "../chat-input";
import type { PermissionMode } from "@/hooks/use-permission-mode";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text = (value: unknown) => (
        typeof value === "string" || typeof value === "number" ? String(value) : ""
      );
      if (key === "interactiveChoice.step") return `${text(opts?.current)}/${text(opts?.total)}`;
      if (key === "interactiveChoice.otherLabel") return "Other";
      if (key === "interactiveChoice.other") return "Other answer";
      if (key === "interactiveChoice.otherPlaceholder") return "Write your answer";
      if (key === "interactiveChoice.recommended") return "Recommended";
      return key;
    },
  }),
}));

vi.mock("../chat-input-editor", () => ({
  ChatInputEditor: () => <div data-testid="chat-input-editor" />,
}));
vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: () => <div data-testid="chat-input-actions-row" />,
}));
vi.mock("../slash-autocomplete", () => ({
  SlashAutocomplete: () => <div data-testid="slash-autocomplete" />,
}));
vi.mock("../file-thumbnail", () => ({
  FileThumbnail: () => <div data-testid="file-thumbnail" />,
}));
vi.mock("@/hooks/use-slash-commands", () => ({
  useSlashCommands: () => ({
    showDropdown: false,
    skills: [],
    activeIndex: 0,
    handleInput: vi.fn(),
    moveUp: vi.fn(),
    moveDown: vi.fn(),
    close: vi.fn(),
  }),
}));
vi.mock("@/hooks/use-active-skills", () => ({
  useActiveSkills: () => ({
    activeSkills: [],
    getSkillsPayload: () => [],
    clearSkills: vi.fn(),
    handleSelectSkill: vi.fn(),
  }),
}));
vi.mock("../chat.css", () => ({}));
vi.mock("../chat-input-textarea.css", () => ({}));
vi.mock("../chat-input-responsive.css", () => ({}));
vi.mock("../interactive-choice-panel.css", () => ({}));

const baseProps = {
  modelName: "llama3",
  providerName: "ollama",
  isStreaming: false,
  contextUsed: 0,
  contextMax: 8000,
  permissionMode: "chat" as PermissionMode,
  onPermissionModeChange: vi.fn(),
  onFileImport: vi.fn(),
  onModelChange: vi.fn(),
  onReasoningModeChange: vi.fn(),
  onSend: vi.fn(),
  onStop: vi.fn(),
};

const interactiveRequest = {
  sessionId: "session-1",
  id: "choice-1",
  currentIndex: 0,
  total: 1,
  questions: [{
    header: "Plan",
    question: "What next?",
    options: [
      { label: "Fast", description: "Do the minimum", recommended: true },
      { id: "complete", label: "Complete", description: "Do the full pass" },
    ],
  }],
};

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

beforeEach(() => {
  vi.mocked(invoke).mockResolvedValue(undefined);
});

describe("ChatInput interactive mode", () => {
  it("remplace l'éditeur par le panneau de choix quand interactiveRequest est fourni", () => {
    render(
      <ChatInput
        {...baseProps}
        interactiveRequest={interactiveRequest}
        onInteractiveResolved={vi.fn()}
      />,
    );

    expect(screen.getByText("What next?")).toBeTruthy();
    expect(screen.getByText("Fast")).toBeTruthy();
    expect(screen.queryByTestId("chat-input-editor")).toBeNull();
    expect(screen.queryByTestId("chat-input-actions-row")).toBeNull();
  });

  it("réaffiche l'éditeur et la barre d'actions quand interactiveRequest est retiré", () => {
    const { rerender } = render(
      <ChatInput
        {...baseProps}
        interactiveRequest={interactiveRequest}
        onInteractiveResolved={vi.fn()}
      />,
    );

    expect(screen.queryByTestId("chat-input-editor")).toBeNull();

    rerender(<ChatInput {...baseProps} interactiveRequest={null} />);

    expect(screen.getByTestId("chat-input-editor")).toBeTruthy();
    expect(screen.getByTestId("chat-input-actions-row")).toBeTruthy();
    expect(screen.queryByText("What next?")).toBeNull();
  });

  it("appelle onInteractiveResolved après un clic sur un choix", async () => {
    const onResolved = vi.fn();
    render(
      <ChatInput
        {...baseProps}
        interactiveRequest={interactiveRequest}
        onInteractiveResolved={onResolved}
      />,
    );

    fireEvent.click(screen.getByText("Complete"));

    await waitFor(() => expect(onResolved).toHaveBeenCalledOnce());
  });
});
