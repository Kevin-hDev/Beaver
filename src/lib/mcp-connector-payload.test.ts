import { describe, it, expect } from "vitest";
import { connectorPayload } from "./mcp-connector-payload";
import type { McpConnectorSpec } from "@/types/mcp";

function spec(overrides: Partial<McpConnectorSpec> = {}): McpConnectorSpec {
  return {
    id: "notion",
    display_name: "Notion",
    category: "productivity",
    auth_type: "oauth",
    short_descriptions: {
      en: "Notion",
      fr: "Notion",
      es: "Notion",
      de: "Notion",
      it: "Notion",
      zh: "Notion",
      ja: "Notion",
    },
    author: "test",
    url: "https://notion.com",
    tools: [],
    endpoint: "https://mcp.notion.com/mcp",
    ...overrides,
  };
}

describe("connectorPayload", () => {
  it("construit un payload avec les valeurs par défaut (connected, enabled)", () => {
    const result = connectorPayload(spec());
    expect(result.id).toBe("notion");
    expect(result.status).toBe("connected");
    expect(result.enabled_in_chat).toBe(true);
  });

  it("préserve l'endpoint du spec", () => {
    const result = connectorPayload(spec({ endpoint: "https://custom.example.com/mcp" }));
    expect(result.endpoint).toBe("https://custom.example.com/mcp");
  });

  it("préserve l'install_command du spec", () => {
    const result = connectorPayload(
      spec({ endpoint: undefined, install_command: "npx @test/mcp@1.0" }),
    );
    expect(result.install_command).toBe("npx @test/mcp@1.0");
  });

  it("préserve les env_keys du spec", () => {
    const result = connectorPayload(spec({ env_keys: ["API_TOKEN", "USER_ID"] }));
    expect(result.env_keys).toEqual(["API_TOKEN", "USER_ID"]);
  });

  it("gère un spec avec endpoint absent (stdio only)", () => {
    const result = connectorPayload(
      spec({ endpoint: undefined, install_command: "uvx mcp-server" }),
    );
    expect(result.endpoint).toBeUndefined();
    expect(result.install_command).toBe("uvx mcp-server");
  });
});
