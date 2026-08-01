import { describe, it, expect } from "vitest";
import { sanitizeToolError, sanitizeToolErrorDetails } from "./tool-error-sanitize";

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

  it("rédige une valeur JSON contenant des espaces", () => {
    const field = ["to", "ken"].join("");
    const value = ["private", "value", "with", "spaces"].join(" ");
    const result = sanitizeToolErrorDetails(JSON.stringify({ [field]: value }));

    expect(result).not.toContain(value);
    expect(result).toContain("[redacted]");
  });

  it("rédige une clé fournisseur même sans libellé", () => {
    const credential = ["sk", "abcdefghijklmnopqrstuvwxyz123456"].join("-");
    const result = sanitizeToolError(`Provider rejected ${credential}`);

    expect(result).not.toContain(credential);
  });

  it("rédige les autorisations, cookies et identifiants intégrés aux URL", () => {
    const basic = ["dXNlcj", "pwYXNzd29yZA=="].join("");
    const session = ["private", "session", "value"].join("-");
    const userInfo = ["user", "password"].join(":");
    const details = [
      `Authorization: Basic ${basic}`,
      `Set-Cookie: session=${session}`,
      `Request: https://${userInfo}@example.test/resource`,
    ].join("\n");
    const result = sanitizeToolErrorDetails(details);

    expect(result).not.toContain(basic);
    expect(result).not.toContain(session);
    expect(result).not.toContain(userInfo);
  });

  it("rédige les JWT et identifiants AWS sans libellé", () => {
    const jwt = ["eyJhbGciOiJIUzI1NiJ9", "eyJzdWIiOiJ1c2VyIn0", "signature123"].join(".");
    const aws = ["AKIA", "IOSFODNN7EXAMPLE"].join("");
    const result = sanitizeToolErrorDetails(`Rejected ${jwt} and ${aws}`);

    expect(result).not.toContain(jwt);
    expect(result).not.toContain(aws);
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

  it("rédige aussi les chemins relatifs", () => {
    const result = sanitizeToolError("Error in ./src/main.ts");
    expect(result).not.toContain("./src/main.ts");
    expect(result).toContain("[path]");
  });

  it("rédige les chemins accolés à un séparateur", () => {
    const result = sanitizeToolError("File:/Users/dev/private.txt source:=./src/main.ts");
    expect(result).not.toContain("/Users/dev/private.txt");
    expect(result).not.toContain("./src/main.ts");
  });

  it("rédige les chemins fichier, home, UNC et relatifs sans point", () => {
    for (const path of [
      "file:///Users/dev/private.txt",
      "~/private/config.json",
      "\\\\server\\share\\secret.txt",
      "src/private/config.ts",
    ]) {
      const result = sanitizeToolError(`Cannot open ${path}`);
      expect(result).not.toContain(path);
      expect(result).toContain("[path]");
    }
  });
});

describe("sanitizeToolErrorDetails", () => {
  it("conserve plusieurs lignes utiles après rédaction", () => {
    const result = sanitizeToolErrorDetails(
      "Build failed\n/path/to/source.ts:3\ntoken=very-secret-value\nExpected 1, got 2",
    );

    expect(result).toContain("Build failed");
    expect(result).toContain("Expected 1, got 2");
    expect(result).not.toContain("/path/to/source.ts");
    expect(result).not.toContain("very-secret-value");
  });

  it("borne les anciens résultats d'erreur très volumineux", () => {
    const result = sanitizeToolErrorDetails("x".repeat(25_000));
    expect([...result].length).toBeLessThanOrEqual(20_003);
    expect(result.endsWith("...")).toBe(true);
  });

  it("retire les contrôles bidirectionnels du résumé et des détails", () => {
    expect(sanitizeToolError("safe\u202etext")).toBe("safetext");
    expect(sanitizeToolErrorDetails("safe\u2066text")).toBe("safetext");
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

  it("ignore les lignes vides ou composées d'espaces en tête", () => {
    const input = "\n  \n\t\nError: real message";
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
