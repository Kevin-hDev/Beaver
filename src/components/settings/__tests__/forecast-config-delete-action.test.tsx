/* @vitest-environment jsdom */
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ForecastConfigDeleteAction } from "../forecast-config-delete-action";

const invoke = vi.hoisted(() => vi.fn());
const showToast = vi.hoisted(() => vi.fn());

vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@/lib/toast-emitter", () => ({ showToast }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("ForecastConfigDeleteAction", () => {
  beforeEach(() => invoke.mockResolvedValue(undefined));

  afterEach(() => {
    cleanup();
    vi.clearAllMocks();
  });

  it("supprime le modele apres confirmation", async () => {
    const onDeleted = vi.fn();
    render(
      <ForecastConfigDeleteAction
        modelId="chronos-bolt-tiny"
        disabled={false}
        onDeleted={onDeleted}
      />,
    );

    fireEvent.click(screen.getByText("forecast.models.uninstall"));
    fireEvent.click(screen.getByText("forecast.models.confirmUninstall"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "uninstall_forecast_model",
      { name: "chronos-bolt-tiny" },
    ));
    expect(onDeleted).toHaveBeenCalledOnce();
  });

  it("reste ferme en cas d'echec", async () => {
    invoke.mockRejectedValueOnce(new Error("failure"));
    render(
      <ForecastConfigDeleteAction
        modelId="chronos-bolt-tiny"
        disabled={false}
        onDeleted={vi.fn()}
      />,
    );

    fireEvent.click(screen.getByText("forecast.models.uninstall"));
    fireEvent.click(screen.getByText("forecast.models.confirmUninstall"));

    await waitFor(() => expect(showToast).toHaveBeenCalledWith("errors.operationFailed", "error"));
  });
});
