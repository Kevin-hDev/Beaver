import { describe, expect, it } from "vitest";
import errorContract from "./ollama-runtime-error-contract.json";
import { ollamaErrorKey, ollamaProgressKey } from "./ollama-runtime-error";

describe("Ollama runtime error mapper", () => {
  it("allowlists every public code and returns only an i18n key", () => {
    for (const code of Object.keys(errorContract)) {
      expect(ollamaErrorKey(code)).toBe("ollama.errors.generic");
    }
  });

  it.each([
    undefined,
    null,
    42,
    {},
    "not-a-public-code",
    "/Users/secret/stack.trace",
    "x".repeat(257),
  ])("maps unsafe input %j to the generic key", (value) => {
    expect(ollamaErrorKey(value)).toBe("ollama.errors.generic");
  });

  it("maps progress through local keys and never returns backend text", () => {
    expect(ollamaProgressKey("downloading")).toBe("ollamaSetup.downloading");
    expect(ollamaProgressKey("backend says /tmp/secret")).toBe("ollama.errors.generic");
  });
});
