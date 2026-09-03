import { describe, expect, it } from "vitest";
import { parseExtensionUiArtifact } from "./extension-ui-artifact";

const SHA = "a".repeat(64);

describe("parseExtensionUiArtifact", () => {
  it("requires the declared entry to be the single JavaScript output", () => {
    const artifact = {
      version: 1,
      builderVersion: "0.28.1",
      nodeVersion: "v20.0.0",
      entry: "style.css",
      totalBytes: 2,
      outputs: [
        { name: "entry.js", type: "javascript", bytes: 1, sha256: SHA },
        { name: "style.css", type: "css", bytes: 1, sha256: SHA },
      ],
      inputs: ["entry.ts"],
      manifestSha256: SHA,
    };

    expect(() => parseExtensionUiArtifact(artifact))
      .toThrow("invalid_extension_response");
    expect(parseExtensionUiArtifact({
      ...artifact,
      entry: "entry.js",
    })?.entry).toBe("entry.js");
  });
});
