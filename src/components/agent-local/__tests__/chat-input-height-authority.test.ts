import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/* La hauteur du champ de saisie n'a qu'une autorité : le plafond posé en CSS
   sur `.cm-editor`.

   Elle en a eu deux. Le CSS plafonnait l'éditeur à 200 px pendant qu'un calcul
   en JavaScript fixait la hauteur du conteneur, marges comprises — deux nombres
   qui ne mesuraient pas la même chose, à 16 px près. Pour mesurer, ce calcul
   remettait la hauteur à « auto » : le temps de cet aller-retour la zone ne
   débordait plus, le navigateur ramenait le défilement en butée, et il y
   restait. À partir de la neuvième ligne, on ne voyait plus que le haut des
   lettres de la ligne en cours d'écriture. */

const ROOT = join(import.meta.dirname, "../../..");

function read(relative: string): string {
  // Chemin fixe écrit ici même, pas une entrée.
  // eslint-disable-next-line security/detect-non-literal-fs-filename
  return readFileSync(join(ROOT, relative), "utf8");
}

const HOOK = read("hooks/use-codemirror-chat.ts");
const BEHAVIOR = read("hooks/chat-editor-behavior.ts");
const EDITOR_CSS = read("components/agent-local/chat-input-textarea.css");

describe("hauteur du champ de saisie", () => {
  it("n'est jamais écrite depuis le JavaScript", () => {
    expect(HOOK).not.toMatch(/style\.height/);
    expect(HOOK).not.toMatch(/scrollHeight/);
    expect(HOOK).not.toMatch(/ResizeObserver/);
    expect(BEHAVIOR).not.toMatch(/height/);
  });

  it("est plafonnée une seule fois, par le token", () => {
    const caps = EDITOR_CSS.match(/max-height:[^;]+;/g) ?? [];

    expect(caps).toEqual(["max-height: var(--chat-input-max-height);"]);
  });

  /* Posées sur le conteneur, elles se retranchent du plafond : la zone qui
     défile devient plus courte que lui, et la ligne du bas passe dessous.
     Posées sur le texte, elles défilent avec lui. */
  it("garde ses marges hautes et basses à l'intérieur de la zone qui défile", () => {
    const host = EDITOR_CSS.match(/\.chat-cm-host \{[^}]*\}/)?.[0] ?? "";
    const content = EDITOR_CSS.match(/\.chat-cm-host \.cm-content \{[^}]*\}/)?.[0] ?? "";

    expect(host).toMatch(/padding: 0 /);
    expect(content).toMatch(/padding: var\(--space-sm\) 0;/);
  });
});
