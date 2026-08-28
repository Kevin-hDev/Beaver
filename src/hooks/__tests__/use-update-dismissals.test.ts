import { describe, expect, it } from "vitest";
import { sameDismissedUpdate } from "../use-update-dismissals";

describe("update notification identities", () => {
  it("ne masque que la version exacte", () => {
    const hidden = { kind: "app" as const, subject: "beaver", version: "1.1.8" };
    expect(sameDismissedUpdate(hidden, hidden)).toBe(true);
    expect(sameDismissedUpdate(hidden, { ...hidden, version: "1.1.9" })).toBe(false);
  });

  it("distingue chaque modèle Ollama", () => {
    const hidden = { kind: "ollama_model" as const, subject: "llama3:latest", version: "abc" };
    expect(sameDismissedUpdate(hidden, { ...hidden, subject: "qwen3:latest" })).toBe(false);
  });
});
