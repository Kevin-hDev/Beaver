import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewContent } from "../file-preview-content";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("FilePreviewContent", () => {
  it("continue les numéros sous un fichier court", () => {
    const { container } = render(
      <FilePreviewContent
        operation={{
          id: "file-1",
          path: "/repo/index.js",
          name: "index.js",
          type: "write",
          timestamp: "2026-09-13T10:00:00Z",
          content: "export default function activate() {}",
          additions: 1,
          deletions: 0,
          source: {
            kind: "git",
            commitId: "abc123",
            filePath: "index.js",
            expectedBranch: "main",
            useParent: false,
          },
        }}
      />,
    );

    const filler = container.querySelector(".fp-code-filler");
    expect(filler).toHaveAttribute("aria-hidden", "true");
    expect(filler!.textContent?.split("\n").slice(0, 3)).toEqual(["2", "3", "4"]);
  });
});
