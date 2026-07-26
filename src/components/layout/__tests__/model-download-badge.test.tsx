/* @vitest-environment jsdom */
import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { ModelDownloadState } from "@/hooks/use-model-downloads";
import { ModelDownloadBadge } from "../model-download-badge";

type BadgeHookValue = {
  activeDownload: Pick<ModelDownloadState, "kind" | "phase" | "percent"> | null;
};

const mockedUseModelDownloads = vi.fn<() => BadgeHookValue>();

vi.mock("@/hooks/use-model-downloads", () => ({
  useModelDownloads: () => mockedUseModelDownloads(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ModelDownloadBadge", () => {
  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("reste cache sans telechargement actif", () => {
    mockedUseModelDownloads.mockReturnValue({ activeDownload: null });

    const { container } = render(<ModelDownloadBadge />);

    expect(container.firstChild).toBeNull();
  });

  it("ne duplique pas la progression locale Forecast", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: {
        kind: "forecast",
        phase: "downloading",
        percent: 64,
      },
    });

    const { container } = render(<ModelDownloadBadge />);

    expect(container.firstChild).toBeNull();
  });

  it("affiche toujours la progression globale Ollama", () => {
    mockedUseModelDownloads.mockReturnValue({
      activeDownload: {
        kind: "ollama",
        phase: "downloading",
        percent: 64,
      },
    });

    render(<ModelDownloadBadge />);

    expect(screen.getByText("modelDownloads.kinds.ollama")).toBeTruthy();
    expect(screen.getByText("modelDownloads.phases.downloading")).toBeTruthy();
    expect(screen.getByText("64%")).toBeTruthy();
  });
});
