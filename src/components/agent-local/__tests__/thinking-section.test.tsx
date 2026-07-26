import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ThinkingSection } from "../thinking-section";

afterEach(cleanup);

vi.mock("@/components/ui/icons", () => ({
  Brain: ({ className }: { className?: string }) => <span className={className} data-testid="brain-icon" />,
  CaretDown: ({ className }: { className?: string }) => <span className={className} data-testid="caret-down" />,
  CaretUp: ({ className }: { className?: string }) => <span className={className} data-testid="caret-up" />,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: { seconds?: string }) => (
      key === "agentLocal.thinkingDuration" ? `Thought ${opts?.seconds}s` : "Réflexion"
    ),
  }),
}));

describe("ThinkingSection", () => {
  it("affiche le cerveau et le chevron sans ancien caractère de flèche", () => {
    const { getByRole, getByTestId, queryByText } = render(<ThinkingSection content="raisonnement" />);

    expect(getByTestId("brain-icon")).toBeTruthy();
    expect(getByTestId("caret-down")).toBeTruthy();
    expect(queryByText("▸")).toBeNull();
    expect(getByRole("button")).toHaveAttribute("aria-expanded", "false");
  });

  it("applique la classe active uniquement quand le thinking est actif", () => {
    const inactive = render(<ThinkingSection content="raisonnement" />);
    expect(inactive.getByRole("button").querySelector(".stream-active-label")).toBeNull();
    cleanup();

    const active = render(<ThinkingSection content="raisonnement" isActive />);
    expect(active.getByRole("button").querySelector(".stream-active-label")).toBeTruthy();
  });

  it("déplie puis replie le contenu de réflexion", () => {
    const { getByRole, getByText, getByTestId } = render(<ThinkingSection content="raisonnement" />);
    const button = getByRole("button");

    fireEvent.click(button);
    expect(button).toHaveAttribute("aria-expanded", "true");
    expect(getByText("raisonnement")).toBeTruthy();
    expect(getByTestId("caret-up")).toBeTruthy();
  });
});
