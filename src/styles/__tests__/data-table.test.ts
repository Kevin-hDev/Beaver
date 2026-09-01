import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const primitive = readFileSync("src/styles/data-table.css", "utf8");
const global = readFileSync("src/styles/global.css", "utf8");
const chatMarkdown = readFileSync("src/components/agent-local/chat-markdown.css", "utf8");
const workbenchData = readFileSync("src/components/forecast/workbench/forecast-workbench-data.css", "utf8");
const evaluation = readFileSync("src/components/forecast/evaluation/forecast-evaluation.css", "utf8");

/* Chemins littéraux : la règle de sécurité du projet refuse une lecture de
   fichier dont le chemin est assemblé à l'exécution. */
const THEMES = {
  "Sombre": readFileSync("src/styles/themes/dark.css", "utf8"),
  "Clair": readFileSync("src/styles/themes/light.css", "utf8"),
  "Émeraude": readFileSync("src/styles/themes/emerald-night.css", "utf8"),
  "Cobalt givré": readFileSync("src/styles/themes/cobalt-frost.css", "utf8"),
  "Brume astrale": readFileSync("src/styles/themes/astral-mist.css", "utf8"),
  "Éclipse écarlate": readFileSync("src/styles/themes/crimson-eclipse.css", "utf8"),
};

describe("Tableau de données — autorité unique", () => {
  it("est chargé par la feuille globale", () => {
    expect(global).toContain('@import "./data-table.css";');
  });

  it("habille le markdown des réponses depuis la primitive", () => {
    // La référence visuelle de Kevin : le tableau que l'agent produit.
    expect(primitive).toContain(".chat-md table");
    expect(primitive).toContain(".chat-md td");
  });

  it("ne laisse aucun fond de cellule limité à un thème", () => {
    // Ces règles n'existaient que pour Sombre et Clair : sur les quatre autres
    // thèmes les cellules n'avaient pas de fond du tout.
    expect(chatMarkdown).not.toMatch(/\[data-theme="[^"]+"\]\s*\.chat-md\s+(th|td)/);
    expect(primitive).not.toContain("[data-theme=\"dark\"]");
    expect(primitive).not.toContain("[data-theme=\"light\"]");
  });

  it("s'appuie sur des jetons que les six thèmes définissent", () => {
    for (const [nom, theme] of Object.entries(THEMES)) {
      expect(theme, `${nom} : --surface-overlay`).toContain("--surface-overlay:");
      expect(theme, `${nom} : --surface-glass`).toContain("--surface-glass:");
    }
  });

  it("laisse les tableaux forecast sans quadrillage ni fond propre", () => {
    for (const [name, sheet] of [["aperçu", workbenchData], ["évaluation", evaluation]] as const) {
      expect(sheet, `${name} : plus de bordure de cellule`).not.toMatch(
        /\.\S*table\S*\s+(th|td)[^{]*\{[^}]*border(-right|-bottom)?:\s*1px/s,
      );
      expect(sheet, `${name} : plus de fusion des bordures`).not.toContain("border-collapse: collapse");
    }
  });
});
