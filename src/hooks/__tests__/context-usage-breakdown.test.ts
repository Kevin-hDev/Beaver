import { describe, expect, it } from "vitest";
import { buildContextUsageBreakdown, CONTEXT_USAGE_KEYS } from "../context-usage-breakdown";
import { buildLiveContextMessage } from "../context-usage-live-message";
import type { AgentMessage } from "@/types/agent";

function msg(overrides: Partial<AgentMessage>): AgentMessage {
  return {
    id: crypto.randomUUID(),
    role: "user",
    content: "",
    files: [],
    timestamp: new Date().toISOString(),
    ...overrides,
  };
}

function item(tokens: ReturnType<typeof buildContextUsageBreakdown>["items"], key: string) {
  return tokens.find((entry) => entry.key === key)?.tokens ?? 0;
}

describe("context-usage-breakdown", () => {
  it("classe les messages simples dans Messages", () => {
    const breakdown = buildContextUsageBreakdown([msg({ content: "a".repeat(400) })]);

    expect(item(breakdown.items, "messages")).toBe(100);
    expect(breakdown.used).toBe(100);
  });

  it("compte le thinking dans Messages", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400), thinking: "b".repeat(400) }),
    ]);

    expect(item(breakdown.items, "messages")).toBe(200);
  });

  it("sépare les outils système et MCP", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [
          { name: "bash", summary: "npm test", result: "ok" },
          { name: "mcp_fetch", summary: "docs", result: "ok" },
        ],
      }),
    ]);

    expect(item(breakdown.items, "systemTools")).toBeGreaterThan(0);
    expect(item(breakdown.items, "mcpConnectors")).toBeGreaterThan(0);
  });

  it("garde uniquement le catalogue par défaut dans Skills", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ skill_names: ["frontend-skill"] }),
    ], { skillContextTokens: 12 });

    expect(item(breakdown.items, "skills")).toBe(12);
  });

  it("classe le contenu d'un skill chargé dans Meta contexte", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [{
          name: "load_skill",
          summary: "frontend-skill",
          result: "Instructions détaillées du skill",
        }],
      }),
    ], { skillContextTokens: 12 });

    expect(item(breakdown.items, "skills")).toBe(12);
    expect(item(breakdown.items, "metaContext")).toBeGreaterThan(0);
  });

  it("garde le system prompt isolé", () => {
    const breakdown = buildContextUsageBreakdown([], { systemPromptTokens: 42 });

    expect(item(breakdown.items, "systemPrompt")).toBe(42);
    expect(item(breakdown.items, "metaContext")).toBe(0);
  });

  it("garde la mémoire dans une catégorie dédiée", () => {
    const breakdown = buildContextUsageBreakdown([], { memoryContextTokens: 320 });

    expect(item(breakdown.items, "memory")).toBe(320);
    expect(item(breakdown.items, "metaContext")).toBe(0);
  });

  it("classe les lectures mémoire détaillées hors des outils système", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [{
          name: "read_file",
          summary: "/memory/global/topics/preference.md",
          result: "# Préférence compacte",
          domain: "memory",
        }],
      }),
    ]);

    expect(item(breakdown.items, "memory")).toBeGreaterThan(0);
    expect(item(breakdown.items, "systemTools")).toBe(0);
  });

  it("sépare les outils mémoire et projet présents dans le même message", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [
          {
            name: "read_file",
            summary: "/memory/global/topics/preference.md",
            result: "# Préférence compacte",
            domain: "memory",
          },
          {
            name: "read_file",
            summary: "/project/src/app.ts",
            result: "export const app = true;",
          },
        ],
      }),
    ]);

    expect(item(breakdown.items, "memory")).toBeGreaterThan(0);
    expect(item(breakdown.items, "systemTools")).toBeGreaterThan(0);
  });

  it("compte le payload mémoire reconstruit une seule fois", () => {
    const content = "a".repeat(400);
    const args = { path: "/memory/preference.md", content };
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [{
          name: "write_file",
          summary: "/memory/preference.md",
          args,
          result: "ok",
          content,
          domain: "memory",
        }],
      }),
    ]);
    const expectedUnits = "write_file".length + JSON.stringify(args).length + 2;

    expect(item(breakdown.items, "memory")).toBe(Math.ceil(expectedUnits / 4));
  });

  it("ajoute les lectures détaillées au résumé mémoire injecté", () => {
    const result = "b".repeat(400);
    const breakdown = buildContextUsageBreakdown([
      msg({
        role: "assistant",
        tool_activities: [{
          name: "read_file",
          summary: "/memory/preference.md",
          args: { path: "/memory/preference.md" },
          result,
          domain: "memory",
        }],
      }),
    ], { memoryContextTokens: 106 });

    expect(item(breakdown.items, "memory")).toBeGreaterThan(106);
  });

  it("range le delta observé dans Meta contexte", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400) }),
    ], { observedUsed: 180 });

    expect(item(breakdown.items, "messages")).toBe(100);
    expect(item(breakdown.items, "metaContext")).toBe(80);
  });

  it("classe les outils de la réponse live comme après sa finalisation", () => {
    const liveMessage = buildLiveContextMessage({
      completedSegments: [],
      currentContent: "réponse",
      currentThinking: "",
      currentTools: [{
        name: "bash",
        args: { command: "pwd" },
        result: "résultat détaillé",
      }],
    });
    expect(liveMessage).not.toBeNull();

    const during = buildContextUsageBreakdown([liveMessage!]);
    const finished = buildContextUsageBreakdown([msg({
      role: "assistant",
      content: liveMessage!.content,
      thinking: liveMessage!.thinking,
      tool_activities: liveMessage!.tool_activities,
    })]);

    expect(item(during.items, "messages")).toBe(item(finished.items, "messages"));
    expect(item(during.items, "systemTools")).toBe(item(finished.items, "systemTools"));
    expect(item(during.items, "systemTools")).toBeGreaterThan(0);
  });

  it("réduit les outils estimés avant les messages mieux mesurés", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({
        content: "m".repeat(400),
        role: "assistant",
        tool_activities: [{ name: "bash", summary: "x".repeat(400) }],
      }),
    ], { observedUsed: 120 });

    expect(item(breakdown.items, "messages")).toBe(100);
    expect(item(breakdown.items, "systemTools")).toBe(20);
  });

  it("respecte un total provider exact même inférieur aux estimations", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400) }),
    ], {
      observedUsed: 80,
      systemPromptTokens: 20,
    });
    const total = breakdown.items.reduce((sum, entry) => sum + entry.tokens, 0);

    expect(breakdown.used).toBe(80);
    expect(total).toBe(80);
  });

  it("garde un total cohérent avec les catégories", () => {
    const breakdown = buildContextUsageBreakdown([
      msg({ content: "a".repeat(400), skill_names: ["skill"] }),
      msg({ role: "tool", tool_name: "grep", content: "x".repeat(80) }),
    ], { systemPromptTokens: 10, metaContextTokens: 20 });
    const total = breakdown.items.reduce((sum, entry) => sum + entry.tokens, 0);

    expect(breakdown.items.map((entry) => entry.key)).toEqual(CONTEXT_USAGE_KEYS);
    expect(total).toBe(breakdown.used);
  });
});
