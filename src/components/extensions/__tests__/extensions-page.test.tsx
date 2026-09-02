import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionHostStatus, ExtensionRecord, ExtensionRecoveryState } from "@/types/extensions";
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
const recovery: ExtensionRecoveryState = {
  extensionId: null,
  stage: null,
  attempts: null,
  canRetry: false,
  markerInvalid: false,
  recoverySnapshotAvailable: false,
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
];

function renderPage(
  section: "plugins" | "custom",
  items = records,
  onSelectSection = vi.fn(),
  state: {
    loading?: boolean;
    loadError?: string | null;
    recovery?: ExtensionRecoveryState;
    hostBusy?: boolean;
  } = {},
) {
  return render(
    <ExtensionsPage
      section={section}
      selected={null}
      onSelectSection={onSelectSection}
      records={items}
      host={host}
      loading={state.loading ?? false}
      loadError={state.loadError ?? null}
      operationError={null}
      recovery={state.recovery ?? recovery}
      hostBusy={state.hostBusy ?? false}
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
      onKeepDisabled={vi.fn()}
      onRetryLoad={vi.fn()}
      onDiscardMarker={vi.fn()}
      onRestoreSnapshot={vi.fn()}
      onPrioritySave={vi.fn(() => Promise.resolve(true))}
    />,
  );
}

describe("ExtensionsPage", () => {
  it("sépare strictement les plugins officiels des extensions locales", () => {
    renderPage("custom");

    expect(screen.getByText("Extension locale")).toBeInTheDocument();
    expect(screen.queryByText("Plugin officiel")).not.toBeInTheDocument();
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

  it("conserve le contenu visible pendant un rafraîchissement", () => {
    renderPage("custom", records, vi.fn(), { loading: true });

    expect(screen.getByText("Extension locale")).toBeInTheDocument();
    expect(screen.getByRole("status"))
      .toHaveTextContent("extensions.loading");
  });

  it("conserve le contenu visible et affiche l'erreur de rafraîchissement", () => {
    renderPage("custom", records, vi.fn(), {
      loadError: "extensions.errors.load",
    });

    expect(screen.getByText("Extension locale")).toBeInTheDocument();
    expect(screen.getByRole("alert"))
      .toHaveTextContent("extensions.errors.load");
  });

  it("annonce le chargement initial lorsque la section est encore vide", () => {
    renderPage("custom", [], vi.fn(), { loading: true });

    expect(screen.getByRole("status"))
      .toHaveTextContent("extensions.loading");
    expect(screen.queryByText("extensions.pages.custom.empty"))
      .not.toBeInTheDocument();
  });

  it("annonce l'erreur initiale au lieu de la présenter comme une liste vide", () => {
    renderPage("custom", [], vi.fn(), { loadError: "extensions.errors.load" });

    expect(screen.getByRole("alert"))
      .toHaveTextContent("extensions.errors.load");
    expect(screen.queryByText("extensions.pages.custom.empty"))
      .not.toBeInTheDocument();
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
        recovery={recovery}
        hostBusy={false}
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
        onKeepDisabled={vi.fn()}
        onRetryLoad={vi.fn()}
        onDiscardMarker={vi.fn()}
        onRestoreSnapshot={vi.fn()}
        onPrioritySave={vi.fn(() => Promise.resolve(true))}
      />,
    );

    expect(screen.getByText(
      "extensions.errors.codes.extensions_host_unavailable",
    )).toBeInTheDocument();
  });

  it("expose les trois sections comme onglets en haut de page", () => {
    renderPage("plugins");

    const tabs = screen.getAllByRole("tab");

    expect(tabs.map((tab) => tab.textContent)).toEqual([
      "extensions.sections.plugins",
      "extensions.sections.custom",
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

  it("ouvre localement la fiche depuis la reprise et bloque les actions Hôte", () => {
    const onSelect = vi.fn();
    render(
      <ExtensionsPage
        section="custom"
        selected={null}
        onSelectSection={vi.fn()}
        records={records}
        host={host}
        loading={false}
        loadError={null}
        operationError={null}
        recovery={{ ...recovery, extensionId: "com.example.custom", canRetry: true }}
        hostBusy
        busyIds={new Set()}
        protectedPluginIds={[]}
        priorityBusy={false}
        onSelect={onSelect}
        onAdd={vi.fn()}
        onEnabled={vi.fn()}
        onShowInChat={vi.fn()}
        onOpenSource={vi.fn()}
        onUpdate={vi.fn()}
        onRemove={vi.fn()}
        onReload={vi.fn()}
        onRecover={vi.fn()}
        onKeepDisabled={vi.fn()}
        onRetryLoad={vi.fn()}
        onDiscardMarker={vi.fn()}
        onRestoreSnapshot={vi.fn()}
        onPrioritySave={vi.fn(() => Promise.resolve(true))}
      />,
    );

    fireEvent.click(screen.getByRole("button", {
      name: "extensions.recovery.openDetail",
    }));
    expect(onSelect).toHaveBeenCalledWith("com.example.custom");
    expect(screen.getByRole("button", { name: "extensions.recovery.retry" }))
      .toBeDisabled();
    expect(screen.getByRole("switch", { name: "extensions.enableFor" }))
      .toBeDisabled();
  });

  it("propose de quitter et relancer après un arrêt non confirmé", () => {
    render(
      <ExtensionsPage
        section="host"
        selected={null}
        onSelectSection={vi.fn()}
        records={records}
        host={{ ...host, state: "error", lastError: "extensions_stop_unconfirmed" }}
        loading={false}
        loadError={null}
        operationError={null}
        recovery={recovery}
        hostBusy={false}
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
        onKeepDisabled={vi.fn()}
        onRetryLoad={vi.fn()}
        onDiscardMarker={vi.fn()}
        onRestoreSnapshot={vi.fn()}
        onPrioritySave={vi.fn(() => Promise.resolve(true))}
      />,
    );

    expect(screen.getByText(
      "extensions.errors.codes.extensions_stop_unconfirmed",
    )).toBeInTheDocument();
    expect(screen.getByText("extensions.host.quitAndRestartHint")).toBeInTheDocument();
  });

  it("traduit l'indisponibilité du Hôte sans exposer le détail brut", () => {
    render(
      <ExtensionsPage
        section="host"
        selected={null}
        onSelectSection={vi.fn()}
        records={records}
        host={{ ...host, state: "error", lastError: "extensions_host_unavailable" }}
        loading={false}
        loadError={null}
        operationError={null}
        recovery={recovery}
        hostBusy={false}
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
        onKeepDisabled={vi.fn()}
        onRetryLoad={vi.fn()}
        onDiscardMarker={vi.fn()}
        onRestoreSnapshot={vi.fn()}
        onPrioritySave={vi.fn(() => Promise.resolve(true))}
      />,
    );

    expect(screen.getByRole("alert")).toHaveTextContent(
      "extensions.errors.codes.extensions_host_unavailable",
    );
    expect(screen.queryByText("extensions_host_unavailable")).not.toBeInTheDocument();
  });
});
