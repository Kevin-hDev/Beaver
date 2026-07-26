import { describe, it, expect } from "vitest";
import { sanitizeToolError } from "./tool-error-sanitize";

// --- Rédaction des secrets -------------------------------------------------

describe("sanitizeToolError - secrets", () => {
  it("rédige un token Bearer", () => {
    const result = sanitizeToolError("Erreur 401: Bearer sk-abc123def456ghi789");
    expect(result).not.toContain("sk-abc123def456ghi789");
    expect(result).toContain("[redacted]");
  });

  it("rédige un api_key au format key=value", () => {
    const result = sanitizeToolError("api_key=sk-secret123456");
    expect(result).not.toContain("sk-secret123456");
    expect(result).toContain("[redacted]");
  });

  it("rédige un token au format token: value", () => {
    const result = sanitizeToolError("token: my-secret-token-12345");
    expect(result).not.toContain("my-secret-token-12345");
  });

  it("rédige un password", () => {
    const result = sanitizeToolError("password=hunter2password");
    expect(result).not.toContain("hunter2password");
  });

  it("rédige un secret_key", () => {
    const result = sanitizeToolError("secret_key=mykey12345678");
    expect(result).not.toContain("mykey12345678");
  });
});

// --- Rédaction des chemins -------------------------------------------------

describe("sanitizeToolError - paths", () => {
  it("rédige un chemin Unix /Users/", () => {
    const result = sanitizeToolError("File not found: /Users/kevin/secret.txt");
    expect(result).not.toContain("/Users/kevin/secret.txt");
    expect(result).toContain("[path]");
  });

  it("rédige un chemin Windows C:\\", () => {
    const result = sanitizeToolError("Cannot open C:\\Users\\admin\\config.json");
    expect(result).not.toContain("C:\\Users\\admin\\config.json");
    expect(result).toContain("[path]");
  });

  it("ne rédactionne pas un chemin relatif sans /Users/", () => {
    // Les chemins relatifs (./src/main) ne sont pas rédactionnés.
    const result = sanitizeToolError("Error in ./src/main.ts");
    expect(result).toContain("./src/main.ts");
  });
});

// --- Troncature ------------------------------------------------------------

describe("sanitizeToolError - truncation", () => {
  it("tronque à 300 caractères + ...", () => {
    const long = "Error: " + "x".repeat(400);
    const result = sanitizeToolError(long);
    expect(result.length).toBeLessThanOrEqual(303); // 300 + "..."
    expect(result.endsWith("...")).toBe(true);
  });

  it("ne tronque pas un message court", () => {
    const result = sanitizeToolError("Error: short message");
    expect(result).toBe("Error: short message");
    expect(result.endsWith("...")).toBe(false);
  });

  it("utilise seulement la première ligne non vide", () => {
    const input = "Error: first line\nstack trace line 2\nmore details";
    const result = sanitizeToolError(input);
    expect(result).toBe("Error: first line");
    expect(result).not.toContain("stack trace");
  });

  it("ignore les lignes vides strictes en tête", () => {
    // find(Boolean) skip les "" mais conserve "  " (string non vide = truthy).
    // On teste donc avec des lignes vides strictes.
    const input = "\n\n\nError: real message";
    const result = sanitizeToolError(input);
    expect(result).toBe("Error: real message");
  });
});

// --- Combinaison secret + path + troncature --------------------------------

describe("sanitizeToolError - combinations", () => {
  it("rédige ET un secret ET un chemin dans le même message", () => {
    const result = sanitizeToolError(
      "Failed: api_key=sk-leaked123456 at /Users/dev/config",
    );
    expect(result).not.toContain("sk-leaked123456");
    expect(result).not.toContain("/Users/dev/config");
  });

  it("garde le contexte du message tout en rédigeant", () => {
    const result = sanitizeToolError("HTTP 500: Bearer abcdefghijk1234 failed");
    expect(result).toContain("HTTP 500");
    expect(result).toContain("failed");
    expect(result).not.toContain("abcdefghijk1234");
  });
});
