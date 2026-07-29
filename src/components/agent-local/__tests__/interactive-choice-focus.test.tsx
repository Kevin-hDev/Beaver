import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { InteractiveChoicePanel } from "../interactive-choice-panel";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "interactiveChoice.otherLabel") return "Other";
      if (key === "interactiveChoice.other") return "Other answer";
      if (key === "interactiveChoice.recommended") return "Recommended";
      if (key === "interactiveChoice.step") {
        return `${String(opts?.current)}/${String(opts?.total)}`;
      }
      return key;
    },
  }),
}));
vi.mock("../interactive-choice-panel.css", () => ({}));

const request = {
  sessionId: "session-1",
  id: "choice-1",
  currentIndex: 0,
  total: 1,
  questions: [
    {
      header: "First",
      question: "First question?",
      options: [
        { id: "fast", label: "Fast", description: "Quick", recommended: true },
        { id: "complete", label: "Complete", description: "Full" },
      ],
    },
    {
      header: "Second",
      question: "Second question?",
      options: [
        { id: "safe", label: "Safe", description: "Careful", recommended: true },
        { id: "flexible", label: "Flexible", description: "Adaptable" },
      ],
    },
  ],
};

afterEach(cleanup);

beforeEach(() => {
  vi.mocked(invoke).mockResolvedValue(undefined);
});

describe("InteractiveChoicePanel focus", () => {
  it("rend le clavier au choix survolé après un focus extérieur", () => {
    render(
      <>
        <button type="button">Outside</button>
        <InteractiveChoicePanel request={request} />
      </>,
    );
    const outside = screen.getByRole("button", { name: "Outside" });
    const complete = screen.getByRole("button", { name: /Complete/ });
    outside.focus();

    fireEvent.mouseEnter(complete);

    expect(complete).toHaveFocus();
  });

  it("focalise la première option de la question suivante", async () => {
    render(<InteractiveChoicePanel request={request} />);
    const fast = screen.getByRole("button", { name: /Fast/ });
    const complete = screen.getByRole("button", { name: /Complete/ });

    fireEvent.keyDown(fast, { key: "ArrowDown" });
    fireEvent.keyDown(complete, { key: "Enter" });

    await waitFor(() => expect(screen.getByText("Second question?")).toBeTruthy());
    expect(screen.getByRole("button", { name: /Safe/ })).toHaveFocus();
  });
});
