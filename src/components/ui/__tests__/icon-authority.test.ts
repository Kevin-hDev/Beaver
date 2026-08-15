import { describe, expect, it } from "vitest";

/* Un dossier et une discussion ont chacun un seul dessin dans l'application.
   Trois dessins concurrents cohabitaient : l'un dans la barre latérale, un
   autre dans le sélecteur de dossier, un troisième dans les chats archivés. Rien
   ne les reliait, donc rien ne signalait qu'ils avaient divergé. Ce test relit
   les sources plutôt que le rendu : c'est l'import qui fait la divergence.

   Les fichiers de tests sont écartés du balayage : ils citent les noms retirés
   pour les remplacer par des doublures, et celui-ci les cite pour les chercher. */
const SOURCES: Record<string, string> = import.meta.glob("/src/**/*.{ts,tsx}", { eager: true, query: "?raw", import: "default" });

/* Dessins de la bibliothèque externe remplacés par les primitives maison.
   Les réintroduire ferait réapparaître deux dossiers dans la même page. */
const BANNED = ["FolderSimple", "FolderSimplePlus", "ChatsCircle", "CopyMessageIcon"];

function scannedSources(): [string, string][] {
  return Object.entries(SOURCES).filter(([path]) => !path.includes("/__tests__/"));
}

describe("autorité unique des dessins de dossier et de discussion", () => {
  it("n'importe plus les dessins remplacés", () => {
    const offenders = scannedSources()
      .filter(([, content]) => BANNED.some((name) => content.includes(name)))
      .map(([path]) => path);

    expect(offenders).toEqual([]);
  });

  it("ne déclare la discussion qu'à un seul endroit", () => {
    const declarations = declarationsOf("SessionIcon");

    expect(declarations).toEqual(["/src/components/ui/session-icon.tsx"]);
  });

  it("ne déclare la copie qu'à un seul endroit", () => {
    const declarations = declarationsOf("CopyIcon");

    expect(declarations).toEqual(["/src/components/ui/copy-icon.tsx"]);
  });
});

function declarationsOf(component: string): string[] {
  return scannedSources()
    .filter(([, content]) => content.includes(`export function ${component}`))
    .map(([path]) => path);
}
