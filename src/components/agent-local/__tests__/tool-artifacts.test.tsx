import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ToolArtifacts } from "../tool-artifacts";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en" },
    t: (key: string, values?: { name?: string }) => values?.name ?? key,
  }),
}));

const workspace = {
  name: "chart.png",
  mime_type: "image/png",
  bytes: 2048,
  sha256: "a".repeat(64),
  purpose: "artifact" as const,
  source: { kind: "workspace_file" as const, path: "/workspace/chart.png" },
};

describe("ToolArtifacts", () => {
  it("opens only a workspace artifact by keyboard and keeps resource provenance private", async () => {
    const user = userEvent.setup();
    const onFilePreview = vi.fn();
    render(<ToolArtifacts artifacts={[
      workspace,
      {
        ...workspace,
        name: "reference.pdf",
        source: {
          kind: "extension_resource" as const,
          resource_id: "extension:sample:reference",
        },
      },
    ]} onFilePreview={onFilePreview} />);

    const button = screen.getByRole("button", { name: "chart.png" });
    button.focus();
    await user.keyboard("{Enter}");
    expect(onFilePreview).toHaveBeenCalledWith("/workspace/chart.png");
    expect(screen.queryByRole("button", { name: "reference.pdf" })).toBeNull();
    expect(screen.getByText("reference.pdf")).toBeVisible();
  });

  it("shows the transient verification state without reading artifact bytes", () => {
    render(<ToolArtifacts artifacts={[
      { ...workspace, name: "intact.png", verification: "intact" },
      { ...workspace, name: "absent.png", verification: "absent" },
      { ...workspace, name: "modified.png", verification: "modified" },
      { ...workspace, name: "inaccessible.png", verification: "inaccessible" },
      { ...workspace, name: "unchecked.pdf", mime_type: "application/pdf" },
    ]} />);

    for (const status of ["intact", "absent", "modified", "inaccessible", "unverified"]) {
      expect(screen.getByText(`agentLocal.toolActivity.artifactStatus.${status}`)).toBeVisible();
    }
    expect(screen.getByText("application/pdf · 2 KiB")).toBeVisible();
  });
});
