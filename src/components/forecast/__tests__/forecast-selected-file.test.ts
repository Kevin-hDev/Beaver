import { beforeEach, describe, expect, it, vi } from "vitest";
import { loadForecastDraftFromFile } from "../forecast-data";

const mocks = vi.hoisted(() => ({ invoke: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({ invoke: mocks.invoke }));

describe("forecast selected file access", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.invoke.mockResolvedValue(JSON.stringify({
      headers: ["date", "value"],
      rows: [["2026-01-01", "12"]],
      total_rows: 1,
    }));
  });

  it("utilise la commande limitée au fichier choisi", async () => {
    const draft = await loadForecastDraftFromFile("/tmp/data.csv");

    expect(mocks.invoke).toHaveBeenCalledWith("read_selected_spreadsheet_preview", {
      path: "/tmp/data.csv",
      maxRows: 5000,
    });
    expect(draft.rows[0].value).toBe(12);
  });
});
