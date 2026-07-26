import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { PlanModeBadge } from "../plan-mode-badge";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "chatMenu.planMode") return "Plan mode";
      if (key === "chatMenu.disablePlanMode") return "Disable plan mode";
      return key;
    },
  }),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("PlanModeBadge", () => {
  it("affiche le badge et désactive au clic", () => {
    const onDisable = vi.fn();
    const { getByLabelText, getByText } = render(<PlanModeBadge onDisable={onDisable} />);

    expect(getByText("Plan mode")).toBeTruthy();
    fireEvent.click(getByLabelText("Disable plan mode"));

    expect(onDisable).toHaveBeenCalledOnce();
  });
});
