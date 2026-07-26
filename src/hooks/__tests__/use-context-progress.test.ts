import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { useContextProgress } from "@/hooks/use-context-progress";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("useContextProgress — used", () => {
  it("retourne la valeur usedTokens passée en paramètre", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "show_ollama_model") return Promise.resolve({ modelfile: "", context_length: 0 });
      if (cmd === "get_effective_context_length") return Promise.resolve(0);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("llama3", 42, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.used).toBe(42);
    });
  });
});

describe("useContextProgress — provider ollama", () => {
  it("max vient en priorité du contexte réellement chargé par Ollama", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "get_loaded_ollama_context") return Promise.resolve(65536);
      if (cmd === "show_ollama_model")
        return Promise.resolve({ modelfile: "PARAMETER num_ctx 32768", context_length: 32768 });
      return Promise.resolve(0);
    });

    const { result } = renderHook(() =>
      useContextProgress("gemma4:e2b", 0, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(65536);
    });
    expect(invoke).not.toHaveBeenCalledWith("show_ollama_model", expect.anything());
  });

  it("max vient de num_ctx dans le modelfile", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "show_ollama_model")
        return Promise.resolve({ modelfile: "PARAMETER num_ctx 32768", context_length: 0 });
      if (cmd === "get_effective_context_length") return Promise.resolve(0);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("llama3", 0, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(32768);
    });
  });

  it("max vient de context_length si pas de num_ctx dans le modelfile", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "show_ollama_model")
        return Promise.resolve({ modelfile: "FROM llama3", context_length: 8192 });
      if (cmd === "get_effective_context_length") return Promise.resolve(0);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("llama3", 0, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(8192);
    });
  });

  it("max = min(context_length, effective_context) si les deux existent", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "show_ollama_model")
        return Promise.resolve({ modelfile: "FROM llama3", context_length: 16384 });
      if (cmd === "get_effective_context_length") return Promise.resolve(8192);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("llama3", 0, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(8192);
    });
  });

  it("max = 0 si le modèle retourne des valeurs vides", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "show_ollama_model")
        return Promise.resolve({ modelfile: "", context_length: 0 });
      if (cmd === "get_effective_context_length") return Promise.resolve(0);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("llama3", 0, "ollama"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(0);
    });
  });
});

describe("useContextProgress — provider codex-oauth", () => {
  it("max vient du context_length du modèle trouvé", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "codex_models")
        return Promise.resolve([
          { id: "gpt-4o", context_length: 128000 },
          { id: "o1", context_length: 200000 },
        ]);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("gpt-4o", 0, "codex-oauth"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(128000);
    });
  });

  it("max = 258000 par défaut si le modèle n'est pas trouvé", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "codex_models")
        return Promise.resolve([{ id: "autre-model", context_length: 4096 }]);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("modele-inconnu", 0, "codex-oauth"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(258000);
    });
  });
});

describe("useContextProgress — provider cloud (autre)", () => {
  it("max vient de list_llm_models pour un provider cloud", async () => {
    vi.mocked(invoke).mockImplementation((cmd: string) => {
      if (cmd === "list_llm_models")
        return Promise.resolve([
          { id: "claude-3-opus", context_length: 200000 },
          { id: "claude-3-sonnet", context_length: 200000 },
        ]);
      return Promise.resolve([]);
    });

    const { result } = renderHook(() =>
      useContextProgress("claude-3-opus", 0, "anthropic"),
    );

    await waitFor(() => {
      expect(result.current.max).toBe(200000);
    });
  });
});
