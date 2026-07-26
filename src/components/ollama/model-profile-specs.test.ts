import { describe, it, expect } from "vitest";
import { buildSpecRows } from "./model-profile-specs";
import type { RegistryModelDetails, RegistryTag, ModelInfo } from "@/types/agent";

const t = (k: string) => k;

describe("buildSpecRows", () => {
  it("retourne un tableau vide quand toutes les entrées sont null", () => {
    expect(buildSpecRows(t, null, null, null)).toEqual([]);
  });

  it("inclut les capabilities quand présentes", () => {
    const details: RegistryModelDetails = {
      capabilities: ["tools", "vision"],
    } as RegistryModelDetails;
    const rows = buildSpecRows(t, details, null, null);

    const capRow = rows.find((r) => r.label === "ollama.capabilities");
    expect(capRow?.value).toBe("tools, vision");
  });

  it("inclut la taille du fichier depuis le tag", () => {
    const tag = { size_gb: 4.7 } as RegistryTag;
    const rows = buildSpecRows(t, null, tag, null);

    const sizeRow = rows.find((r) => r.label === "ollama.fileSize");
    expect(sizeRow?.value).toBe("4.7 GB");
  });

  it("inclut la taille des paramètres depuis info", () => {
    const info = { parameter_size: "7B" } as ModelInfo;
    const rows = buildSpecRows(t, null, null, info);

    const paramRow = rows.find((r) => r.label === "ollama.paramsLabel");
    expect(paramRow?.value).toBe("7B");
  });

  it("calcule le contexte en K (divisé par 1024)", () => {
    const tag = { context_length: 8192 } as RegistryTag;
    const rows = buildSpecRows(t, null, tag, null);

    const ctxRow = rows.find((r) => r.label === "ollama.context");
    // 8192 / 1024 = 8 → "8" + suffix tokens
    expect(ctxRow?.value).toContain("8");
    expect(ctxRow?.value).toContain("ollama.contextTokens");
  });

  it("privilégie le context_length du tag sur details", () => {
    const tag = { context_length: 4096 } as RegistryTag;
    const details = { context_length: 2048 } as RegistryModelDetails;
    const rows = buildSpecRows(t, details, tag, null);

    const ctxRow = rows.find((r) => r.label === "ollama.context");
    // 4096 / 1024 = 4 (tag prioritaire sur details 2048 → 2)
    expect(ctxRow?.value).toContain("4");
  });

  it("inclut la quantization depuis info", () => {
    const info = { quantization: "Q4_K_M" } as ModelInfo;
    const rows = buildSpecRows(t, null, null, info);

    const quantRow = rows.find((r) => r.label === "ollama.quantization");
    expect(quantRow?.value).toBe("Q4_K_M");
  });

  it("marque le digest comme mono", () => {
    const tag = { digest_short: "abc123de" } as RegistryTag;
    const rows = buildSpecRows(t, null, tag, null);

    const digestRow = rows.find((r) => r.label === "ollama.digest");
    expect(digestRow?.value).toBe("abc123de");
    expect(digestRow?.mono).toBe(true);
  });

  it("inclut MoE yes/no quand info présent", () => {
    const infoMoe = { is_moe: true } as ModelInfo;
    const rowsMoe = buildSpecRows(t, null, null, infoMoe);
    const moeRow = rowsMoe.find((r) => r.label === "ollama.moe");
    expect(moeRow?.value).toBe("ollama.yes");

    const infoNoMoe = { is_moe: false } as ModelInfo;
    const rowsNoMoe = buildSpecRows(t, null, null, infoNoMoe);
    const moeRowNo = rowsNoMoe.find((r) => r.label === "ollama.moe");
    expect(moeRowNo?.value).toBe("ollama.no");
  });

  it("combine plusieurs sources dans une seule liste", () => {
    const details = { capabilities: ["tools"], context_length: 4096 } as RegistryModelDetails;
    const tag = { size_gb: 2.0 } as RegistryTag;
    const info = { parameter_size: "3B", quantization: "Q4" } as ModelInfo;
    const rows = buildSpecRows(t, details, tag, info);

    expect(rows.length).toBeGreaterThanOrEqual(5);
    const labels = rows.map((r) => r.label);
    expect(labels).toContain("ollama.capabilities");
    expect(labels).toContain("ollama.fileSize");
    expect(labels).toContain("ollama.paramsLabel");
    expect(labels).toContain("ollama.context");
    expect(labels).toContain("ollama.quantization");
  });
});
