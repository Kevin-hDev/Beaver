import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { ExtensionActivationDialog } from "../extension-activation-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const extension: ExtensionRecord = {
  manifest: {
    id: "com.example.untrusted",
    name: "Untrusted",
    version: "1.0.0",
    beaverApi: "1",
    runtime: "node",
    access: "full",
    apiLevel: "stable",
  },
  kind: "local",
  source: "/extension",
  enabled: false,
  trusted: false,
  showInChat: true,
  status: "inactive",
  contributions: { tools: [], events: [] },
};

describe("ExtensionActivationDialog", () => {
  it("requires an explicit confirmation before first activation", () => {
    const onConfirm = vi.fn();
    render(
      <ExtensionActivationDialog
        extension={extension}
        busy={false}
        onCancel={vi.fn()}
        onConfirm={onConfirm}
      />,
    );

    expect(screen.getByRole("dialog")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", {
      name: "extensions.activation.confirm",
    }));
    expect(onConfirm).toHaveBeenCalledOnce();
  });
});
