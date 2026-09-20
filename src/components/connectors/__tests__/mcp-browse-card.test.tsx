import { render, screen } from "@testing-library/react";
import { expect, it, vi } from "vitest";
import { MCP_CATALOG } from "@/lib/mcp-catalog";
import { McpBrowseCard } from "../mcp-browse-card";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key, i18n: { language: "fr" } }),
}));
vi.mock("@tauri-apps/plugin-shell", () => ({ open: vi.fn() }));

it("désactive l'ajout de Slack", () => {
  const slack = MCP_CATALOG.find((entry) => entry.id === "slack")!;
  render(<McpBrowseCard connector={slack} configured={false} onAdd={vi.fn()} />);

  expect(screen.getByText("connectors.comingSoon")).toBeInTheDocument();
  const buttons = screen.getAllByRole("button");
  expect(buttons[buttons.length - 1].hasAttribute("disabled")).toBe(true);
});
