import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewSummarySelector } from "../file-preview-summary-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("FilePreviewSummarySelector", () => {
  it("élargit le menu à son libellé le plus long", async () => {
    const scrollWidth = vi.spyOn(HTMLElement.prototype, "scrollWidth", "get")
      .mockReturnValue(260);
    try {
      render(
        <FilePreviewSummarySelector
          active
          mode="latest"
          onSelect={vi.fn()}
          onModeChange={vi.fn()}
        />,
      );

      fireEvent.click(screen.getByRole("button", { name: "filePreview.listModes.latest" }));

      await waitFor(() => {
        expect(document.querySelector<HTMLElement>(".fps-menu")?.style.width).toBe("260px");
      });
    } finally {
      scrollWidth.mockRestore();
    }
  });
});
