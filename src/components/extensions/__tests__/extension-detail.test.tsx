import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionDetail } from "../extension-detail";

const catalog = vi.hoisted(() => ({
  current: null as null | Record<string, unknown>,
}));
const startup = vi.hoisted(() => ({
  current: null as null | Record<string, unknown>,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));
vi.mock("@/features/extension-ui/standard/catalog-context", () => ({
  useOptionalStandardCatalog: () => catalog.current,
}));
vi.mock("@/hooks/use-extension-ui-startup", () => ({
  useExtensionUiStartupContext: () => startup.current,
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
  beforeEach(() => {
    catalog.current = null;
    startup.current = null;
  });
  it("shows replacement status and the prompt-visible tool description", () => {
    render(
      <ExtensionDetail
        extension={extension}
        diagnostics={[]}
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

  it("intègre les métadonnées des skills et ressources sans leurs chemins", () => {
    const withCapabilities: ExtensionRecord = {
      ...extension,
      contributions: {
        ...extension.contributions,
        skills: [{
          id: "guide",
          name: "Guide",
          description: "A concise guide.",
          path: "skills/private-guide.md",
        }],
        resources: [{
          id: "preview",
          name: "Preview",
          description: "An image preview.",
          type: "image",
          path: "resources/private-preview.png",
        }],
      },
    };
    renderDetail(withCapabilities);

    expect(screen.getByText("Guide")).toBeInTheDocument();
    expect(screen.getByText("Preview")).toBeInTheDocument();
    expect(screen.queryByText("skills/private-guide.md")).not.toBeInTheDocument();
    expect(screen.queryByText("resources/private-preview.png")).not.toBeInTheDocument();
  });

  it("reste affichable si une ancienne réponse IPC omet les contributions", () => {
    const incomplete = {
      ...extension,
      contributions: undefined,
    } as unknown as ExtensionRecord;

    render(
      <ExtensionDetail
        extension={incomplete}
        diagnostics={[]}
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
        diagnostics={[]}
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
        diagnostics={[]}
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

  it("affiche les contributions standard, leurs emplacements et leurs thèmes", () => {
    const standard = {
      ...extension,
      manifest: {
        ...extension.manifest,
        ui: { apiVersion: "1", mode: "standard" as const },
      },
    };
    catalog.current = {
      state: {
        kind: "ready",
        snapshot: {
          revision: 1,
          contributions: [{
            extensionId: extension.manifest.id,
            contributionId: `${extension.manifest.id}.toolbar`,
            contribution: {
              type: "action",
              id: `${extension.manifest.id}.toolbar`,
              placement: "app.toolbar.primary",
              order: 0,
              label: { default: "Toolbar action" },
              actionId: `${extension.manifest.id}.run`,
            },
          }, {
            extensionId: extension.manifest.id,
            contributionId: `${extension.manifest.id}.theme`,
            contribution: {
              type: "theme",
              id: `${extension.manifest.id}.theme`,
              order: 0,
              label: { default: "Search blue" },
              base: "dark",
              tokens: { "--ink": "#FFFFFF" },
            },
          }],
        },
      },
    };

    renderDetail(standard);

    expect(screen.getByText("extensions.uiModes.standard")).toBeInTheDocument();
    expect(screen.getByText("Toolbar action")).toBeInTheDocument();
    expect(screen.getByText("extensions.uiPlacements.app_toolbar_primary"))
      .toBeInTheDocument();
    expect(screen.getByText("Search blue")).toBeInTheDocument();
  });

  it("distingue chargement, erreur et catalogue standard vide", () => {
    const standard = {
      ...extension,
      manifest: {
        ...extension.manifest,
        ui: { apiVersion: "1", mode: "standard" as const },
      },
    };
    catalog.current = { state: { kind: "loading", snapshot: null } };
    const loading = renderDetail(standard);
    expect(screen.getByRole("status")).toHaveTextContent("extensions.detail.uiLoading");
    loading.unmount();

    catalog.current = { state: { kind: "error", snapshot: null } };
    const failed = renderDetail(standard);
    expect(screen.getByRole("alert")).toHaveTextContent("extensions.detail.uiError");
    expect(screen.getByRole("button", { name: "extensions.actions.retry" })).toBeEnabled();
    failed.unmount();

    catalog.current = {
      state: { kind: "empty", snapshot: { revision: 1, contributions: [] } },
    };
    renderDetail(standard);
    expect(screen.getByText("extensions.detail.uiEmpty")).toBeInTheDocument();
  });

  it("signale un artefact avancé absent et rappelle le risque total", () => {
    const advanced = {
      ...extension,
      manifest: {
        ...extension.manifest,
        ui: { apiVersion: "1", mode: "advanced" as const, entry: "ui.tsx" },
      },
      uiArtifact: undefined,
    };

    renderDetail(advanced);

    expect(screen.getByText("extensions.uiModes.advanced")).toBeInTheDocument();
    expect(screen.getByText("extensions.detail.uiArtifactMissing")).toBeInTheDocument();
    expect(screen.getByText("extensions.detail.uiAdvancedWarning")).toBeInTheDocument();
  });

  it("affiche l'incident UI sous une forme générique et horodatée", () => {
    const interruptedAt = "2026-09-03T10:15:00Z";
    startup.current = {
      incident: {
        extensionId: extension.manifest.id,
        stage: "activate",
        attempts: 1,
        startedAt: interruptedAt,
      },
    };
    const expected = new Intl.DateTimeFormat("fr", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(interruptedAt));

    renderDetail({
      ...extension,
      manifest: {
        ...extension.manifest,
        ui: { apiVersion: "1", mode: "advanced", entry: "ui.tsx" },
      },
    });

    expect(screen.getByText("extensions.diagnostics.uiGeneric")).toBeInTheDocument();
    expect(screen.getByText(expected)).toBeInTheDocument();
    expect(screen.queryByText("activate")).not.toBeInTheDocument();
  });

  it("affiche le dernier diagnostic UI réel reçu du Hôte", () => {
    const occurredAt = "2026-09-03T11:15:00Z";
    const expected = new Intl.DateTimeFormat("fr", {
      dateStyle: "medium",
      timeStyle: "short",
    }).format(new Date(occurredAt));

    renderDetail({
      ...extension,
      manifest: {
        ...extension.manifest,
        ui: { apiVersion: "1", mode: "standard" },
      },
    }, [{
      extensionId: extension.manifest.id,
      stage: "register",
      code: "ui_contribution_invalid",
      occurredAt,
    }]);

    expect(screen.getByText("extensions.diagnostics.uiGeneric")).toBeInTheDocument();
    expect(screen.getByText(expected)).toBeInTheDocument();
    expect(screen.queryByText("ui_contribution_invalid")).not.toBeInTheDocument();
  });
});

function renderDetail(
  value: ExtensionRecord,
  diagnostics: React.ComponentProps<typeof ExtensionDetail>["diagnostics"] = [],
) {
  return render(
    <ExtensionDetail
      extension={value}
      diagnostics={diagnostics}
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
}
