import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ChannelsConfigDialog } from "../channels-config-dialog";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ChannelsConfigDialog", () => {
  beforeEach(() => vi.mocked(invoke).mockReset().mockResolvedValue(undefined));

  it("enregistre les deux tokens Slack en une seule opération", async () => {
    const { container } = render(
      <ChannelsConfigDialog channelId="slack" onClose={vi.fn()} onSaved={vi.fn()} />,
    );
    const inputs = container.querySelectorAll("input");
    fireEvent.change(inputs[0], { target: { value: "work" } });
    fireEvent.change(inputs[1], { target: { value: "xoxb-test" } });
    fireEvent.change(inputs[2], { target: { value: "xapp-test" } });
    fireEvent.click(screen.getByText("channels.config.addAndTest"));

    await waitFor(() => expect(invoke).toHaveBeenCalledTimes(1));
    expect(invoke).toHaveBeenCalledWith("gateway_configure_account_tokens", {
      channelId: "slack",
      accountId: "work",
      credentials: { botToken: "xoxb-test", appToken: "xapp-test" },
    });
    await waitFor(() => {
      expect(inputs[1]).toHaveValue("");
      expect(inputs[2]).toHaveValue("");
    });
  });

  it("efface le token après un échec et à la fermeture", async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error("secret detail"));
    const onClose = vi.fn();
    const { container } = render(
      <ChannelsConfigDialog channelId="telegram" onClose={onClose} onSaved={vi.fn()} />,
    );
    const inputs = container.querySelectorAll("input");
    fireEvent.change(inputs[0], { target: { value: "main" } });
    fireEvent.change(inputs[1], { target: { value: "secret-token" } });
    fireEvent.click(screen.getByText("channels.config.addAndTest"));
    await waitFor(() => expect(inputs[1]).toHaveValue(""));

    fireEvent.change(inputs[1], { target: { value: "second-secret" } });
    fireEvent.keyDown(window, { key: "Escape" });
    expect(inputs[1]).toHaveValue("");
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
