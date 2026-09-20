import { expect, it } from "vitest";
import { MCP_CATALOG } from "./mcp-catalog";
import endpoints from "@/config/mcp-endpoints.json";

it("Lucid utilise le chemin officiel, partagé avec le backend", () => {
  expect(endpoints.lucid).toBe("https://mcp.lucid.app/mcp");
  expect(MCP_CATALOG.find((entry) => entry.id === "lucid")?.endpoint)
    .toBe(endpoints.lucid);
});

it("Slack reste non activable sans identité d'application", () => {
  expect(MCP_CATALOG.find((entry) => entry.id === "slack")?.coming_soon).toBe(true);
});

it("Reddit ne promet pas d'écriture sans identifiants", () => {
  const reddit = MCP_CATALOG.find((entry) => entry.id === "reddit");
  expect(reddit?.tools).toEqual(["read_posts", "search", "trending"]);
  expect(reddit?.short_descriptions.fr).toContain("sans identifiants");
});
