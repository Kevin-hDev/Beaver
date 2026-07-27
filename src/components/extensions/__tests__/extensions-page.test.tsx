import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";
import { ExtensionsPage } from "../extensions-page";
import { ExtensionsSidebar } from "../extensions-sidebar";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const host: ExtensionHostStatus = {
  state: "running",
  nodeVersion: "v24.18.0",
  jitiVersion: "2.7.0",
  apiVersion: "1",
  activeExtensions: 0,
  diagnostics: [],
};

function record(kind: ExtensionRecord["kind"], id: string, name: string): ExtensionRecord {
  return {
    manifest: {
      id,
      name,
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      access: "full",
      apiLevel: "stable",
    },
    kind,
    source: kind,
    enabled: false,
    trusted: false,
    showInChat: false,
    status: "inactive",
    contributions: { tools: [], events: [] },
  };
}

const records = [
  record("builtin", "beaver.sample.official", "Plugin officiel"),
  record("local", "com.example.custom", "Extension locale"),
  record("external", "com.example.external", "Application externe"),
];

function renderPage(section: "plugins" | "custom" | "external", items = records) {
  return render(
    <ExtensionsPage
      section={section}
      selected={null}
      records={items}
      host={host}
      loading={false}
      loadError={false}
      operationError={false}
      busyIds={new Set()}
      onSelect={vi.fn()}
      onAdd={vi.fn()}
      onEnabled={vi.fn()}
      onShowInChat={vi.fn()}
      onOpenSource={vi.fn()}
      onRemove={vi.fn()}
      onReload={vi.fn()}
      onRecover={vi.fn()}
    />,
  );
}

describe("ExtensionsPage", () => {
  it("sépare strictement les plugins officiels des extensions locales", () => {
    renderPage("custom");

    expect(screen.getByText("Extension locale")).toBeInTheDocument();
    expect(screen.queryByText("Plugin officiel")).not.toBeInTheDocument();
    expect(screen.queryByText("Application externe")).not.toBeInTheDocument();
  });

  it("affiche uniquement les vrais plugins dans le catalogue officiel", () => {
    renderPage("plugins");

    expect(screen.getByText("Plugin officiel")).toBeInTheDocument();
    expect(screen.queryByText("Extension locale")).not.toBeInTheDocument();
  });

  it("utilise le nom localisé et l'icône du plugin Documents", () => {
    const { container } = renderPage(
      "plugins",
      [record("builtin", "beaver.office.documents", "Fallback")],
    );

    expect(
      screen.getByText("extensions.official.documents.name"),
    ).toBeInTheDocument();
    expect(container.querySelector(".exti-artwork")).toBeInTheDocument();
  });

  it("garde le catalogue officiel vide sans transformer les Tools en plugins", () => {
    renderPage("plugins", records.filter((item) => item.kind !== "builtin"));

    expect(screen.getByText("extensions.pages.plugins.empty")).toBeInTheDocument();
  });

  it("ramène la section active dans la zone visible sans attendre un rendu", () => {
    const scrollIntoView = vi.fn();
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    const { rerender } = render(
      <ExtensionsSidebar section="plugins" onSelect={vi.fn()} />,
    );

    rerender(<ExtensionsSidebar section="host" onSelect={vi.fn()} />);

    expect(scrollIntoView).toHaveBeenLastCalledWith({ block: "nearest" });
  });
});
