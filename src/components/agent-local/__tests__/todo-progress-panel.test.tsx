import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { TodoProgressPanel } from "../todo-progress-panel";
import { useTodos } from "@/hooks/use-todos";

vi.mock("@/hooks/use-todos", () => ({
  useTodos: vi.fn(),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, values?: Record<string, unknown>) => {
      if (key === "todos.progress") return `${String(values?.done)}/${String(values?.total)} tasks`;
      if (key === "todos.noActive") return "Todo complete";
      if (key === "todos.status.pending") return "pending";
      if (key === "todos.status.in_progress") return "in progress";
      if (key === "todos.status.completed") return "completed";
      return key;
    },
  }),
}));

vi.mock("../todo-progress-panel.css", () => ({}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("TodoProgressPanel", () => {
  it("ne rend rien sans tâches", () => {
    vi.mocked(useTodos).mockReturnValue([]);

    const { container } = render(<TodoProgressPanel sessionId="s1" />);

    expect(container.innerHTML).toBe("");
  });

  it("affiche le compteur et la tâche en cours", () => {
    vi.mocked(useTodos).mockReturnValue([
      { content: "Lire", status: "completed" },
      { content: "Implémenter", active_form: "Implémente", status: "in_progress" },
      { content: "Tester", status: "pending" },
    ]);

    const { getAllByText, getByText } = render(<TodoProgressPanel sessionId="s1" />);

    expect(getByText("1/3 tasks")).toBeTruthy();
    expect(getAllByText("Implémente")).toHaveLength(2);
    expect(getByText("33%")).toBeTruthy();
  });

  it("déplie la liste complète au clic", () => {
    vi.mocked(useTodos).mockReturnValue([
      { content: "Lire", status: "completed" },
      { content: "Tester", status: "pending" },
    ]);

    const { getAllByText, getByRole, getByText } = render(<TodoProgressPanel sessionId="s1" />);
    fireEvent.click(getByRole("button"));

    expect(getByText("Lire")).toBeTruthy();
    expect(getAllByText("Tester")).toHaveLength(2);
    expect(getByText("completed")).toBeTruthy();
  });
});
