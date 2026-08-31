import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* L'écran du terminal est peint par xterm, hors de portée des feuilles de
   style : rien de ce qu'il affiche ne peut être vérifié en regardant le CSS.
   Ces tests tiennent donc le seul passage entre les deux mondes — le fichier
   qui lit les jetons de l'application et les donne à xterm.

   Ils viennent d'un défaut réel : le terminal réclamait la police
   « JetBrains Mono », nom que la police installée ne porte pas. La famille
   n'était jamais trouvée, et le terminal était le seul écran de l'application
   à écrire dans la police par défaut du système. */

const ROOT = join(__dirname, "..", "..", "..", "..");

function read(relative: string): string {
  /* eslint-disable-next-line security/detect-non-literal-fs-filename -- les
     chemins sont écrits dans ce fichier, aucun ne vient de l'extérieur. */
  return readFileSync(join(ROOT, relative), "utf8");
}

describe("apparence du terminal", () => {
  /* Le jeton suivait la police au moment du défaut ; c'est le terminal qui
     réclamait un autre nom. Ce test garde l'autre bout du fil : le jour où la
     police installée change de nom, le jeton doit changer avec elle. */
  it("le jeton de police nomme la famille réellement installée", () => {
    const installed = /font-family:\s*'([^']+)'/.exec(
      read("node_modules/@fontsource-variable/jetbrains-mono/index.css"),
    );
    const tokens = read("src/styles/tokens.css");
    const declared = /--font-mono:\s*([^;]+);/.exec(tokens);

    expect(installed?.[1]).toBeTruthy();
    expect(declared?.[1]).toContain(installed![1]);
  });

  it("le terminal ne nomme aucune police lui-même", () => {
    const instance = read("src/components/terminal/terminal-instance.tsx");

    expect(instance).toContain("fontFamily: readTerminalFont()");
    expect(instance).not.toMatch(/JetBrains|Fira Code|monospace/);
  });

  it("le terminal ne nomme aucune couleur lui-même", () => {
    const instance = read("src/components/terminal/terminal-instance.tsx");

    expect(instance).toContain("theme: readTerminalTheme()");
    expect(instance).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgba?\(/);
  });

  it("le collage naturel de xterm suit l'unique file d'entrée bornée", () => {
    const instance = read("src/components/terminal/terminal-instance.tsx");
    const bridge = read("src/components/terminal/terminal-pty-bridge.ts");

    expect(instance).not.toContain('addEventListener("paste"');
    expect(instance).not.toContain("clipboardData");
    expect(instance).not.toContain('invoke("pty_write"');
    expect(instance).toContain("createTerminalPtyBridge");
    expect(bridge).toContain("terminal.onData");
    expect(bridge).toContain("new TerminalInputQueue");
  });

  /* Sans les seize, xterm garde sa palette d'usine — inchangée dans les six
     thèmes, et dont le blanc vif est illisible sur les fonds clairs. */
  it("les seize couleurs de sortie sont dérivées des jetons de l'application", () => {
    const palette = read("src/components/terminal/terminal-palette.css");
    const names = [
      "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
    ];

    for (const name of names) {
      expect(palette).toContain(`--term-${name}:`);
      expect(palette).toContain(`--term-bright-${name}:`);
    }
    /* Aucune teinte écrite ici : chacune renvoie à un jeton des six thèmes. */
    expect(palette).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  });

  /* getPropertyValue rend le texte de la feuille de style, pas la couleur : un
     color-mix en sortirait tel quel et xterm ne saurait pas le lire. */
  it("les couleurs sont résolues par une sonde, pas lues telles quelles", () => {
    const theme = read("src/components/terminal/terminal-theme.ts");

    expect(theme).toContain("getComputedStyle(probe).color");
  });
});
