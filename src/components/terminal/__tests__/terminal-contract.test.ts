import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

const TERMINAL_RUST = "src-tauri/src/services/terminal";
const LIMITS_PATH = `${TERMINAL_RUST}/limits.rs`;
const MANAGER_PATH = `${TERMINAL_RUST}/manager.rs`;
const COMMAND_PATH = "src-tauri/src/commands/terminal.rs";
const TYPES_PATH = "src/hooks/terminal-types.ts";
const INPUT_QUEUE_PATH = "src/components/terminal/terminal-input-queue.ts";

function source(path: string): string {
  // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins du dépôt issus des constantes de ce test
  return readFileSync(path, "utf8");
}

function filesBelow(root: string, extensions: string[]): string[] {
  // eslint-disable-next-line security/detect-non-literal-fs-filename -- racines fixes du dépôt, aucune entrée externe
  return readdirSync(root, { withFileTypes: true }).flatMap((entry) => {
    const path = join(root, entry.name);
    return entry.isDirectory()
      ? filesBelow(path, extensions)
      : extensions.some((extension) => path.endsWith(extension)) ? [path] : [];
  });
}

describe("terminal limits contract", () => {
  it("pince les plafonds de sessions et d'entrée frontend", () => {
    expect(source(MANAGER_PATH)).toMatch(/pub const MAX_PTY_SESSIONS: usize = 16;/u);
    expect(source(TYPES_PATH)).toMatch(/export const MAX_LIVE_TERMINALS = 16;/u);
    expect(source(INPUT_QUEUE_PATH)).toMatch(/export const MAX_WRITE_BYTES = 65_536;/u);
    expect(source(INPUT_QUEUE_PATH)).toMatch(/export const MAX_PENDING_INPUT_BYTES = 256 \* 1024;/u);
  });

  it("garde les quatre limites de flux dans la seule autorité Rust", () => {
    const limits = source(LIMITS_PATH);
    const expected = new Map([
      ["MAX_PTY_WRITE_BYTES", "64 \\* 1024"],
      ["MAX_FRAME_BYTES", "64 \\* 1024"],
      ["MAX_IN_FLIGHT_BYTES", "1024 \\* 1024"],
      ["MAX_IN_FLIGHT_FRAMES", "256"],
    ]);
    const rustSources = filesBelow(TERMINAL_RUST, [".rs"]);

    for (const [name, value] of expected) {
      // eslint-disable-next-line security/detect-non-literal-regexp -- noms et valeurs proviennent de la Map statique ci-dessus
      expect(limits).toMatch(new RegExp(`const ${name}: usize = ${value};`, "u"));
      const definitions = rustSources.filter((path) =>
        // eslint-disable-next-line security/detect-non-literal-regexp -- nom issu de la Map statique ci-dessus
        new RegExp(`const\\s+${name}\\s*:`).test(source(path))
      );
      expect(definitions).toEqual([LIMITS_PATH]);
    }
  });

  it("pince les cinq limites de persistance dans une autorité par langage", () => {
    const limits = source(LIMITS_PATH);
    const types = source(TYPES_PATH);
    const expected = new Map([
      ["MAX_GROUPS", "128"],
      ["MAX_TABS_PER_GROUP", "16"],
      ["MAX_TOTAL_TABS", "256"],
      ["MAX_GROUP_KEY_BYTES", "128"],
      ["MAX_LABEL_BYTES", "512"],
    ]);
    const rustSources = filesBelow(TERMINAL_RUST, [".rs"]);
    const frontendSources = filesBelow("src/hooks", [".ts", ".tsx"])
      .filter((path) => !path.includes("/__tests__/"));

    for (const [name, value] of expected) {
      // eslint-disable-next-line security/detect-non-literal-regexp -- contrat statique défini ci-dessus
      expect(limits).toMatch(new RegExp(`pub\\(super\\) const ${name}: usize = ${value};`, "u"));
      // eslint-disable-next-line security/detect-non-literal-regexp -- contrat statique défini ci-dessus
      expect(types).toMatch(new RegExp(`export const ${name} = ${value};`, "u"));
      const rustDefinitions = rustSources.filter((path) =>
        // eslint-disable-next-line security/detect-non-literal-regexp -- nom issu de la Map statique ci-dessus
        new RegExp(`const\\s+${name}\\s*:`).test(source(path))
      );
      const frontendDefinitions = frontendSources.filter((path) =>
        // eslint-disable-next-line security/detect-non-literal-regexp -- nom issu de la Map statique ci-dessus
        new RegExp(`const\\s+${name}\\s*=`).test(source(path))
      );
      expect(rustDefinitions).toEqual([LIMITS_PATH]);
      expect(frontendDefinitions).toEqual([TYPES_PATH]);
    }
  });

  it("refuse les valeurs brutes répétées dans les composants de production", () => {
    const componentSources = filesBelow("src/components/terminal", [".ts", ".tsx"])
      .filter((path) => !path.includes("/__tests__/") && path !== INPUT_QUEUE_PATH)
      .map((path) => source(path));
    const repeatedLimits = /(?:65_536|64\s*\*\s*1024|256\s*\*\s*1024|1024\s*\*\s*1024)/u;

    for (const component of componentSources) expect(component).not.toMatch(repeatedLimits);
  });

  it("restreint le spawn direct et garde le worker comme entrée Linux", () => {
    const manager = source(MANAGER_PATH);
    const command = source(COMMAND_PATH);

    expect(manager).toContain("pub(crate) fn spawn(");
    expect(command).toContain(".spawn_linux(");
    expect(command).toContain("#[cfg(target_os = \"linux\")]");
  });
});
