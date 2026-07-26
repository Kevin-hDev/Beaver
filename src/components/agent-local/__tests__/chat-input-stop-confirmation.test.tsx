import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatInput } from "../chat-input";
import type { PermissionMode } from "@/hooks/use-permission-mode";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../chat-input-editor", () => ({
  ChatInputEditor: ({ onTextChange }: { onTextChange: (value: string, cursor: number) => void }) => (
    <button type="button" onClick={() => onTextChange("Correction", 10)}>
      type message
    </button>
  ),
}));

vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: ({
    buttonState,
    onSend,
    onStop,
  }: {
    buttonState: string;
    onSend: () => void;
    onStop: () => void;
  }) => (
    <button
      type="button"
      data-state={buttonState}
      onClick={buttonState === "send" ? onSend : onStop}
    >
      stop action
    </button>
  ),
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

const baseProps = {
  modelName: "llama3",
  providerName: "ollama",
  isStreaming: true,
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

afterEach(() => {
  vi.clearAllMocks();
});

describe("ChatInput stop confirmation", () => {
  it("affiche la confirmation au premier clic et arrête au deuxième", () => {
    const onStop = vi.fn();

    render(<ChatInput {...baseProps} onStop={onStop} />);

    const stopAction = screen.getByText("stop action");

    expect(stopAction).toHaveAttribute("data-state", "stop");

    fireEvent.click(stopAction);

    expect(stopAction).toHaveAttribute("data-state", "confirmStop");
    expect(onStop).not.toHaveBeenCalled();

    fireEvent.click(stopAction);

    expect(onStop).toHaveBeenCalledOnce();
  });

  it("permet d'envoyer un nouveau message pendant le stream", () => {
    const onSend = vi.fn();

    render(<ChatInput {...baseProps} onSend={onSend} />);

    fireEvent.click(screen.getByText("type message"));
    const action = screen.getByText("stop action");

    expect(action).toHaveAttribute("data-state", "send");
    fireEvent.click(action);
    expect(onSend).toHaveBeenCalledWith("Correction", undefined, []);
  });
});
