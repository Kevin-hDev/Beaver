import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  EXTENSION_VIEW_LIMITS,
  parseExtensionRecords,
} from "./extension-records";
import { EXTENSION_INSTALL_LIMITS } from "./extension-install";

function backendRecord() {
  return {
    manifest: {
      id: "beaver.office.documents",
      name: "Documents",
      version: "1.0.0",
      beaverApi: "1",
      runtime: "node",
      main: "builtin-plugins/documents/index.mjs",
      ui: null,
      access: "full",
      apiLevel: "stable",
      essential: false,
      author: "Beaver",
      homepage: null,
      description: "Create documents.",
    },
    kind: "builtin",
    source: "Beaver",
    enabled: true,
    trusted: true,
    showInChat: true,
    status: "active",
    lastError: null,
    lastActivatedAt: null,
    contributions: {
      tools: [{
        name: "beaver.office.documents.create",
        description: "Create a document.",
        parameters: { type: "object" },
        replacesCore: false,
      }],
      events: [] as string[],
    },
  };
}

describe("parseExtensionRecords", () => {
  it("accepte la forme IPC réelle et normalise les valeurs nulles", () => {
    const [record] = parseExtensionRecords([backendRecord()]);

    expect(record.manifest.ui).toBeUndefined();
    expect(record.lastError).toBeUndefined();
    expect(record.contributions.tools[0].name)
      .toBe("beaver.office.documents.create");
  });

  it("refuse un contrat sans contributions au lieu de laisser React planter", () => {
    const input = backendRecord();
    Reflect.deleteProperty(input, "contributions");

    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");
  });

  it("refuse les collections qui dépassent les limites partagées", () => {
    const input = backendRecord();
    input.contributions.events = Array.from(
      { length: EXTENSION_VIEW_LIMITS.eventsPerExtension + 1 },
      (_, index) => `event.${index}`,
    );

    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");
  });

  it("valide la provenance Git ou npm exposée par le registre", () => {
    const input = {
      ...backendRecord(),
      kind: "local",
      source: "/managed/extension",
      origin: {
        kind: "git",
        locator: "https://github.com/example/extension.git",
        revision: "a".repeat(40),
      },
    };

    const [record] = parseExtensionRecords([input]);

    expect(record.origin?.kind).toBe("git");
    expect(record.origin?.revision).toHaveLength(40);
  });

  it("reste aligné sur la source de vérité du contrat Rust et Node", () => {
    const contract = JSON.parse(readFileSync(
      "src-tauri/resources/extension-host/contract.json",
      "utf8",
    )) as { limits: Record<string, number> };

    expect(EXTENSION_VIEW_LIMITS).toEqual({
      records: contract.limits.maxExtensions,
      toolsPerExtension: contract.limits.maxToolsPerExtension,
      eventsPerExtension: contract.limits.maxEventsPerExtension,
    });
    expect(EXTENSION_INSTALL_LIMITS).toEqual({
      git: contract.limits.maxGitLocatorChars,
      npm: contract.limits.maxNpmSpecChars,
    });
  });
});
