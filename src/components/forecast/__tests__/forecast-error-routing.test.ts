import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/* Deux sortes d'échec, deux traitements décidés avec Kevin :
   — une ACTION qui échoue (lancer, enregistrer, supprimer) se signale par une
     notification qui s'efface ;
   — une ZONE qui n'a pas pu charger garde son message, sinon elle reste vide
     sans que rien n'en dise la raison.

   Ce test tient la première moitié : plus aucun échec d'action ne s'installe
   dans la page. */

const ACTIONS = {
  "évaluation — backtest et ensemble":
    "src/components/forecast/evaluation/use-forecast-evaluation.ts",
  "historique — renommage et suppression":
    "src/components/forecast/sections/forecast-history.tsx",
  "scénarios — enregistrement et suppression":
    "src/components/forecast/sections/use-forecast-scenario-form.ts",
  "plan de travail — enregistrement du brouillon":
    "src/components/forecast/workbench/forecast-workbench-window.tsx",
  "panneau — import de fichier et lancement":
    "src/components/forecast/forecast-panel.tsx",
};

const ZONES = {
  "aperçu des données": "src/components/forecast/workbench/forecast-workbench-data.tsx",
  "vue principale": "src/components/forecast/sections/forecast-view.tsx",
  "comparaisons": "src/components/forecast/sections/forecast-comparisons.tsx",
  "notes": "src/components/forecast/sections/forecast-notes.tsx",
  "analyse": "src/components/forecast/sections/forecast-analysis.tsx",
};

/* Les classes des messages d'action supprimés. `-state-error`, qui habille un
   état de zone, n'en fait pas partie : celui-là doit rester. */
const RETIREES = {
  ".fcwe-error": "src/components/forecast/evaluation/forecast-evaluation.css",
  ".fc-empty-error": "src/components/forecast/forecast-empty.css",
  ".fcs-error": "src/components/forecast/sections/forecast-scenario-form.css",
  ".fcw-inline-error": "src/components/forecast/workbench/forecast-workbench-window.tsx",
};

describe("Erreurs forecast — action ou état de zone", () => {
  it("signale chaque échec d'action par une notification", () => {
    for (const [nom, path] of Object.entries(ACTIONS)) {
      // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins fixes déclarés ci-dessus
      const source = readFileSync(path, "utf8");
      expect(source, nom).toContain('showToast');
    }
  });

  it("laisse chaque zone garder son propre message d'échec", () => {
    for (const [nom, path] of Object.entries(ZONES)) {
      // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins fixes déclarés ci-dessus
      const source = readFileSync(path, "utf8");
      expect(source, nom).toMatch(/if \(error\)/);
    }
  });

  it("ne garde aucune trace des messages rouges retirés", () => {
    for (const [classe, path] of Object.entries(RETIREES)) {
      // eslint-disable-next-line security/detect-non-literal-fs-filename -- chemins fixes déclarés ci-dessus
      const source = readFileSync(path, "utf8");
      expect(source, classe).not.toContain(classe.slice(1));
    }
  });
});
