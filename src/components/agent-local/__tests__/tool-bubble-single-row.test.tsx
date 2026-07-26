import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@testing-library/react";
import { ToolBubble } from "../tool-bubble";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretUp: () => <span />,
  Check: () => <span />,
  Copy: () => <span />,
  Spinner: () => <span data-testid="spinner" />,
}));

vi.mock("../tool-icons", () => ({
  ToolIcon: ({ name }: { name: string }) => <span data-testid={`tool-icon-${name}`} />,
}));

vi.mock("../tool-status-icon", () => ({
  ToolStatusIcon: () => <span data-testid="status-icon-error" />,
}));

vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: ({ name }: { name: string }) => <span data-testid={`file-icon-${name}`} />,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "en" },
    t: (key: string, opts?: Record<string, unknown>) => {
      const rawCount = opts?.count;
      const count = typeof rawCount === "string" || typeof rawCount === "number" ? String(rawCount) : "";
      if (key === "agentLocal.toolActivity.groups.command") return "Commands";
      if (key === "agentLocal.toolActivity.groups.exploration") return "Explore";
      if (key === "agentLocal.toolActivity.counts.commands") return `${count} command`;
      if (key === "agentLocal.toolActivity.counts.searches") return `${count} search`;
      if (key === "agentLocal.toolActivity.counts.lists") return `${count} list`;
      if (key === "agentLocal.toolActivity.actions.list") return "List";
      if (key === "agentLocal.toolActivity.actions.search") return "Search";
      return key;
    },
  }),
}));

vi.mock("@/lib/tool-file-path", () => ({
  isFileTool: () => false,
}));

describe("ToolBubble single row rendering", () => {
  it("affiche directement un seul tool visible", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "npm test" } }]} />,
    );

    expect(container.querySelector(".tb-group-toggle")).toBeNull();
    expect(container.querySelector(".tb-row")).not.toBeNull();
    expect(container.textContent).toContain("bash");
    expect(container.textContent).toContain("npm test");
  });

  it("place le chevron de détail à droite du libellé du tool", () => {
    const { container } = render(
      <ToolBubble tools={[{ name: "bash", args: { command: "npm test" }, result: "ok" }]} />,
    );
    const toggle = container.querySelector(".tb-toggle");
    if (!toggle) throw new Error("tool toggle absent");

    const children = Array.from(toggle.children);
    expect(children[0]?.getAttribute("data-testid")).toBe("tool-icon-TerminalWindow");
    expect(children[1]?.textContent).toBe("bash");
    expect(children[2]?.classList.contains("tb-tool-arrow")).toBe(true);
  });

  it("ignore les tools cachés pour décider du rendu direct", () => {
    const { container } = render(
      <ToolBubble
        tools={[
          { name: "ask_user_choice", args: { questions: [] }, result: "ok" },
          { name: "bash", args: { command: "npm test" } },
        ]}
      />,
    );

    expect(container.querySelector(".tb-group-toggle")).toBeNull();
    expect(container.querySelector(".tb-row")).not.toBeNull();
    expect(container.textContent).toContain("npm test");
  });

  it("garde les groupes quand plusieurs tools visibles existent dans le même groupe", () => {
    const { container } = render(
      <ToolBubble
        tools={[
          { name: "grep", args: { pattern: "ToolBubble" }, result: "hit" },
          { name: "glob", args: { pattern: "*.tsx" }, result: "hit" },
        ]}
      />,
    );

    expect(container.querySelector(".tb-group-toggle")).not.toBeNull();
    expect(container.querySelector(".tb-row")).toBeNull();
    expect(container.textContent).toContain("Explore");
  });

  it("affiche directement un groupe contenant un seul tool", () => {
    const { container } = render(
      <ToolBubble
        tools={[
          { name: "grep", args: { pattern: "ToolBubble" }, result: "hit" },
          { name: "list_dir", args: { path: "." }, result: "src" },
          { name: "bash", args: { command: "npm test" }, result: "ok" },
        ]}
      />,
    );

    expect(container.querySelectorAll(".tb-group-toggle")).toHaveLength(1);
    expect(container.textContent).toContain("Explore");
    expect(container.textContent).not.toContain("Commands");
    expect(container.textContent).toContain("bash");
    expect(container.textContent).toContain("npm test");
  });
});
