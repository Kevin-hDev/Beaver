import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionPriorityDialog } from "../extension-priority-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

function plugin(id: string, enabled = true): ExtensionRecord {
  return {
    manifest: {
      id,
      name: id,
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      access: "full",
      apiLevel: "stable",
      essential: false,
    },
    kind: "local",
    source: "/extension",
    enabled,
    trusted: true,
    showInChat: false,
    status: enabled ? "active" : "inactive",
    contributions: { tools: [], events: [] },
  };
}

describe("ExtensionPriorityDialog", () => {
  it("exclut les plugins désactivés et enregistre la sélection", () => {
    const onSave = vi.fn(() => Promise.resolve());
    render(
      <ExtensionPriorityDialog
        records={[plugin("example.enabled"), plugin("example.disabled", false)]}
        selectedIds={[]}
        busy={false}
        onCancel={vi.fn()}
        onSave={onSave}
      />,
    );

    fireEvent.click(screen.getByRole("checkbox"));
    fireEvent.click(screen.getByText("extensions.discovery.validate"));

    expect(screen.queryByText("example.disabled")).toBeNull();
    expect(onSave).toHaveBeenCalledWith(["example.enabled"]);
  });
});
