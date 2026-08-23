import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { WelcomeView } from "../welcome-view";
import {
  clearComposerDraft,
  WELCOME_COMPOSER_DRAFT_KEY,
} from "@/hooks/use-composer-draft";

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
      aria-label="welcome composer"
      value={value}
      onChange={(event) => onTextChange(event.target.value, event.target.value.length)}
    />
  ),
}));

vi.mock("../chat-input-actions-row", () => ({
  ChatInputActionsRow: () => null,
}));
vi.mock("../slash-autocomplete", () => ({ SlashAutocomplete: () => null }));
vi.mock("../file-thumbnail", () => ({ FileThumbnail: () => null }));
vi.mock("../welcome-wordmark", () => ({ WelcomeWordmark: () => null }));
vi.mock("../project-selector", () => ({ ProjectSelector: () => null }));
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
  useDirectoryAccessGuard: () => ({ prompt: null, request: vi.fn() }),
}));
vi.mock("@/hooks/project-directory-selection", () => ({
  addProjectDirectory: vi.fn(),
  selectProjectDirectory: vi.fn(),
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
});
