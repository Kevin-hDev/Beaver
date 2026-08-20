import { act, renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { useAgentMissingDirectory } from "../use-agent-missing-directory";
import { showToast } from "@/lib/toast-emitter";

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

  it("bloque l’envoi quand le dossier sort des racines autorisées", async () => {
    vi.mocked(invoke).mockResolvedValue({
      status: "forbidden",
      allowed_paths: ["/project/allowed"],
    });
    const run = vi.fn(async (_workingDir?: string) => {});
    const { result } = renderHook(() => useAgentMissingDirectory("session-1"));

    await act(async () => {
      await result.current.runOrDefer("/project", run);
    });

    expect(run).not.toHaveBeenCalled();
    expect(result.current.forbiddenAllowedPaths).toEqual(["/project/allowed"]);
    act(() => result.current.dismissForbidden());
    expect(result.current.forbiddenAllowedPaths).toBeNull();
  });

  it("traduit le refus lecture seule du préflight", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("subagent-read-only"));
    const { result } = renderHook(() => useAgentMissingDirectory("child-session"));

    await act(async () => {
      await result.current.runOrDefer(undefined, vi.fn());
    });

    expect(showToast).toHaveBeenCalledWith("errors.admission.subagentReadOnly", "error");
  });
});
