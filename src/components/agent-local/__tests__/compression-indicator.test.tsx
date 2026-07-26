import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { CompressionIndicator } from "../compression-indicator";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key === "agentLocal.compression"
      ? "Compression du contexte"
      : key,
  }),
}));

describe("CompressionIndicator", () => {
  it("affiche le libellé centré entre deux lignes", () => {
    const view = render(<CompressionIndicator />);

    expect(view.getByRole("status").textContent).toBe("Compression du contexte");
    expect(view.container.querySelectorAll(".compression-line")).toHaveLength(2);
  });
});
