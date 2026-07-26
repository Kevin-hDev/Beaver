import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { OnboardingWelcome } from "../onboarding-welcome";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  CaretRight: () => <span />,
}));

describe("OnboardingWelcome", () => {
  it("affiche le castor comme marque décorative", () => {
    const { container } = render(<OnboardingWelcome onNext={vi.fn()} />);

    const mark = container.querySelector(".ob-brand-castor");
    expect(mark).not.toBeNull();
    expect(mark).toHaveAttribute("aria-hidden", "true");
  });
});
