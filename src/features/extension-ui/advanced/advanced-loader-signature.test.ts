import { describe, expect, it } from "vitest";
import type { ExtensionRecord } from "@/types/extensions";
import { advancedRecordsSignature } from "./advanced-loader-signature";

describe("advancedRecordsSignature", () => {
  it("ignores record churn that cannot change an advanced module", () => {
    const first = record();
    const refreshed = { ...first, status: "loading" as const, lastActivatedAt: "later" };

    expect(advancedRecordsSignature([refreshed])).toBe(advancedRecordsSignature([first]));
  });

  it("changes when the content-addressed advanced artifact changes", () => {
    const first = record();
    const changed = {
      ...first,
      uiArtifact: { ...first.uiArtifact!, manifestSha256: "b".repeat(64) },
    };

    expect(advancedRecordsSignature([changed])).not.toBe(advancedRecordsSignature([first]));
  });
});

function record(): ExtensionRecord {
  return {
    manifest: {
      id: "com.example.advanced",
      name: "Advanced",
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      access: "full",
      apiLevel: "advanced",
      essential: false,
      ui: { apiVersion: "1", mode: "advanced", entry: "entry.ts" },
    },
    kind: "local",
    source: "fixture",
    enabled: true,
    trusted: true,
    showInChat: false,
    status: "active",
    contributions: { tools: [], events: [] },
    uiArtifact: {
      version: 1,
      builderVersion: "fixture",
      nodeVersion: "20.0.0",
      entry: "entry.mjs",
      totalBytes: 1,
      outputs: [],
      inputs: [],
      manifestSha256: "a".repeat(64),
    },
  };
}
