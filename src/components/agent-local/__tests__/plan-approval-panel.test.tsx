import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { InteractiveChoicePanel } from "../interactive-choice-panel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "interactiveChoice.planQuestion": "Implement this plan?",
      "interactiveChoice.planImplement": "Implement this plan",
      "interactiveChoice.planAdjustments": "Request adjustments",
      "interactiveChoice.ignore": "Ignore",
      "interactiveChoice.send": "Send",
    })[key] ?? key,
  }),
}));
vi.mock("../interactive-choice-panel.css", () => ({}));
vi.mock("../plan-approval-panel.css", () => ({}));

const request = {
  sessionId: "session-1",
  id: "plan-approval-1",
  kind: "plan_approval" as const,
  currentIndex: 0,
  total: 1,
  questions: [{
    header: "Plan",
    question: "Backend fallback",
    options: [{
      id: "implement_plan",
      label: "Implement",
      description: "Start implementation",
      recommended: true,
    }],
  }],
};

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

beforeEach(() => {
  vi.mocked(invoke).mockResolvedValue(undefined);
});

describe("PlanApprovalPanel", () => {
  it("affiche seulement la validation et les ajustements sur deux lignes", () => {
    const { container } = render(<InteractiveChoicePanel request={request} />);

    expect(screen.getByText("Implement this plan?")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Implement this plan" })).toBeTruthy();
    expect(screen.getByPlaceholderText("Request adjustments")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Ignore/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Send" })).toBeTruthy();
    expect(container.querySelector(".pap-implement")).toBeTruthy();
    expect(container.querySelector(".pap-adjust-row")).toBeTruthy();
    expect(screen.queryByText("Continue planning")).toBeNull();
    expect(screen.queryByText("Quit Plan Mode")).toBeNull();
  });

  it("valide le plan avec son identifiant stable", async () => {
    render(<InteractiveChoicePanel request={request} />);

    fireEvent.click(screen.getByRole("button", { name: "Implement this plan" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "respond_to_interactive_choice",
      {
        sessionId: "session-1",
        id: "plan-approval-1",
        answers: [{
          questionIndex: 0,
          selectedIds: ["implement_plan"],
          selectedLabels: ["Implement this plan"],
        }],
      },
    ));
  });

  it("envoie les ajustements comme réponse libre", async () => {
    render(<InteractiveChoicePanel request={request} />);
    fireEvent.change(screen.getByPlaceholderText("Request adjustments"), {
      target: { value: "Use the existing service" },
    });

    fireEvent.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "respond_to_interactive_choice",
      {
        sessionId: "session-1",
        id: "plan-approval-1",
        answers: [{
          questionIndex: 0,
          selectedIds: ["other"],
          selectedLabels: ["other"],
          customAnswer: "Use the existing service",
        }],
      },
    ));
  });

  it("ignore avec Échap sans fabriquer de réponse", async () => {
    const onResolved = vi.fn();
    render(<InteractiveChoicePanel request={request} onResolved={onResolved} />);

    fireEvent.keyDown(
      screen.getByRole("button", { name: "Implement this plan" }),
      { key: "Escape" },
    );

    await waitFor(() => expect(invoke).toHaveBeenCalledWith(
      "dismiss_interactive_choice",
      { sessionId: "session-1", id: "plan-approval-1" },
    ));
    expect(onResolved).toHaveBeenCalledOnce();
  });

  it("ne traite pas Échap quand le focus appartient à un autre panneau", () => {
    render(
      <>
        <button type="button">Other panel</button>
        <InteractiveChoicePanel request={request} />
      </>,
    );
    const otherPanel = screen.getByRole("button", { name: "Other panel" });
    otherPanel.focus();

    fireEvent.keyDown(otherPanel, { key: "Escape" });

    expect(invoke).not.toHaveBeenCalled();
  });

  it("remonte les échecs de validation et d’annulation", async () => {
    const onError = vi.fn();
    vi.mocked(invoke).mockRejectedValue(new Error("unavailable"));
    render(<InteractiveChoicePanel request={request} onError={onError} />);

    fireEvent.click(screen.getByRole("button", { name: "Implement this plan" }));
    await waitFor(() => expect(onError).toHaveBeenCalledTimes(1));

    fireEvent.click(screen.getByRole("button", { name: /Ignore/ }));
    await waitFor(() => expect(onError).toHaveBeenCalledTimes(2));
  });
});
