import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAgentMissingDirectory } from "../use-agent-missing-directory";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

describe("useAgentMissingDirectory", () => {
  it("rejoue l'envoi avec le dossier résolu par Switcher", async () => {
    vi.mocked(invoke).mockImplementation((command) => {
      if (command === "prepare_agent_send") {
        return Promise.resolve({
          status: "missing",
          missing_path: "/project/gone",
          nearest_parent: "/project",
        });
      }
      if (command === "resolve_missing_session_directory") {
        return Promise.resolve("/project");
      }
      return Promise.resolve(undefined);
    });
    const run = vi.fn(async (_workingDir?: string) => {});
    const { result } = renderHook(() => useAgentMissingDirectory("session-1"));

    await act(async () => {
      await result.current.runOrDefer("/project/gone", run);
    });
    await waitFor(() => expect(result.current.missingDirectory).not.toBeNull());
    await act(async () => {
      await result.current.resolve("switch");
    });

    expect(run).toHaveBeenCalledOnce();
    expect(run).toHaveBeenCalledWith("/project");
  });
});
