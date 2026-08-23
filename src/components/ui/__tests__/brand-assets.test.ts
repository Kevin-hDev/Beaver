import { existsSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import { createCanvas, loadImage } from "@napi-rs/canvas";
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

function sha256(relativePath: string): string {
  // Les chemins sont tous des constantes internes déclarées dans ce test.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return createHash("sha256").update(readFileSync(path(relativePath))).digest("hex");
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

async function pngPixel(
  relativePath: string,
  x: number,
  y: number,
): Promise<[number, number, number, number]> {
  const image = await loadImage(path(relativePath));
  const canvas = createCanvas(image.width, image.height);
  const context = canvas.getContext("2d");
  context.drawImage(image, 0, 0);
  return [...context.getImageData(x, y, 1, 1).data] as [number, number, number, number];
}

describe("assets de marque", () => {
  it("conserve le logo approuvé avec son fond sombre et ses coins arrondis", async () => {
    expect(fileExists("public/castor-surface.svg")).toBe(true);
    expect(fileExists("public/castor-encre.svg")).toBe(true);
    expect(pngInfo("src/assets/logo.png")).toEqual({
      width: 1024,
      height: 1024,
      hasAlpha: true,
    });
    expect((await pngPixel("src/assets/logo.png", 0, 0))[3]).toBe(0);
    expect((await pngPixel("src/assets/logo.png", 512, 0))[3]).toBe(255);
    // Le logo est le produit visuel lui-même : son empreinte verrouille l'image approuvée.
    expect(sha256("src/assets/logo.png")).toBe(
      "7282d130d743f011bfabd76d0a5d1be71c9e5e9e36ff2229ff5e695e46b0b763",
    );

    for (const obsolete of [
      "src/assets/logo-dark.png",
      "src/assets/logo-light.png",
      "src/assets/icone-app.png",
      "public/castor.svg",
      "public/splash-icon.png",
      "public/splash-icon-light.png",
    ]) {
      expect(fileExists(obsolete)).toBe(false);
    }
  });

  /* Les deux pochoirs se superposent, et rien dans le rendu ne signale un décalage :
     un cadre différent sur l'un des deux réduirait l'encre et la découpe de facteurs
     différents, et l'erreur ne se verrait qu'à l'œil, sur un écran, dans une palette. */
  it("aligne les deux pochoirs sur un cadre commun", () => {
    const viewBox = /viewBox="([^"]+)"/;
    const surface = readText("public/castor-surface.svg").match(viewBox)?.[1];
    const ink = readText("public/castor-encre.svg").match(viewBox)?.[1];

    expect(surface).toBeDefined();
    expect(ink).toBe(surface);
  });

  it("colore le castor selon le thème aux tailles prévues", () => {
    const splash = readText("index.html");
    const onboarding = readText("src/components/onboarding/onboarding.css");

    expect(splash).toContain("width: 170px");
    expect(splash).toContain("height: 170px");
    expect(splash).toContain('mask: url("/castor-surface.svg")');
    expect(splash).toContain('mask: url("/castor-encre.svg")');
    expect(onboarding).toContain("width: 4.5rem");
    expect(onboarding).toContain('mask: url("/castor-surface.svg")');
    expect(onboarding).toContain('mask: url("/castor-encre.svg")');
    expect(onboarding).toContain("background: var(--brand-surface)");
    expect(onboarding).toContain("background: var(--brand-mark)");
  });

  /* L'onboarding lit les jetons de thème, le splash les recopie en dur faute d'être
     peint après leur chargement. Les deux déclarations doivent donc échanger les
     rôles au même endroit : sur fond sombre la découpe prend --ink, sur fond clair
     elle prend --void et disparaît dans le fond. */
  it("échange découpe et encre entre les palettes claires et sombres", () => {
    const onboarding = readText("src/components/onboarding/onboarding.css");

    expect(onboarding).toContain("--brand-surface: var(--void)");
    expect(onboarding).toContain("--brand-mark: var(--ink)");
    expect(onboarding).toMatch(
      /\[data-theme="dark"\] \.ob-brand-castor \{\s*--brand-surface: var\(--ink\);\s*--brand-mark: var\(--void\);/,
    );
  });

  it("fournit toutes les icônes desktop requises", async () => {
    expect(pngSize("src-tauri/icons/32x32.png")).toEqual({ width: 32, height: 32 });
    expect(pngSize("src-tauri/icons/128x128.png")).toEqual({ width: 128, height: 128 });
    expect(pngSize("src-tauri/icons/128x128@2x.png")).toEqual({ width: 256, height: 256 });
    expect(pngSize("src-tauri/icons/tray.png")).toEqual({ width: 64, height: 64 });
    expect(pngInfo("src-tauri/icons/tray.png").hasAlpha).toBe(true);
    expect((await pngPixel("src-tauri/icons/tray.png", 0, 0))[3]).toBe(0);
    expect(fileExists("src-tauri/icons/icon.icns")).toBe(true);
    expect(fileExists("src-tauri/icons/icon.ico")).toBe(true);
    // icon.ico is the Windows packaging authority, not a duplicated pixel hash.
    expect(sha256("src-tauri/icons/icon.ico")).toBe(
      "76efa8ed52632b06614fa7c95d8d5dd0ff20f9f00b13104b7c2aadb69a3581e5",
    );
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

  it("embarque les planches des quatre nouvelles mascottes", () => {
    for (const mascotId of ["mokai", "volt", "raku", "pico"]) {
      const directory = `src/assets/mascot/${mascotId}`;
      const manifest = JSON.parse(
        readText(`${directory}/manifest.json`),
      ) as { id: string; sheets: { standard: unknown; actions: unknown } };

      expect(fileExists(`${directory}/standard.webp`)).toBe(true);
      expect(fileExists(`${directory}/actions.webp`)).toBe(true);
      expect(manifest.id).toBe(mascotId);
      expect(manifest.sheets.standard).toBeDefined();
      expect(manifest.sheets.actions).toBeDefined();
    }
  });
});
