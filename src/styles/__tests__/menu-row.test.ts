import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/* Chemins littéraux : la règle de sécurité du projet refuse une lecture de
   fichier dont le chemin est assemblé à l'exécution. */
const primitive = readFileSync("src/styles/menu-row.css", "utf8");
const global = readFileSync("src/styles/global.css", "utf8");

const modelSelector = readFileSync("src/components/agent-local/model-selector.css", "utf8");
const settingsSelect = readFileSync("src/components/settings/settings-select.css", "utf8");
const customSelect = readFileSync("src/components/ui/custom-select.css", "utf8");
const contextMenu = readFileSync("src/components/ui/context-menu.css", "utf8");
const modeSelector = readFileSync("src/components/agent-local/mode-selector.css", "utf8");
const reasoningSelector = readFileSync("src/components/agent-local/reasoning-selector.css", "utf8");
const permissionMode = readFileSync("src/components/agent-local/permission-mode-selector.css", "utf8");
const chatPlusMenu = readFileSync("src/components/agent-local/chat-plus-menu.css", "utf8");
const slashAutocomplete = readFileSync("src/components/agent-local/slash-autocomplete.css", "utf8");
const fileTabMenu = readFileSync("src/components/file-preview/file-tab-menu.css", "utf8");
const exportDropdown = readFileSync("src/components/forecast/widgets/export-dropdown.css", "utf8");
const projectSelector = readFileSync("src/components/agent-local/project-selector.css", "utf8");
const branchSelector = readFileSync("src/components/agent-local/branch-selector.css", "utf8");
const filePreviewTabs = readFileSync("src/components/file-preview/file-preview-tabs.css", "utf8");
const scenarioMenu = readFileSync("src/components/forecast/sections/forecast-scenario-menu.css", "utf8");

/* Les dix-neuf lignes de menu de l'application, telles qu'elles étaient avant le
   4 septembre 2026 : sept retraits horizontaux et quatre couleurs de survol pour
   le même geste. Chacune passe désormais par la primitive ; ce fichier tombe en
   rouge si l'une d'elles se remet à dessiner sa propre ligne. */
const MENU_ROWS: [selector: string, css: string][] = [
  [".ms-item", modelSelector],
  [".ms-provider", modelSelector],
  [".ms-section-fav", modelSelector],
  [".ss-option", settingsSelect],
  [".ss-group-header", settingsSelect],
  [".cs-option", customSelect],
  [".context-item", contextMenu],
  [".asp-mode-item", modeSelector],
  [".rs-option", reasoningSelector],
  [".perm-mode-option", permissionMode],
  [".cpm-item", chatPlusMenu],
  [".slash-item", slashAutocomplete],
  [".fp-menu-item", fileTabMenu],
  [".exd-item", exportDropdown],
  [".cpm-sub-item", chatPlusMenu],
  [".project-dropdown-item", projectSelector],
  [".bs-item", branchSelector],
  [".fps-menu-item", filePreviewTabs],
  [".fcs-menu-option", scenarioMenu],
];

/* Ce que la primitive seule a le droit de dire. Le retrait vertical n'y figure
   pas : les lignes qui portent une description le règlent par
   --menu-row-padding-y, sans jamais réécrire le retrait latéral. */
const OWNED_BY_PRIMITIVE = ["min-height:", "border-radius:", "padding: 0 "];

/** Corps de la règle qui porte exactement ce sélecteur, sans ses variantes. */
function ruleBody(css: string, selector: string): string {
  const start = css.indexOf(`\n${selector} {`);
  if (start === -1) return "";
  const open = css.indexOf("{", start);
  const close = css.indexOf("}", open);
  return close === -1 ? "" : css.slice(open + 1, close);
}

/** Corps de toutes les règles où ce sélecteur apparaît au survol. */
function hoverBodies(css: string, selector: string): string[] {
  const bodies: string[] = [];
  let from = 0;
  for (;;) {
    const hit = css.indexOf(`${selector}:hover`, from);
    if (hit === -1) return bodies;
    from = hit + 1;
    const open = css.indexOf("{", hit);
    const close = css.indexOf("}", open);
    if (open === -1 || close === -1) return bodies;
    bodies.push(css.slice(open + 1, close));
  }
}

describe("menu-row, primitive de ligne de couche flottante", () => {
  it("est chargée globalement, comme les autres primitives de style", () => {
    expect(global).toContain('@import "./menu-row.css";');
  });

  it("porte le retrait latéral en un seul endroit", () => {
    expect(primitive).toContain("--menu-row-padding-x: var(--chrome-4)");
    expect(ruleBody(primitive, ".menu-row")).toContain(
      "padding: var(--menu-row-padding-y) var(--menu-row-padding-x)",
    );
  });

  it("donne un seul survol aux lignes et un seul, plus discret, aux en-têtes", () => {
    expect(primitive).toContain("background: var(--select-bg)");
    expect(primitive).toContain("background: var(--surface-hover)");
  });

  it.each(MENU_ROWS)("%s ne redessine pas la ligne que la primitive porte", (selector, css) => {
    const body = ruleBody(css, selector);
    expect(body, `${selector} n'a plus de règle`).not.toBe("");
    for (const property of OWNED_BY_PRIMITIVE) {
      expect(body, `${selector} redéclare ${property}`).not.toContain(property);
    }
  });

  it.each(MENU_ROWS)("%s laisse le survol à la primitive", (selector, css) => {
    for (const body of hoverBodies(css, selector)) {
      expect(body, `${selector}:hover repeint son fond`).not.toContain("background:");
    }
  });
});
