import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewSummary } from "../file-preview-summary";
import type { FileOperation } from "@/types/file-preview";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("FilePreviewSummary", () => {
  it("affiche une liste compacte sans titre de section", () => {
    render(
      <FilePreviewSummary
        operations={[
          operation({
            path: "/repo/src/components/agent-local/message-tool-blocks.ts",
            name: "message-tool-blocks.ts",
            additions: 11,
            deletions: 5,
          }),
        ]}
        baseDir="/repo"
        onOpen={vi.fn()}
        onOpenFile={vi.fn()}
      />,
    );

    expect(screen.queryByText("filePreview.filesModified")).toBeNull();
    expect(screen.getByText("src/components/agent-local/", { exact: false })).toBeTruthy();
    expect(screen.getByText("message-tool-blocks.ts")).toBeTruthy();
    expect(screen.getByText("+11")).toBeTruthy();
    expect(screen.getByText("-5")).toBeTruthy();
  });

  it("affiche les suppressions à zéro et ouvre le fichier au clic", () => {
    const onOpen = vi.fn();
    const file = operation({ additions: 24, deletions: 0 });

    render(
      <FilePreviewSummary
        operations={[file]}
        baseDir="/repo"
        onOpen={onOpen}
        onOpenFile={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: /agent-chat-utils/ }));

    expect(screen.getByText("+24")).toBeTruthy();
    expect(screen.getByText("-0")).toBeTruthy();
    expect(onOpen).toHaveBeenCalledWith(file);
  });

  it("ouvre le fichier complet avec le bouton de droite sans ouvrir la diff", () => {
    const onOpen = vi.fn();
    const onOpenFile = vi.fn();
    const file = operation({ additions: 6, deletions: 0 });

    render(
      <FilePreviewSummary
        operations={[file]}
        baseDir="/repo"
        onOpen={onOpen}
        onOpenFile={onOpenFile}
      />,
    );

    fireEvent.click(screen.getByRole("button", { name: "filePreview.open" }));

    expect(onOpenFile).toHaveBeenCalledWith(file);
    expect(onOpen).not.toHaveBeenCalled();
  });
});

function operation(overrides: Partial<FileOperation>): FileOperation {
  return {
    id: "op-1",
    path: "/repo/src/hooks/agent-chat-utils.test.ts",
    name: "agent-chat-utils.test.ts",
    type: "edit",
    timestamp: "2026-07-02T00:00:00Z",
    additions: 1,
    deletions: 0,
    ...overrides,
  };
}
