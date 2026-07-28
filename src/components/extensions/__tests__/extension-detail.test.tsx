import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionDetail } from "../extension-detail";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
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
        onReload={vi.fn()}
        onRemove={vi.fn()}
      />,
    );

    expect(screen.getByText("Search")).toBeInTheDocument();
    expect(screen.queryByText("extensions.detail.contributions"))
      .not.toBeInTheDocument();
  });
});
