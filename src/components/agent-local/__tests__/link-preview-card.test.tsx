import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ChatMarkdown } from "../chat-markdown";
import { LinkPreviewCard } from "../link-preview-card";

const mocks = vi.hoisted(() => ({ open: vi.fn(() => Promise.resolve()) }));

vi.mock("@tauri-apps/plugin-shell", () => ({ open: mocks.open }));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: (key: string) => key }) }));

function apercu(url: string, extra: Record<string, unknown> = {}) {
  return {
    url,
    domain: "github.com",
    site_name: "GitHub",
    title: "Beaver",
    description: "Une description fournie par le site",
    image: "https://github.com/og.png",
    favicon: "https://github.com/favicon.ico",
    ...extra,
  };
}

beforeEach(() => {
  vi.mocked(invoke).mockReset();
  mocks.open.mockClear();
  localStorage.setItem("clgo-link-preview", "true");
});

describe("LinkPreviewCard", () => {
  it("réserve la place de la vignette pendant le chargement", () => {
    vi.mocked(invoke).mockReturnValue(new Promise(() => {}));
    const { container } = render(<LinkPreviewCard url="https://github.com/attente" />);
    const carte = container.querySelector(".lpc-card");

    expect(carte).toHaveAttribute("aria-busy", "true");
    expect(carte?.querySelector(".lpc-media")).toBeTruthy();
    expect(carte?.querySelectorAll(".lpc-bar")).toHaveLength(3);
  });

  it("montre le titre et le domaine seul, sans nom de site, description ni adresse", async () => {
    const url = "https://github.com/contenu";
    vi.mocked(invoke).mockResolvedValue(apercu(url));
    const { container } = render(<LinkPreviewCard url={url} />);

    expect(await screen.findByText("Beaver")).toBeTruthy();
    expect(screen.getByText("github.com")).toBeTruthy();
    expect(screen.queryByText("GitHub")).toBeNull();
    expect(screen.queryByText("Une description fournie par le site")).toBeNull();
    expect(screen.queryByText(url)).toBeNull();
    expect(container.querySelector(".lpc-card")).toHaveClass("relief");
    expect(container.querySelector(".lpc-chip img")).toHaveAttribute("src", "https://github.com/favicon.ico");
  });

  it("garde le cadre de l'image, icône au centre, quand le site n'a pas d'image", async () => {
    const url = "https://github.com/sans-image";
    vi.mocked(invoke).mockResolvedValue(apercu(url, { image: null }));
    const { container } = render(<LinkPreviewCard url={url} />);

    await screen.findByText("Beaver");
    expect(container.querySelector(".lpc-media.lpc-no-image .lpc-chip-large img")).toBeTruthy();
  });

  it("passe au cadre sans image quand l'image ne charge pas", async () => {
    const url = "https://github.com/image-cassee";
    vi.mocked(invoke).mockResolvedValue(apercu(url));
    const { container } = render(<LinkPreviewCard url={url} />);

    await screen.findByText("Beaver");
    fireEvent.error(container.querySelector(".lpc-media img")!);
    expect(container.querySelector(".lpc-media.lpc-no-image")).toBeTruthy();
  });

  it("retire la pastille quand l'icône du site ne charge pas", async () => {
    const url = "https://github.com/icone-cassee";
    vi.mocked(invoke).mockResolvedValue(apercu(url));
    const { container } = render(<LinkPreviewCard url={url} />);

    await screen.findByText("Beaver");
    fireEvent.error(container.querySelector(".lpc-chip img")!);
    expect(container.querySelector(".lpc-chip")).toBeNull();
  });

  it("donne l'adresse complète dans l'infobulle au survol", async () => {
    const url = "https://github.com/infobulle";
    vi.mocked(invoke).mockResolvedValue(apercu(url));
    const { container } = render(<LinkPreviewCard url={url} />);

    await screen.findByText("Beaver");
    vi.useFakeTimers();
    try {
      fireEvent.mouseEnter(container.querySelector(".tooltip-wrapper")!);
      await act(() => vi.advanceTimersByTime(300));
      expect(screen.getByText(url)).toBeTruthy();
    } finally {
      vi.useRealTimers();
    }
  });

  it("ouvre le lien au clic", async () => {
    const url = "https://github.com/clic";
    vi.mocked(invoke).mockResolvedValue(apercu(url));
    render(<LinkPreviewCard url={url} />);

    fireEvent.click(await screen.findByRole("button"));
    expect(mocks.open).toHaveBeenCalledWith(url);
  });

  it.each([
    ["lien Markdown", "[Documentation](https://example.com/echec-markdown)", "https://example.com/echec-markdown"],
    ["adresse directe", "https://example.com/echec-direct", "https://example.com/echec-direct"],
  ])("conserve le %s après l'échec de son aperçu", async (_nom, contenu, url) => {
    vi.mocked(invoke).mockRejectedValue(new Error("indisponible"));
    const { container } = render(<ChatMarkdown content={contenu} linkPreviews />);

    await waitFor(() => expect(container.querySelector(".lpc-card")).toBeNull());
    const lien = screen.getByRole("link");
    expect(lien).toHaveAttribute("href", url);
    fireEvent.click(lien);
    expect(mocks.open).toHaveBeenCalledWith(url);
  });
});
