import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatInput } from "../chat-input";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import { clearComposerDraft } from "@/hooks/use-composer-draft";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../chat-input-editor", () => ({
  ChatInputEditor: ({
    value,
    onTextChange,
  }: {
    value: string;
    onTextChange: (value: string, cursor: number) => void;
  }) => (
    <textarea
      aria-label="composer"
      value={value}
      onChange={(event) => onTextChange(event.target.value, event.target.value.length)}
    />
  ),
}));

vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: ({ onSend }: { onSend: () => void }) => (
    <button type="button" onClick={onSend}>send</button>
  ),
}));

vi.mock("../slash-autocomplete", () => ({
  SlashAutocomplete: () => null,
}));

vi.mock("../file-thumbnail", () => ({
  FileThumbnail: () => null,
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

const baseProps = {
  modelName: "test-model",
  providerName: "test-provider",
  isStreaming: false,
  fastModeEnabled: false,
  fastModePending: false,
  contextUsed: 0,
  contextMax: 8_000,
  permissionMode: "chat" as PermissionMode,
  onPermissionModeChange: vi.fn(),
  onFileImport: vi.fn(),
  onModelChange: vi.fn(),
  onReasoningModeChange: vi.fn(),
  onFastModeChange: vi.fn(),
  onSend: vi.fn(),
  onStop: vi.fn(),
};

afterEach(() => {
  clearComposerDraft("session:one");
  clearComposerDraft("session:two");
  vi.clearAllMocks();
});

describe("ChatInput drafts", () => {
  it("restores one session draft after the composer is remounted", () => {
    const first = render(<ChatInput {...baseProps} draftKey="session:one" />);
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Message non envoyé" },
    });
    first.unmount();

    render(<ChatInput {...baseProps} draftKey="session:one" />);

    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue(
      "Message non envoyé",
    );
  });

  it("keeps drafts isolated and clears only the one that is sent", () => {
    const first = render(<ChatInput {...baseProps} draftKey="session:one" />);
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Premier brouillon" },
    });
    first.unmount();

    const second = render(<ChatInput {...baseProps} draftKey="session:two" />);
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Second brouillon" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));
    second.unmount();

    render(<ChatInput {...baseProps} draftKey="session:one" />);
    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue(
      "Premier brouillon",
    );
  });

  it("préserve le texte et les pièces jointes quand l'envoi est refusé", async () => {
    const onClearFiles = vi.fn();
    const onSend = vi.fn().mockResolvedValue(false);
    render(
      <ChatInput
        {...baseProps}
        draftKey="session:one"
        files={[{
          name: "note.txt", path: "/tmp/note.txt", size: 4, type: "text/plain",
        }]}
        onSend={onSend}
        onClearFiles={onClearFiles}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Texte à conserver" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));

    await waitFor(() => expect(onSend).toHaveBeenCalledOnce());
    expect(screen.getByRole("textbox", { name: "composer" }))
      .toHaveValue("Texte à conserver");
    expect(onClearFiles).not.toHaveBeenCalled();
  });

  it("evicts the oldest draft when the bounded cache is full", () => {
    for (let index = 0; index < 65; index += 1) {
      const view = render(
        <ChatInput {...baseProps} draftKey={`capacity:${index}`} />,
      );
      fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
        target: { value: `Draft ${index}` },
      });
      view.unmount();
    }

    const oldest = render(<ChatInput {...baseProps} draftKey="capacity:0" />);
    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue("");
    oldest.unmount();

    for (let index = 0; index < 65; index += 1) {
      clearComposerDraft(`capacity:${index}`);
    }
  });
});
