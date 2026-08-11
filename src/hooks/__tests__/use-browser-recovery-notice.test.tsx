import { invoke } from "@tauri-apps/api/core";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ToastProvider } from "@/components/ui/toast";
import "@/i18n";
import { BrowserCapabilityProvider } from "../use-browser-capability";
import { useBrowserRecoveryNotice } from "../use-browser-recovery-notice";

function RecoveryHarness() {
  useBrowserRecoveryNotice();
  return <button type="button">Continue working</button>;
}

describe("useBrowserRecoveryNotice", () => {
  beforeEach(() => {
    vi.mocked(invoke).mockReset().mockImplementation((command) => {
      if (command === "browser_capability") {
        return Promise.resolve({
          status: "unavailable",
          restartRecommended: true,
        });
      }
      return Promise.resolve(undefined);
    });
  });

  it("reste non bloquante et redÃ©marre par la commande coordonnÃ©e", async () => {
    render(
      <ToastProvider>
        <BrowserCapabilityProvider>
          <RecoveryHarness />
        </BrowserCapabilityProvider>
      </ToastProvider>,
    );

    expect(screen.getByRole("button", { name: "Continue working" })).toBeEnabled();
    expect(await screen.findByText(/integrated browser could not start/i)).toBeVisible();

    fireEvent.click(screen.getByRole("button", { name: /restart/i }));
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("restart_application"));
    expect(screen.queryByText(/integrated browser could not start/i)).not.toBeInTheDocument();
  });

  it("ne propose rien tant qu'un redÃ©marrage n'est pas utile", async () => {
    vi.mocked(invoke).mockResolvedValue({
      status: "unavailable",
      restartRecommended: false,
    });

    render(
      <ToastProvider>
        <BrowserCapabilityProvider>
          <RecoveryHarness />
        </BrowserCapabilityProvider>
      </ToastProvider>,
    );

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("browser_capability"));
    expect(screen.queryByRole("button", { name: /restart/i })).not.toBeInTheDocument();
  });
});
