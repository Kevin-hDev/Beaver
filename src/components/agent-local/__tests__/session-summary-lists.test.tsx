import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import {
  SessionSummaryPlanList,
  SessionSummarySubagentList,
  SessionSummaryTodoList,
} from "../session-summary-lists";
import type { AgentPlanRun, AgentTodoRun, SubagentInfo } from "@/types/agent";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key.endsWith("todoProgress")) return `${numberValue(opts?.done)}/${numberValue(opts?.total)}`;
      const parts = key.split(".");
      return parts[parts.length - 1];
    },
  }),
}));

vi.mock("../subagent-icon", () => ({
  SubagentIcon: ({ agent, className }: { agent: SubagentInfo; className: string }) => (
    <span className={`${className} ${agent.status === "running" ? "sai-claudiator sai-running" : "sai-geminitor"}`} />
  ),
}));

const runs: AgentTodoRun[] = [{
  id: "todo-1",
  title: "Implement UI",
  status: "active",
  created_at: "",
  updated_at: "",
  todos: [
    { content: "One", status: "completed" },
    { content: "Two", status: "pending" },
  ],
}];

const plan: AgentPlanRun = {
  id: "plan-1",
  title: "Plan title",
  status: "awaiting_approval",
  path: "/tmp/plan.md",
  created_at: "",
  updated_at: "",
};

const agent: SubagentInfo = {
  sessionId: "child-1",
  name: "Explorer",
  type: "explorer",
  status: "interrupted",
  promptPreview: "",
  description: "Analyse sous-agent",
};

describe("Session summary lists", () => {
  it("deplie une todo list et affiche ses taches", () => {
    const { getAllByText, getByRole, getByText } = render(<SessionSummaryTodoList runs={runs} />);

    expect(getByText("1/2")).toBeTruthy();
    fireEvent.click(getByRole("button", { name: /Implement UI/ }));

    expect(getByText("One")).toBeTruthy();
    expect(getByText("Two")).toBeTruthy();
    expect(getAllByText("pending").length).toBeGreaterThan(0);
  });

  it("ouvre le plan au clic sur son entree", () => {
    const onOpenPlan = vi.fn();
    const { getByText } = render(
      <SessionSummaryPlanList plans={[plan]} onOpenPlan={onOpenPlan} />,
    );

    fireEvent.click(getByText("Plan title"));
    expect(onOpenPlan).toHaveBeenCalledWith(plan);
  });

  it("ouvre et archive un sous-agent sans melanger les actions", () => {
    const onOpen = vi.fn();
    const onArchive = vi.fn();
    const { container, getByRole, getByText } = render(
      <SessionSummarySubagentList subagents={[agent]} onOpen={onOpen} onArchive={onArchive} />,
    );

    expect(container.querySelector(".sai-geminitor")).toBeTruthy();
    fireEvent.click(getByText("Geminitor"));
    expect(onOpen).toHaveBeenCalledWith("child-1");
    fireEvent.click(getByRole("button", { name: "archive" }));
    expect(onArchive).toHaveBeenCalledWith("child-1");
    expect(onOpen).toHaveBeenCalledOnce();
  });

  it("anime un sous-agent actif sans proposer l'archive", () => {
    const running = { ...agent, status: "running" as const };
    const { container, queryByRole } = render(
      <SessionSummarySubagentList subagents={[running]} onOpen={vi.fn()} onArchive={vi.fn()} />,
    );

    expect(container.querySelector(".sai-claudiator.sai-running")).toBeTruthy();
    expect(queryByRole("button", { name: "archive" })).toBeNull();
  });
});

function numberValue(value: unknown) {
  return typeof value === "number" || typeof value === "string" ? value : "";
}
