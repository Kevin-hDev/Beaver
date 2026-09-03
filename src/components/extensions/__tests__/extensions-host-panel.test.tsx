import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ExtensionsHostPanel } from "../extensions-host-panel";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ExtensionsHostPanel", () => {
  it("shows a translated generic message instead of a raw UI diagnostic code", () => {
    render(
      <ExtensionsHostPanel
        host={{
          state: "running",
          nodeVersion: "v24.18.0",
          jitiVersion: "2.7.0",
          apiVersion: "1",
          activeExtensions: 1,
          diagnostics: [{
            extensionId: "com.example.ui",
            stage: "register",
            code: "ui_contribution_invalid",
          }],
        }}
        loaded
        loading={false}
        loadError={null}
        busy={false}
        onRestart={vi.fn()}
        onRecover={vi.fn()}
      />,
    );

    expect(screen.getByText("extensions.diagnostics.uiGeneric")).toBeInTheDocument();
    expect(screen.queryByText("extensions.diagnostics.codes.ui_contribution_invalid"))
      .not.toBeInTheDocument();
  });

  it("keeps the actionable legacy UI diagnostic distinct", () => {
    render(
      <ExtensionsHostPanel
        host={{
          state: "running",
          nodeVersion: "v24.18.0",
          jitiVersion: "2.7.0",
          apiVersion: "1",
          activeExtensions: 1,
          diagnostics: [{
            extensionId: "com.example.legacy",
            stage: "register",
            code: "ui_manifest_legacy",
          }],
        }}
        loaded
        loading={false}
        loadError={null}
        busy={false}
        onRestart={vi.fn()}
        onRecover={vi.fn()}
      />,
    );

    expect(screen.getByText("extensions.diagnostics.codes.ui_manifest_legacy"))
      .toBeInTheDocument();
    expect(screen.queryByText("extensions.diagnostics.uiGeneric")).not.toBeInTheDocument();
  });
});
