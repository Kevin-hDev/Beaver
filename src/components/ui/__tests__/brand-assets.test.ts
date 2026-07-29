import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const root = process.cwd();

function path(relativePath: string): string {
  return resolve(root, relativePath);
}

function fileExists(relativePath: string): boolean {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return existsSync(path(relativePath));
}

function readText(relativePath: string): string {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readFileSync(path(relativePath), "utf8");
}

function pngInfo(relativePath: string): { width: number; height: number; hasAlpha: boolean } {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  const bytes = readFileSync(path(relativePath));
  expect(bytes.subarray(1, 4).toString("ascii")).toBe("PNG");
  return {
    width: bytes.readUInt32BE(16),
    height: bytes.readUInt32BE(20),
    hasAlpha: [4, 6].includes(bytes.readUInt8(25)),
  };
}

function pngSize(relativePath: string): { width: number; height: number } {
  const { width, height } = pngInfo(relativePath);
  return { width, height };
}

describe("assets de marque", () => {
  it("conserve uniquement les deux sources visuelles actives", () => {
    expect(fileExists("public/castor.svg")).toBe(true);
    expect(pngInfo("src/assets/logo.png")).toEqual({
      width: 1024,
      height: 1024,
      hasAlpha: true,
    });

    for (const obsolete of [
      "src/assets/logo-dark.png",
      "src/assets/logo-light.png",
      "src/assets/icone-app.png",
      "public/splash-icon.png",
      "public/splash-icon-light.png",
    ]) {
      expect(fileExists(obsolete)).toBe(false);
    }
  });

  it("colore le castor selon le thème aux tailles prévues", () => {
    const splash = readText("index.html");
    const onboarding = readText("src/components/onboarding/onboarding.css");

    expect(splash).toContain("width: 170px");
    expect(splash).toContain("height: 170px");
    expect(splash).toContain("--splash-mark: #c8c8ce");
    expect(splash).toContain("--splash-mark: #1a1a1a");
    expect(splash).toContain('mask: url("/castor.svg")');
    expect(onboarding).toContain("width: 4.5rem");
    expect(onboarding).toContain("background: var(--ink)");
  });

  it("fournit toutes les icônes desktop requises", () => {
    expect(pngSize("src-tauri/icons/32x32.png")).toEqual({ width: 32, height: 32 });
    expect(pngSize("src-tauri/icons/128x128.png")).toEqual({ width: 128, height: 128 });
    expect(pngSize("src-tauri/icons/128x128@2x.png")).toEqual({ width: 256, height: 256 });
    expect(pngSize("src-tauri/icons/tray.png")).toEqual({ width: 64, height: 64 });
    expect(pngInfo("src-tauri/icons/tray.png").hasAlpha).toBe(true);
    expect(fileExists("src-tauri/icons/icon.icns")).toBe(true);
    expect(fileExists("src-tauri/icons/icon.ico")).toBe(true);
  });

  it("utilise Beaver sur les surfaces desktop sans changer l'identifiant système", () => {
    const mainHtml = readText("index.html");
    const mascotHtml = readText("mascot.html");
    const tauriConfig = readText("src-tauri/tauri.conf.json");

    expect(mainHtml).toContain("<title>Beaver</title>");
    expect(mascotHtml).toContain("<title>Beaver Mascotte</title>");
    expect(tauriConfig).toContain('"productName": "Beaver"');
    expect(tauriConfig).toContain('"title": "Beaver"');
    expect(tauriConfig).toContain('"identifier": "com.clgo.dash"');
    expect(tauriConfig).not.toContain('"mainBinaryName"');
  });

  it("renomme seulement l'affichage de la mascotte", () => {
    const manifest = readText("src/assets/mascot/cl-go-beaver/manifest.json");

    expect(manifest).toContain('"id": "cl-go-beaver"');
    expect(manifest).toContain('"displayName": "Beaver"');
    expect(manifest).not.toContain("Castor CL-GO");
  });

  it("embarque les deux planches transparentes de Circuit", () => {
    const manifest = readText("src/assets/mascot/circuit/manifest.json");

    expect(fileExists("src/assets/mascot/circuit/standard.webp")).toBe(true);
    expect(fileExists("src/assets/mascot/circuit/actions.webp")).toBe(true);
    expect(manifest).toContain('"id": "circuit"');
    expect(manifest).toContain('"columns": 6');
    expect(manifest).toContain('"id": "work-laptop"');
    expect(manifest).toContain('"id": "sleeping"');
  });

  it("embarque les planches standard et avancée de Kova", () => {
    const manifest = readText("src/assets/mascot/kova/manifest.json");

    expect(fileExists("src/assets/mascot/kova/standard.webp")).toBe(true);
    expect(fileExists("src/assets/mascot/kova/actions.webp")).toBe(true);
    expect(manifest).toContain('"id": "kova"');
    expect(manifest).toContain('"startFrame": 5');
    expect(manifest).toContain('"id": "success"');
    expect(manifest).toContain('"id": "grabbed"');
  });

  it("embarque les planches standard et avancée de Nival", () => {
    const manifest = JSON.parse(
      readText("src/assets/mascot/nival/manifest.json"),
    ) as { id: string; states: Array<{ id: string; startFrame?: number }> };

    expect(fileExists("src/assets/mascot/nival/standard.webp")).toBe(true);
    expect(fileExists("src/assets/mascot/nival/actions.webp")).toBe(true);
    expect(manifest.id).toBe("nival");
    expect(manifest.states.some((state) => state.startFrame === 5)).toBe(true);
    expect(manifest.states.some((state) => state.id === "success")).toBe(true);
    expect(manifest.states.some((state) => state.id === "grabbed")).toBe(true);
  });
});
