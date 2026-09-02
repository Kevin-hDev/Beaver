import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionDetail } from "../extension-detail";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));

const extension: ExtensionRecord = {
  manifest: {
    id: "com.example.search",
    name: "Search",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    access: "full",
    apiLevel: "advanced",
    essential: false,
  },
  kind: "local",
  source: "/extension",
  enabled: true,
  trusted: true,
  showInChat: true,
  status: "active",
  contributions: {
    tools: [{
      name: "web_search",
      description: "Custom search behavior",
      parameters: { type: "object" },
      replacesCore: true,
      effect: "unknown",
    }],
    events: [],
  },
};

describe("ExtensionDetail", () => {
  it("shows replacement status and the prompt-visible tool description", () => {
    render(
      <ExtensionDetail
        extension={extension}
        busy={false}
        onBack={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={vi.fn()}
        onReload={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("extensions.detail.replacesCore")).toBeInTheDocument();
    expect(screen.getByText("Custom search behavior")).toBeInTheDocument();
  });

  it("reste affichable si une ancienne réponse IPC omet les contributions", () => {
    const incomplete = {
      ...extension,
      contributions: undefined,
    } as unknown as ExtensionRecord;

    render(
      <ExtensionDetail
        extension={incomplete}
        busy={false}
        onBack={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={vi.fn()}
        onReload={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("Search")).toBeInTheDocument();
    expect(screen.queryByText("extensions.detail.contributions"))
      .not.toBeInTheDocument();
  });

  it("affiche la provenance et la mise à jour uniquement pour une source gérée", () => {
    const onUpdate = vi.fn();
    const managed: ExtensionRecord = {
      ...extension,
      origin: {
        kind: "git",
        locator: "https://github.com/example/search.git",
        revision: "a".repeat(40),
      },
    };
    render(
      <ExtensionDetail
        extension={managed}
        busy={false}
        onBack={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={onUpdate}
        onReload={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText(managed.origin?.locator ?? "")).toBeInTheDocument();
    expect(screen.getByText("extensions.updateTrustWarning")).toBeInTheDocument();
    fireEvent.click(screen.getByText("extensions.actions.update"));
    fireEvent.click(screen.getByText("extensions.actions.confirmUpdate"));
    expect(onUpdate).toHaveBeenCalledTimes(1);
  });

  it("formate les dates persistées dans la langue de l'interface", () => {
    const dated = {
      ...extension,
      lastActivatedAt: "2026-09-02T15:59:35Z",
      trustedAt: "date-invalide",
    };
    const expected = new Intl.DateTimeFormat("fr", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(dated.lastActivatedAt));

    render(
      <ExtensionDetail
        extension={dated}
        busy={false}
        onBack={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={vi.fn()}
        onReload={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText(expected)).toBeInTheDocument();
    expect(screen.getByText("date-invalide")).toBeInTheDocument();
    expect(screen.queryByText(dated.lastActivatedAt)).not.toBeInTheDocument();
  });
});
