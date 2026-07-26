import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { FilePreviewTab } from "../file-preview-tab";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

describe("FilePreviewTab", () => {
  it("superpose la fermeture et l'icone dans le meme emplacement", () => {
    const { container } = render(
      <FilePreviewTab
        active
        label="TEST-2.md"
        onSelect={vi.fn()}
        onClose={vi.fn()}
      />,
    );

    const tooltip = container.querySelector<HTMLElement>(".tooltip-wrapper");
    const iconSlot = container.querySelector<HTMLElement>(".fp-tab-icon");

    expect(tooltip).toContainElement(iconSlot);
  });
});
