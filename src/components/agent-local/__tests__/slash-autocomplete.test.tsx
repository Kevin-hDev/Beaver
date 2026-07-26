import { render } from "@testing-library/react";
import { afterAll, beforeAll, describe, expect, it, vi } from "vitest";
import type { SkillInfo } from "@/types/agent";
import { MAGIC_WAND_PATH } from "../skill-chip-icons";
import { SlashAutocomplete } from "../slash-autocomplete";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

const originalScrollIntoView = Object.getOwnPropertyDescriptor(
  Element.prototype,
  "scrollIntoView",
);

beforeAll(() => {
  Object.defineProperty(Element.prototype, "scrollIntoView", {
    configurable: true,
    value: vi.fn(),
  });
});

afterAll(() => {
  if (originalScrollIntoView) {
    Object.defineProperty(
      Element.prototype,
      "scrollIntoView",
      originalScrollIntoView,
    );
  } else {
    Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  }
});

const skill: SkillInfo = {
  id: "local:skill:context7-docs",
  name: "context7-docs",
  command: "context7-docs",
  description: "Documentation à jour",
  path: "/skills/context7-docs/SKILL.md",
  source: "local",
  source_name: "CL-GO-DASH",
};

describe("SlashAutocomplete", () => {
  it("affiche la même baguette magique que les chips de skill", () => {
    const { container } = render(
      <SlashAutocomplete
        skills={[skill]}
        activeIndex={0}
        onSelect={vi.fn()}
      />,
    );

    const iconPath = container.querySelector(".slash-item-icon path");
    expect(iconPath).toHaveAttribute("d", MAGIC_WAND_PATH);
  });
});
