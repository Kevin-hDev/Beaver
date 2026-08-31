import { act, fireEvent, render, renderHook, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ChatInput } from "../chat-input";
import type { PermissionMode } from "@/hooks/use-permission-mode";
import { clearComposerDraft, useComposerDraft } from "@/hooks/use-composer-draft";

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

  it("préserve le texte saisi pendant l'envoi précédent", async () => {
    const pending = deferred<boolean>();
    render(
      <ChatInput
        {...baseProps} draftKey="session:one"
        onSend={vi.fn().mockReturnValue(pending.promise)}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Premier" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Suivant" },
    });
    await act(async () => { pending.resolve(true); await pending.promise; });

    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue("Suivant");
  });

  it("retire immédiatement le brouillon du nouveau stream pendant son admission", async () => {
    const pending = deferred<boolean>();
    render(
      <ChatInput
        {...baseProps} draftKey="session:one"
        onSend={vi.fn().mockReturnValue(pending.promise)}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Message envoyé" },
    });

    fireEvent.click(screen.getByRole("button", { name: "send" }));

    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue("");
    await act(async () => { pending.resolve(true); await pending.promise; });
  });

  it("ne remplace pas un nouveau brouillon si l'admission initiale est refusée", async () => {
    const pending = deferred<boolean>();
    render(
      <ChatInput
        {...baseProps} draftKey="session:one"
        onSend={vi.fn().mockReturnValue(pending.promise)}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Premier" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Nouveau brouillon" },
    });

    await act(async () => { pending.resolve(false); await pending.promise; });

    expect(screen.getByRole("textbox", { name: "composer" }))
      .toHaveValue("Nouveau brouillon");
  });

  it("conserve le brouillon visible pendant un stream si l'envoi est refusé", async () => {
    const pending = deferred<boolean>();
    render(
      <ChatInput
        {...baseProps} draftKey="session:one" isStreaming
        onSend={vi.fn().mockReturnValue(pending.promise)}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Précision pendant le stream" },
    });

    fireEvent.click(screen.getByRole("button", { name: "send" }));

    expect(screen.getByRole("textbox", { name: "composer" }))
      .toHaveValue("Précision pendant le stream");
    await act(async () => { pending.resolve(false); await pending.promise; });
    expect(screen.getByRole("textbox", { name: "composer" }))
      .toHaveValue("Précision pendant le stream");
  });

  it("préserve le nouveau fichier et le nouveau skill pendant l'envoi", async () => {
    const pending = deferred<boolean>();
    const onClearFiles = vi.fn();
    const firstFile = { name: "a.txt", path: "/tmp/a.txt", size: 1, type: "text/plain" };
    const secondFile = { name: "b.txt", path: "/tmp/b.txt", size: 1, type: "text/plain" };
    const draft = renderHook(() => useComposerDraft("session:one"));
    const view = render(
      <ChatInput
        {...baseProps} draftKey="session:one" files={[firstFile]}
        onSend={vi.fn().mockReturnValue(pending.promise)} onClearFiles={onClearFiles}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Premier" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));
    view.rerender(
      <ChatInput
        {...baseProps} draftKey="session:one" files={[firstFile, secondFile]}
        onSend={vi.fn().mockReturnValue(pending.promise)} onClearFiles={onClearFiles}
      />,
    );
    act(() => draft.result.current.rememberSkill({
      id: "local:new", name: "new", command: "new", description: "new",
      source: "local", source_name: "Beaver", path: "/skills/new.md",
    }, "manifest"));
    await act(async () => { pending.resolve(true); await pending.promise; });

    expect(onClearFiles).not.toHaveBeenCalled();
    expect(draft.result.current.skills.map((entry) => entry.info.id)).toEqual(["local:new"]);
  });

  it("ignore un double envoi du même instantané", async () => {
    const pending = deferred<boolean>();
    const onSend = vi.fn().mockReturnValue(pending.promise);
    render(<ChatInput {...baseProps} draftKey="session:one" onSend={onSend} />);
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Unique" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));
    fireEvent.click(screen.getByRole("button", { name: "send" }));

    expect(onSend).toHaveBeenCalledOnce();
    await act(async () => { pending.resolve(true); await pending.promise; });
  });

  it("nettoie l'instantané nominal accepté", async () => {
    const onClearFiles = vi.fn();
    render(
      <ChatInput
        {...baseProps} draftKey="session:one"
        files={[{ name: "a.txt", path: "/tmp/a.txt", size: 1, type: "text/plain" }]}
        onSend={vi.fn().mockResolvedValue(true)} onClearFiles={onClearFiles}
      />,
    );
    fireEvent.change(screen.getByRole("textbox", { name: "composer" }), {
      target: { value: "Envoyé" },
    });
    fireEvent.click(screen.getByRole("button", { name: "send" }));

    await waitFor(() => expect(onClearFiles).toHaveBeenCalledOnce());
    expect(screen.getByRole("textbox", { name: "composer" })).toHaveValue("");
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((yes) => { resolve = yes; });
  return { promise, resolve };
}
