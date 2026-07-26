import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { FilePreviewDiff } from "../file-preview-diff";

describe("FilePreviewDiff", () => {
  it("n'affiche pas de ligne fictive après le retour final", () => {
    const oldText = "# Test\n\nAncien texte\n";
    const newText = [
      "# Test",
      "",
      "Ligne 1",
      "Ligne 2",
      "Ligne 3",
      "Ligne 4",
      "Ligne 5",
      "Ligne 6",
      "",
    ].join("\n");

    const { container } = render(
      <FilePreviewDiff
        currentContent={newText}
        operation={{
          id: "edit:test",
          path: "TEST-3.md",
          name: "TEST-3.md",
          type: "edit",
          timestamp: "",
          oldText,
          newText,
          startLine: 1,
          additions: 8,
          deletions: 3,
        }}
      />,
    );

    const addedNumbers = [...container.querySelectorAll(".tp-line-ok .tp-num")]
      .map((node) => node.textContent);
    expect(addedNumbers).toHaveLength(8);
    expect(addedNumbers[addedNumbers.length - 1]).toBe("8");
    expect(addedNumbers).not.toContain("9");
  });
});
