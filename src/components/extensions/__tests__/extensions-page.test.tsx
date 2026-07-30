import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionHostStatus, ExtensionRecord } from "@/types/extensions";
import { ExtensionsPage } from "../extensions-page";

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
      essential: false,
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

function renderPage(
  section: "plugins" | "custom" | "external",
  items = records,
  onSelectSection = vi.fn(),
) {
  return render(
    <ExtensionsPage
      section={section}
      selected={null}
      onSelectSection={onSelectSection}
      records={items}
      host={host}
      loading={false}
      loadError={null}
      operationError={null}
      busyIds={new Set()}
      protectedPluginIds={[]}
      priorityBusy={false}
      onSelect={vi.fn()}
      onAdd={vi.fn()}
      onEnabled={vi.fn()}
      onShowInChat={vi.fn()}
      onOpenSource={vi.fn()}
      onUpdate={vi.fn()}
      onRemove={vi.fn()}
      onReload={vi.fn()}
      onRecover={vi.fn()}
      onPrioritySave={vi.fn(() => Promise.resolve(true))}
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

  it("affiche la clé d’erreur précise fournie par le registre", () => {
    render(
      <ExtensionsPage
        section="plugins"
        selected={null}
        onSelectSection={vi.fn()}
        records={records}
        host={host}
        loading={false}
        loadError={null}
        operationError="extensions.errors.codes.extensions_host_unavailable"
        busyIds={new Set()}
        protectedPluginIds={[]}
        priorityBusy={false}
        onSelect={vi.fn()}
        onAdd={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={vi.fn()}
        onRemove={vi.fn()}
        onReload={vi.fn()}
        onRecover={vi.fn()}
        onPrioritySave={vi.fn(() => Promise.resolve(true))}
      />,
    );

    expect(screen.getByText(
      "extensions.errors.codes.extensions_host_unavailable",
    )).toBeInTheDocument();
  });

  it("expose les quatre sections comme onglets en haut de page", () => {
    renderPage("plugins");

    const tabs = screen.getAllByRole("tab");

    expect(tabs.map((tab) => tab.textContent)).toEqual([
      "extensions.sections.plugins",
      "extensions.sections.custom",
      "extensions.sections.external",
      "extensions.sections.host",
    ]);
    expect(tabs[0]).toHaveAttribute("aria-selected", "true");
  });

  it("change de section au clic sur un onglet", () => {
    const onSelectSection = vi.fn();
    renderPage("plugins", records, onSelectSection);

    fireEvent.click(screen.getByText("extensions.sections.host"));

    expect(onSelectSection).toHaveBeenCalledWith("host");
  });
});
