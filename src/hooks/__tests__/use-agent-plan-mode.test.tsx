import { act, renderHook } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { describe, expect, it, vi } from "vitest";
import { showToast } from "@/lib/toast-emitter";
import { useAgentPlanMode } from "../use-agent-plan-mode";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@/lib/toast-emitter", () => ({ showToast: vi.fn() }));
vi.mock("@/i18n", () => ({ default: { t: (key: string) => key } }));

describe("useAgentPlanMode", () => {
  it("traduit le refus lecture seule et restaure l'état précédent", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("subagent-read-only"));
    const setChatState = vi.fn();
    const { result } = renderHook(() => useAgentPlanMode("child-session", setChatState));

    await act(async () => {
      await result.current.setEnabled(true);
    });

    expect(result.current.enabled).toBe(false);
    expect(showToast).toHaveBeenCalledWith("errors.admission.subagentReadOnly", "error");
  });
});
