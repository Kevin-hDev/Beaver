import { render } from "@testing-library/react";
import { createElement } from "react";
import { describe, expect, it } from "vitest";
import { FastModeIcon } from "../fast-mode-icon";

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

  it("ne déclare l'icône Rapide qu'à un seul endroit", () => {
    expect(declarationsOf("FastModeIcon")).toEqual([
      "/src/components/ui/fast-mode-icon.tsx",
    ]);
  });

  it("rend le tracé Rapide original comme décoration thématique", () => {
    const { container } = render(createElement(FastModeIcon));
    const icon = container.querySelector("svg");
    const path = container.querySelector("path");

    expect(icon?.getAttribute("aria-hidden")).toBe("true");
    expect(icon?.getAttribute("focusable")).toBe("false");
    expect(icon?.getAttribute("viewBox")).toBe("0 0 24 24");
    expect(path?.getAttribute("fill")).toBe("currentColor");
    expect(path?.getAttribute("d")).toBe("M14.5 4h.005M14.5 4L12 10l5 2.898L9.5 20l2.5-6l-5-2.9zm0-2a2.02 2.02 0 0 0-1.379.551L5.624 9.646a2 2 0 0 0-.61 1.686c.072.626.437 1.182.982 1.498l3.482 2.021l-1.826 4.381a2.003 2.003 0 0 0 1.847 2.77c.498 0 .993-.186 1.375-.548l7.5-7.103a2 2 0 0 0 .61-1.685a2 2 0 0 0-.982-1.498L14.52 9.15l1.789-4.293A2 2 0 0 0 14.5 2");
  });
});

function declarationsOf(component: string): string[] {
  return scannedSources()
    .filter(([, content]) => content.includes(`export function ${component}`))
    .map(([path]) => path);
}
