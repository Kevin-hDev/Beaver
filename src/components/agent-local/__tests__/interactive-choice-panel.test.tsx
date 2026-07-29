import { readFileSync } from "node:fs";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { InteractiveChoicePanel } from "../interactive-choice-panel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text = (value: unknown) => (
        typeof value === "string" || typeof value === "number" ? String(value) : ""
      );
      if (key === "interactiveChoice.otherLabel") return "Other";
      if (key === "interactiveChoice.other") return "Other answer";
      if (key === "interactiveChoice.recommended") return "Recommended";
      if (key === "interactiveChoice.otherPlaceholder") return "Write your answer";
      if (key === "interactiveChoice.step") return `${text(opts?.current)}/${text(opts?.total)}`;
      return key;
    },
  }),
}));
vi.mock("../interactive-choice-panel.css", () => ({}));

const optionCss = readFileSync(
  "src/components/agent-local/interactive-choice-option.css",
  "utf8",
);

const request = {
  sessionId: "session-1",
  id: "choice-1",
  currentIndex: 0,
  total: 1,
  questions: [{
    header: "Plan",
    question: "What next?",
    options: [
      { label: "Fast", description: "Do the minimum", recommended: true },
      { id: "complete", label: "Complete", description: "Do the full pass" },
    ],
  }],
};

afterEach(cleanup);

beforeEach(() => {
  vi.mocked(invoke).mockResolvedValue(undefined);
});

describe("InteractiveChoicePanel", () => {
  it("affiche la question et les choix", () => {
    const { container } = render(<InteractiveChoicePanel request={request} />);

    expect(screen.getByText("What next?")).toBeTruthy();
    expect(screen.getByText("Fast")).toBeTruthy();
    expect(screen.getByText("Recommended")).toBeTruthy();
    expect(container.querySelector(".icp-option-marker")?.textContent).toBe("1");
    expect(container.querySelectorAll(".icp-option-marker")).toHaveLength(3);
    expect(container.querySelector(".icp-option-marker-icon")).toBeTruthy();
  });

  it("focalise le panneau sans choisir automatiquement une option", () => {
    render(<InteractiveChoicePanel request={request} />);
    const panel = screen.getByRole("group", { name: "interactiveChoice.title" });

    expect(panel).toHaveFocus();
    fireEvent.keyDown(panel, { key: "Enter" });
    fireEvent.keyDown(panel, { key: " " });

    expect(invoke).not.toHaveBeenCalled();
  });

  it("garde le titre et la recommandation complets avant de tronquer la description", () => {
    expect(optionCss).toMatch(
      /\.icp-option-label-host,\s*\.icp-recommended-host\s*\{[^}]*flex:\s*0 0 auto;/,
    );
    expect(optionCss).toMatch(
      /\.icp-option-description\s*\{[^}]*flex:\s*1 1 auto;/,
    );
  });

  it("valide un choix au clic", async () => {
    const onResolved = vi.fn();
    render(<InteractiveChoicePanel request={request} onResolved={onResolved} />);

    fireEvent.click(screen.getByText("Complete"));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("respond_to_interactive_choice", {
      sessionId: "session-1",
      id: "choice-1",
      answers: [{ questionIndex: 0, selectedIds: ["complete"], selectedLabels: ["Complete"] }],
    }));
    expect(onResolved).toHaveBeenCalledOnce();
  });

  it("navigue avec les flèches et valide avec Entrée", async () => {
    render(<InteractiveChoicePanel request={request} />);
    const panel = screen.getByRole("group", { name: "interactiveChoice.title" });
    const completeOption = screen.getByRole("button", { name: /Complete/ });

    fireEvent.keyDown(panel, { key: "ArrowDown" });
    expect(completeOption).toHaveFocus();
    fireEvent.keyDown(completeOption, { key: "Enter" });

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("respond_to_interactive_choice", {
      sessionId: "session-1",
      id: "choice-1",
      answers: [{ questionIndex: 0, selectedIds: ["complete"], selectedLabels: ["Complete"] }],
    }));
  });

  it("ouvre Autre et envoie la réponse libre", async () => {
    render(<InteractiveChoicePanel request={request} />);

    fireEvent.click(screen.getByText("Other"));
    fireEvent.change(screen.getByPlaceholderText("Write your answer"), {
      target: { value: "Use a custom path" },
    });
    fireEvent.keyDown(screen.getByPlaceholderText("Write your answer"), { key: "Enter" });

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("respond_to_interactive_choice", {
      sessionId: "session-1",
      id: "choice-1",
      answers: [{
        questionIndex: 0,
        selectedIds: ["other"],
        selectedLabels: ["other"],
        customAnswer: "Use a custom path",
      }],
    }));
  });

  it("rend le focus aux choix en fermant Autre avec Échap", () => {
    render(<InteractiveChoicePanel request={request} />);
    const panel = screen.getByRole("group", { name: "interactiveChoice.title" });
    const otherOption = screen.getByRole("button", { name: /Other/ });

    fireEvent.keyDown(panel, { key: "ArrowUp" });
    fireEvent.keyDown(otherOption, { key: "Enter" });
    const input = screen.getByPlaceholderText("Write your answer");
    fireEvent.keyDown(input, { key: "Escape" });

    expect(screen.queryByPlaceholderText("Write your answer")).toBeNull();
    expect(otherOption).toHaveFocus();
  });

  it("annule proprement avec Échap", async () => {
    const onResolved = vi.fn();
    render(<InteractiveChoicePanel request={request} onResolved={onResolved} />);

    fireEvent.keyDown(screen.getByRole("button", { name: /Fast/ }), { key: "Escape" });

    await waitFor(() => expect(invoke).toHaveBeenCalledWith("dismiss_interactive_choice", {
      sessionId: "session-1",
      id: "choice-1",
    }));
    expect(onResolved).toHaveBeenCalledOnce();
  });

  it("remonte un échec au panneau parent", async () => {
    const onError = vi.fn();
    vi.mocked(invoke).mockRejectedValueOnce(new Error("unavailable"));
    render(<InteractiveChoicePanel request={request} onError={onError} />);

    fireEvent.click(screen.getByText("Complete"));

    await waitFor(() => expect(onError).toHaveBeenCalledOnce());
  });
});
