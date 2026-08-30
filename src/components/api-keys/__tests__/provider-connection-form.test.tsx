import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  DEFAULT_QWEN_CONNECTION,
  isQwenConnectionValid,
  ProviderConnectionForm,
} from "../provider-connection-form";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ProviderConnectionForm", () => {
  afterEach(cleanup);

  it("shows the regional controls and only requests a workspace when needed", () => {
    const onChange = vi.fn();
    render(<ProviderConnectionForm value={DEFAULT_QWEN_CONNECTION} onChange={onChange} />);

    expect(screen.getByLabelText("apiKeys.connection.region")).toBeTruthy();
    expect(screen.getByLabelText("apiKeys.connection.endpointMode")).toBeTruthy();
    expect(screen.queryByLabelText("apiKeys.connection.workspaceId")).toBeNull();

    fireEvent.click(screen.getByLabelText("apiKeys.connection.endpointMode"));
    fireEvent.click(screen.getByText("apiKeys.connection.modes.workspace"));
    expect(onChange).toHaveBeenCalledWith({
      region: "singapore",
      endpointMode: "workspace",
      workspaceId: undefined,
    });
  });

  it("rejects invalid workspaces and invalid region-mode pairs", () => {
    expect(isQwenConnectionValid({
      region: "singapore", endpointMode: "workspace", workspaceId: "team-42",
    })).toBe(true);
    expect(isQwenConnectionValid({
      region: "singapore", endpointMode: "workspace", workspaceId: "../team",
    })).toBe(false);
    expect(isQwenConnectionValid({ region: "tokyo", endpointMode: "shared" })).toBe(false);
  });
});
