import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useContextHiddenUsage } from "../use-context-hidden-usage";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const invokeMock = vi.mocked(invoke);

describe("use-context-hidden-usage", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("charge les blocs de contexte cachés depuis Rust", async () => {
    invokeMock.mockResolvedValue({
      systemPromptTokens: 10,
      metaContextTokens: 20,
      skillContextTokens: 30,
      systemToolDefinitionTokens: 40,
      mcpDefinitionTokens: 50,
    });

    const { result } = renderHook(() => useContextHiddenUsage({
      sessionId: "s1",
      model: "llama3",
      provider: "ollama",
      workingDir: "/tmp/project",
      permissionMode: "auto",
      planMode: true,
      supportsTools: true,
    }));

    await waitFor(() => expect(result.current.systemPromptTokens).toBe(10));
    expect(result.current).toEqual({
      systemPromptTokens: 10,
      metaContextTokens: 20,
      skillContextTokens: 30,
      systemToolDefinitionTokens: 40,
      mcpDefinitionTokens: 50,
    });
    expect(invokeMock).toHaveBeenCalledWith("estimate_context_hidden_usage", {
      sessionId: "s1",
      model: "llama3",
      provider: "ollama",
      workingDir: "/tmp/project",
      permissionMode: "auto",
      planMode: true,
      supportsTools: true,
    });
  });

  it("garde un fallback vide si la commande échoue", async () => {
    invokeMock.mockRejectedValue(new Error("failed"));

    const { result } = renderHook(() => useContextHiddenUsage({
      sessionId: "s1",
      model: "llama3",
      provider: "ollama",
    }));

    await waitFor(() => expect(invokeMock).toHaveBeenCalledTimes(1));
    expect(result.current).toEqual({});
  });
});
