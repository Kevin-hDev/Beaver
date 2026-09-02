import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { PermissionDialog } from "../permission-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, options?: Record<string, string>) => {
      if (key === "permissionDialog.extensionAction") return `use ${options?.extension}`;
      if (key === "permissionDialog.title") return `Allow ${options?.action}?`;
      if (key === "permissionDialog.effects.external-write") return "External write";
      return key;
    },
  }),
}));

describe("PermissionDialog", () => {
  it("uses only the extension display prepared by Rust", () => {
    render(<PermissionDialog request={{
      id: "request", toolName: "misleading.read_file", arguments: {},
      extensionId: "plugin-id", extensionName: "Safe Plugin",
      effectClass: "external-write", actionSummary: "{\"target\":\"summary\"}",
    }} onDecide={vi.fn()} />);

    expect(screen.getByText("Allow use Safe Plugin?")).toBeTruthy();
    expect(screen.getByText("External write")).toBeTruthy();
    expect(screen.getByText("{\"target\":\"summary\"}")).toBeTruthy();
    expect(screen.queryByText("misleading.read_file")).toBeNull();
  });

  it("returns the selected decision", () => {
    const onDecide = vi.fn();
    render(<PermissionDialog request={{
      id: "request", toolName: "plugin.tool", arguments: {},
      extensionId: "plugin-id", extensionName: "Plugin",
      effectClass: "local-write", actionSummary: "{}",
    }} onDecide={onDecide} />);

    fireEvent.click(screen.getByText("permissionDialog.allow"));
    expect(onDecide).toHaveBeenCalledWith("request", "allow");
  });

  it("reuses the web fetch confirmation wording for external reads", () => {
    render(<PermissionDialog request={{
      id: "request", toolName: "plugin.fetch", arguments: {},
      extensionId: "plugin-id", extensionName: "Network Plugin",
      effectClass: "external-read", actionSummary: "{}",
    }} onDecide={vi.fn()} />);

    expect(screen.getByText("Allow permissionDialog.tools.web_fetch?")).toBeTruthy();
    expect(screen.getByText("Network Plugin")).toBeTruthy();
  });

  it("does not offer a persistent permission when Rust forbids caching it", () => {
    render(<PermissionDialog request={{
      id: "request", toolName: "plugin.secret", arguments: {},
      extensionId: "plugin-id", extensionName: "Plugin",
      effectClass: "secret", actionSummary: "{}", allowSession: false,
    }} onDecide={vi.fn()} />);

    expect(screen.queryByText("permissionDialog.allowSession")).toBeNull();
    expect(screen.getByText("permissionDialog.allow")).toBeTruthy();
  });

  it("requires an explicit cache grant for extension tools", () => {
    render(<PermissionDialog request={{
      id: "request", toolName: "plugin.process", arguments: {},
      extensionId: "plugin-id", extensionName: "Plugin",
      effectClass: "process", actionSummary: "{}",
    }} onDecide={vi.fn()} />);

    expect(screen.queryByText("permissionDialog.allowSession")).toBeNull();
  });
});
