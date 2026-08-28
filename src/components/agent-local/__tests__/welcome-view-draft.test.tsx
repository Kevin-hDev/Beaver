import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WelcomeView } from "../welcome-view";
import {
  clearComposerDraft,
  WELCOME_COMPOSER_DRAFT_KEY,
} from "@/hooks/use-composer-draft";

const directoryAccess = vi.hoisted(() => ({ request: vi.fn() }));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("../chat-input-editor", () => ({
  ChatInputEditor: ({
    value,
    onTextChange,
    onKeyEvent,
  }: {
    value: string;
    onTextChange: (value: string, cursor: number) => void;
    onKeyEvent: (event: KeyboardEvent) => boolean | void;
  }) => (
    <textarea
      aria-label="welcome composer"
      value={value}
      onChange={(event) => onTextChange(event.target.value, event.target.value.length)}
      onKeyDown={(event) => onKeyEvent(event.nativeEvent)}
    />
  ),
}));

vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: () => null,
}));
vi.mock("../slash-autocomplete", () => ({ SlashAutocomplete: () => null }));
vi.mock("../file-thumbnail", () => ({ FileThumbnail: () => null }));
vi.mock("../welcome-wordmark", () => ({ WelcomeWordmark: () => null }));
vi.mock("../project-selector", () => ({
  ProjectSelector: ({ onSelect }: { onSelect: (id: string) => void }) => (
    <button onClick={() => onSelect("project-1")}>select project</button>
  ),
}));
vi.mock("../file-drop-zone", () => ({
  FileDropZone: ({ children }: { children: React.ReactNode }) => children,
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

vi.mock("@/hooks/use-permission-mode", () => ({
  usePermissionMode: () => ({ mode: "chat", change: vi.fn() }),
}));
vi.mock("@/hooks/use-directory-access-guard", () => ({
  useDirectoryAccessGuard: () => ({ prompt: null, request: directoryAccess.request }),
}));
vi.mock("@/hooks/project-directory-selection", () => ({
  addProjectDirectory: vi.fn(),
  selectProjectDirectory: (
    id: string | null,
    _projects: unknown,
    _request: unknown,
    select: (selected: string | null) => void,
  ) => select(id),
}));
vi.mock("@/hooks/use-file-drop", () => ({
  useFileDrop: () => ({
    files: [],
    dragging: false,
    setDragging: vi.fn(),
    addByPaths: vi.fn(),
    removeFile: vi.fn(),
    clearFiles: vi.fn(),
  }),
}));

const props = {
  model: "test-model",
  provider: "test-provider",
  projects: [],
  onAddProject: vi.fn(),
  onSend: vi.fn(),
  onModelChange: vi.fn(),
  onReasoningModeChange: vi.fn(),
  fastModeEnabled: false,
  onFastModeChange: vi.fn(),
};

afterEach(() => {
  clearComposerDraft(WELCOME_COMPOSER_DRAFT_KEY);
  vi.clearAllMocks();
});

describe("WelcomeView draft", () => {
  it("restores its unsent text after leaving and reopening the welcome view", () => {
    const first = render(<WelcomeView {...props} />);
    fireEvent.change(screen.getByRole("textbox", { name: "welcome composer" }), {
      target: { value: "Nouvelle conversation à préparer" },
    });
    first.unmount();

    render(<WelcomeView {...props} />);

    expect(screen.getByRole("textbox", { name: "welcome composer" })).toHaveValue(
      "Nouvelle conversation à préparer",
    );
  });

  it("keeps the draft when directory access refuses the send", async () => {
    directoryAccess.request.mockResolvedValueOnce(false);
    const onSend = vi.fn();
    render(
      <WelcomeView
        {...props}
        onSend={onSend}
        projects={[{
          id: "project-1",
          name: "Project",
          path: "/private/project",
          order: 0,
          created_at: "2026-08-27T00:00:00Z",
        }]}
      />,
    );
    fireEvent.click(screen.getByRole("button", { name: "select project" }));
    const editor = screen.getByRole("textbox", { name: "welcome composer" });
    fireEvent.change(editor, { target: { value: "Texte à conserver" } });
    fireEvent.keyDown(editor, { key: "Enter" });

    await waitFor(() => expect(directoryAccess.request).toHaveBeenCalledOnce());
    expect(onSend).not.toHaveBeenCalled();
    expect(editor).toHaveValue("Texte à conserver");
  });
});
