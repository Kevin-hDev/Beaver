import { describe, expect, it } from "vitest";
import { toolResultForModel } from "./tool-result-model";

describe("toolResultForModel", () => {
  it("conserve une réussite simple sans enveloppe", () => {
    expect(toolResultForModel({ name: "read_file", result: "hello" })).toBe("hello");
  });

  it("sépare les métadonnées de la sortie brute", () => {
    const output = '{"large":"value"}\nnext line';
    const rendered = toolResultForModel({
      name: "forecast_read",
      result: output,
      result_meta: {
        status: "partial",
        warnings: ["incomplet"],
        truncated: true,
      },
    });
    const newline = rendered.indexOf("\n");
    const metadata = JSON.parse(rendered.slice(0, newline)) as Record<string, unknown>;

    expect(metadata.outputFormat).toBe("raw_following");
    expect(metadata.output).toBeUndefined();
    expect(rendered.slice(newline + 1)).toBe(output);
  });
});
