import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { parseExtensionRecords } from "@/lib/extension-records";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionsPage } from "../extensions-page";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));

const record = parseExtensionRecords([{
  manifest: {
    id: "beaver.office.documents",
    name: "Documents",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    main: "builtin-plugins/documents/index.mjs",
    ui: null,
    access: "full",
    apiLevel: "stable",
    essential: false,
    author: "Beaver",
    homepage: null,
    description: "Create documents.",
  },
  kind: "builtin",
  source: "Beaver",
  enabled: true,
  trusted: true,
  showInChat: true,
  status: "active",
  lastError: "extensions_host_timeout",
  lastActivatedAt: "2026-09-01T10:00:00Z",
  trustedAt: "2026-08-31T09:00:00Z",
  contributions: {
    tools: [{
      name: "beaver.office.documents.create",
      description: "Create a document.",
      parameters: { type: "object" },
      effect: "secret",
      replacesCore: false,
    }],
    events: [],
  },
}])[0];

function SelectablePage({ operationError = null }: { operationError?: string | null }) {
  const [selected, setSelected] = useState<ExtensionRecord | null>(null);
  return (
    <ExtensionsPage
      section="plugins"
      selected={selected}
      onSelectSection={vi.fn()}
      records={[record]}
      host={{
        state: "running",
        jitiVersion: "2.7.0",
        apiVersion: "1",
        activeExtensions: 1,
        diagnostics: [],
      }}
      hostLoaded
      loading={false}
      loadError={null}
      operationError={operationError}
      recovery={{ extensionId: null, stage: null, attempts: null, canRetry: false, markerInvalid: false, recoverySnapshotAvailable: false }}
      hostBusy={false}
      busyIds={new Set()}
      protectedPluginIds={[]}
      priorityBusy={false}
      onSelect={(id) => setSelected(id ? record : null)}
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
    />
  );
}

describe("Extension detail flow", () => {
  it("ouvre le détail depuis la ligne avec les contributions IPC", () => {
    render(<SelectablePage />);

    fireEvent.click(screen.getByRole("button", {
      name: /extensions\.official\.documents\.name/,
    }));

    expect(screen.getByText("extensions.detail.contributions"))
      .toBeInTheDocument();
    expect(screen.getByText("beaver.office.documents.create"))
      .toBeInTheDocument();
  });

  it("affiche l'erreur d'opération au-dessus de la fiche ouverte", () => {
    render(
      <SelectablePage
        operationError="extensions.errors.codes.extensions_operation_failed"
      />,
    );

    fireEvent.click(screen.getByRole("button", {
      name: /extensions\.official\.documents\.name/,
    }));

    const alert = screen.getByRole("alert");
    expect(alert).toHaveTextContent(
      "extensions.errors.codes.extensions_operation_failed",
    );
    expect(alert.compareDocumentPosition(screen.getByText("extensions.detail.status")))
      .toBe(Node.DOCUMENT_POSITION_FOLLOWING);
  });

  it("distingue activation, chargement, Hôte et effets sensibles", () => {
    render(<SelectablePage />);
    fireEvent.click(screen.getByRole("button", {
      name: /extensions\.official\.documents\.name/,
    }));

    expect(screen.getByText("extensions.detail.enabled")).toBeInTheDocument();
    expect(screen.getByText("extensions.detail.apiLevel")).toBeInTheDocument();
    const formatter = new Intl.DateTimeFormat("fr", {
      dateStyle: "medium",
      timeStyle: "short",
    });
    expect(screen.getByText(formatter.format(new Date("2026-09-01T10:00:00Z"))))
      .toBeInTheDocument();
    expect(screen.getByText(formatter.format(new Date("2026-08-31T09:00:00Z"))))
      .toBeInTheDocument();
    expect(screen.getByText("extensions.errors.codes.extensions_host_timeout"))
      .toBeInTheDocument();
    expect(screen.getByText("extensions.effects.secret")).toBeInTheDocument();
  });
});
