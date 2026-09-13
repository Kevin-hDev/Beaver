import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { FilePreviewBreadcrumb } from "../file-preview-breadcrumb";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

let writeText: ReturnType<typeof vi.fn>;

beforeEach(() => {
  writeText = vi.fn(() => Promise.resolve());
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText },
  });
});

describe("FilePreviewBreadcrumb", () => {
  it("copie le chemin complet avec l'icône placée avant le chemin", async () => {
    const { container } = render(
      <FilePreviewBreadcrumb
        operation={{
          id: "file-1",
          path: "src/index.ts",
          name: "index.ts",
          type: "write",
          timestamp: "2026-09-13T10:00:00Z",
          additions: 1,
          deletions: 0,
        }}
        baseDir="/repo"
      />,
    );

    const button = screen.getByRole("button", { name: "agentLocal.copy" });
    expect(container.querySelector(".fp-breadcrumb")?.firstElementChild).toContainElement(button);

    fireEvent.click(button);

    await waitFor(() => expect(writeText).toHaveBeenCalledWith("/repo/src/index.ts"));
  });
});
