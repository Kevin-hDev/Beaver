import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { parseExtensionRecords } from "@/lib/extension-records";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionsPage } from "../extensions-page";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
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
  lastError: null,
  lastActivatedAt: null,
  contributions: {
    tools: [{
      name: "beaver.office.documents.create",
      description: "Create a document.",
      parameters: { type: "object" },
      replacesCore: false,
    }],
    events: [],
  },
}])[0];

function SelectablePage() {
  const [selected, setSelected] = useState<ExtensionRecord | null>(null);
  return (
    <ExtensionsPage
      section="plugins"
      selected={selected}
      records={[record]}
      host={{
        state: "running",
        jitiVersion: "2.7.0",
        apiVersion: "1",
        activeExtensions: 1,
        diagnostics: [],
      }}
      loading={false}
      loadError={null}
      operationError={null}
      busyIds={new Set()}
      onSelect={(id) => setSelected(id ? record : null)}
      onAdd={vi.fn()}
      onEnabled={vi.fn()}
      onShowInChat={vi.fn()}
      onOpenSource={vi.fn()}
      onRemove={vi.fn()}
      onReload={vi.fn()}
      onRecover={vi.fn()}
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
});
