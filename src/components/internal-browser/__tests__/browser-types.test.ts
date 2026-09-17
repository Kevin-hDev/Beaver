import { describe, expect, it } from "vitest";
import { resolveBrowserAddress } from "../browser-address";
import {
  normalizeBrowserUrl,
} from "../browser-types";

describe("browser types", () => {
  it("accepte uniquement les URL HTTP et HTTPS sans identifiants", () => {
    expect(normalizeBrowserUrl("https://example.com")).toBe("https://example.com/");
    expect(normalizeBrowserUrl("http://localhost:3000/a")).toBe("http://localhost:3000/a");
    expect(normalizeBrowserUrl("https://user:secret@example.com")).toBeNull();
    expect(normalizeBrowserUrl(" file:///tmp/test ")).toBeNull();
    expect(normalizeBrowserUrl(`https://example.com/${"a".repeat(2_048)}`)).toBeNull();
  });

  it("complète les domaines et les adresses localhost", () => {
    expect(resolveBrowserAddress("openai.com")).toBe("https://openai.com/");
    expect(resolveBrowserAddress("exemple.fr/docs?q=test")).toBe("https://exemple.fr/docs?q=test");
    expect(resolveBrowserAddress("localhost:5173")).toBe("http://localhost:5173/");
  });

  it("transforme le texte libre en recherche Google", () => {
    expect(resolveBrowserAddress("google")).toBe("https://www.google.com/search?q=google");
    expect(resolveBrowserAddress("guide rust français")).toBe(
      "https://www.google.com/search?q=guide+rust+fran%C3%A7ais",
    );
  });

  it("ne transforme jamais un protocole interdit en recherche", () => {
    expect(resolveBrowserAddress("javascript:alert(1)")).toBeNull();
    expect(resolveBrowserAddress("file:///tmp/test")).toBeNull();
    expect(resolveBrowserAddress("https://user:secret@example.com")).toBeNull();
  });

});
