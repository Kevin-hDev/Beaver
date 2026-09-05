import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import {
  EXTENSION_VIEW_LIMITS,
  parseExtensionRecords,
} from "./extension-records";
import { LIMITS } from "@/types/extension-contract.generated";
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
      skills: [] as Array<{ id: string; name: string; description: string; path: string }>,
      resources: [] as Array<{
        id: string;
        name: string;
        description: string;
        type: string;
        path: string;
      }>,
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

  it("accepte les manifestes UI v2 standard et avancé", () => {
    const standard = backendRecord();
    standard.manifest.ui = { apiVersion: "1", mode: "standard" } as never;
    const advanced = backendRecord();
    advanced.manifest.ui = {
      apiVersion: "1",
      mode: "advanced",
      entry: "ui/index.mjs",
    } as never;

    expect(parseExtensionRecords([standard])[0].manifest.ui).toEqual({
      apiVersion: "1",
      mode: "standard",
    });
    expect(parseExtensionRecords([advanced])[0].manifest.ui?.entry)
      .toBe("ui/index.mjs");
  });

  it("refuse les formes UI v2 ambiguës ou contenant des champs inconnus", () => {
    for (const ui of [
      { apiVersion: "1", mode: "standard", entry: "ui.mjs" },
      { apiVersion: "1", mode: "advanced" },
      { apiVersion: "2", mode: "standard" },
      { apiVersion: "1", mode: "standard", extra: true },
    ]) {
      const input = backendRecord();
      input.manifest.ui = ui as never;
      expect(() => parseExtensionRecords([input]))
        .toThrow("invalid_extension_response");
    }
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

  it("borne séparément les skills et ressources projetés", () => {
    const input = backendRecord();
    input.contributions.skills = Array.from(
      { length: EXTENSION_VIEW_LIMITS.skillsPerExtension + 1 },
      (_, index) => ({
        id: `skill-${index}`,
        name: "Skill",
        description: "Description",
        path: `skills/${index}.md`,
      }),
    );
    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");

    input.contributions.skills = [];
    input.contributions.resources = Array.from(
      { length: EXTENSION_VIEW_LIMITS.resourcesPerExtension + 1 },
      (_, index) => ({
        id: `resource-${index}`,
        name: "Resource",
        description: "Description",
        type: "text",
        path: `resources/${index}.txt`,
      }),
    );
    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");
  });

  it("accepte exactement 32 skills et 64 ressources projetés", () => {
    expect(EXTENSION_VIEW_LIMITS.skillsPerExtension).toBe(32);
    expect(EXTENSION_VIEW_LIMITS.resourcesPerExtension).toBe(64);
    const input = backendRecord();
    input.contributions.skills = Array.from(
      { length: EXTENSION_VIEW_LIMITS.skillsPerExtension },
      (_, index) => ({
        id: `skill-${index}`,
        name: "Skill",
        description: "Description",
        path: `skills/${index}.md`,
      }),
    );
    input.contributions.resources = Array.from(
      { length: EXTENSION_VIEW_LIMITS.resourcesPerExtension },
      (_, index) => ({
        id: `resource-${index}`,
        name: "Resource",
        description: "Description",
        type: "text",
        path: `resources/${index}.txt`,
      }),
    );

    const [record] = parseExtensionRecords([input]);

    expect(record.contributions.skills).toHaveLength(32);
    expect(record.contributions.resources).toHaveLength(64);
  });

  it("ignore les anciens événements inconnus sans perdre le registre", () => {
    const input = backendRecord();
    input.contributions.events = ["session.legacy", "session.turn.started"];

    const [record] = parseExtensionRecords([input]);

    expect(record.contributions.events).toEqual(["session.turn.started"]);
  });

  it("accepte les contributions R0 et mesure les chemins en valeurs Unicode", () => {
    const input = backendRecord();
    input.contributions.skills = [{
      id: "reference-skill",
      name: "reference-skill",
      description: "Compétence 🦫",
      path: "SKILL.md",
    }];
    input.contributions.resources = [{
      id: "preview",
      name: "preview",
      description: "Aperçu 🦫",
      type: "image",
      path: "🦫".repeat(4096),
    }];

    const [record] = parseExtensionRecords([input]);

    expect(record.contributions.skills?.[0].name).toBe("reference-skill");
    expect(record.contributions.resources?.[0].type).toBe("image");
  });

  it("accepte les noms humains R0 mais refuse leurs identifiants non ASCII", () => {
    const input = backendRecord();
    input.contributions.skills = [{
      id: "reference-skill",
      name: "Compétence 🦫",
      description: "Résumé",
      path: "SKILL.md",
    }];
    expect(parseExtensionRecords([input])[0].contributions.skills?.[0].name)
      .toBe("Compétence 🦫");

    input.contributions.skills[0].id = "compétence";
    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");
  });

  it("refuse les champs inconnus des contributions skills et ressources", () => {
    const input = backendRecord();
    const rawContributions = input.contributions as Record<string, unknown>;
    rawContributions.skills = [{
      id: "guide",
      name: "Guide",
      description: "Description",
      path: "SKILL.md",
      root: "/untrusted",
    }];
    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");

    rawContributions.skills = [];
    rawContributions.resources = [{
      id: "resource",
      name: "Resource",
      description: "Description",
      type: "text",
      path: "resources/reference.txt",
      mimeType: "text/plain",
    }];
    expect(() => parseExtensionRecords([input]))
      .toThrow("invalid_extension_response");
  });

  it("préserve la compatibilité des contributions historiques sans skills ni resources", () => {
    const input = backendRecord();
    Reflect.deleteProperty(input.contributions, "skills");
    Reflect.deleteProperty(input.contributions, "resources");
    const [record] = parseExtensionRecords([input]);

    expect(record.contributions.skills).toEqual([]);
    expect(record.contributions.resources).toEqual([]);
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
      skillsPerExtension: contract.limits.maxSkillsPerExtension,
      resourcesPerExtension: contract.limits.maxResourcesPerExtension,
    });
    expect(EXTENSION_INSTALL_LIMITS).toEqual({
      git: contract.limits.maxGitLocatorChars,
      npm: contract.limits.maxNpmSpecChars,
    });
    expect(LIMITS.maxExtensionNameChars).toBe(contract.limits.maxExtensionNameChars);
    expect(LIMITS.maxExtensionTextChars).toBe(contract.limits.maxExtensionTextChars);
  });
});
