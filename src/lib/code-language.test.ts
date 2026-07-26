import { describe, it, expect } from "vitest";
import { languageFromPath, shouldWrapFile } from "./code-language";

// --- languageFromPath -------------------------------------------------------

describe("languageFromPath", () => {
  it("détecte le langage par extension (typescript)", () => {
    expect(languageFromPath("src/main.ts")).toBe("typescript");
  });

  it("détecte rust", () => {
    expect(languageFromPath("src/main.rs")).toBe("rust");
  });

  it("détecte python", () => {
    expect(languageFromPath("script.py")).toBe("python");
  });

  it("détecte go", () => {
    expect(languageFromPath("main.go")).toBe("go");
  });

  it("détecte les extensions JSX/TSX", () => {
    expect(languageFromPath("Component.jsx")).toBe("jsx");
    expect(languageFromPath("Component.tsx")).toBe("tsx");
  });

  it("détecte .h comme c (header C)", () => {
    expect(languageFromPath("header.h")).toBe("c");
  });

  it("détecte yaml et yml", () => {
    expect(languageFromPath("config.yaml")).toBe("yaml");
    expect(languageFromPath("config.yml")).toBe("yaml");
  });

  it("retourne 'text' pour une extension inconnue", () => {
    expect(languageFromPath("file.xyz")).toBe("text");
  });

  it("retourne 'text' pour un fichier sans extension", () => {
    expect(languageFromPath("Makefile")).toBe("text");
  });

  it("est insensible à la casse de l'extension", () => {
    expect(languageFromPath("MAIN.RS")).toBe("rust");
    expect(languageFromPath("Script.PY")).toBe("python");
  });

  it("gère les chemins avec points multiples", () => {
    expect(languageFromPath("my.config.file.json")).toBe("json");
  });

  it("retourne 'text' pour une chaîne vide", () => {
    expect(languageFromPath("")).toBe("text");
  });
});

// --- shouldWrapFile ---------------------------------------------------------

describe("shouldWrapFile", () => {
  it("retourne true pour les extensions à wrapper (md, json, yaml)", () => {
    expect(shouldWrapFile("README.md")).toBe(true);
    expect(shouldWrapFile("data.json")).toBe(true);
    expect(shouldWrapFile("config.yaml")).toBe(true);
    expect(shouldWrapFile("config.yml")).toBe(true);
  });

  it("retourne true pour txt, csv, log, toml", () => {
    expect(shouldWrapFile("notes.txt")).toBe(true);
    expect(shouldWrapFile("data.csv")).toBe(true);
    expect(shouldWrapFile("app.log")).toBe(true);
    expect(shouldWrapFile("Cargo.toml")).toBe(true);
  });

  it("retourne false pour du code source (rs, ts, py)", () => {
    expect(shouldWrapFile("main.rs")).toBe(false);
    expect(shouldWrapFile("index.ts")).toBe(false);
    expect(shouldWrapFile("script.py")).toBe(false);
  });

  it("retourne false pour une extension inconnue", () => {
    expect(shouldWrapFile("file.xyz")).toBe(false);
  });

  it("retourne false pour un fichier sans extension", () => {
    expect(shouldWrapFile("Makefile")).toBe(false);
  });

  it("est insensible à la casse", () => {
    expect(shouldWrapFile("README.MD")).toBe(true);
    expect(shouldWrapFile("DATA.JSON")).toBe(true);
  });
});
