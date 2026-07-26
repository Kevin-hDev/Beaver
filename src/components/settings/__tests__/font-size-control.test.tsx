import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FontSizeControl } from "../font-size-control";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const labels: Record<string, string> = {
        "settings.general.fontSizeIncrease": "Increase font size",
        "settings.general.fontSizeDecrease": "Decrease font size",
      };
      return labels[key] ?? key;
    },
  }),
}));

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("FontSizeControl", () => {
  it("applique une valeur tapée dans la plage autorisée", () => {
    const onChange = vi.fn();
    render(<FontSizeControl value={18} onChange={onChange} />);

    fireEvent.change(screen.getByRole("spinbutton"), { target: { value: "22" } });

    expect(onChange).toHaveBeenLastCalledWith(22);
  });

  it("incrémente et décrémente avec les boutons", () => {
    const onChange = vi.fn();
    render(<FontSizeControl value={18} onChange={onChange} />);

    fireEvent.click(screen.getByLabelText("Increase font size"));
    fireEvent.click(screen.getByLabelText("Decrease font size"));

    expect(onChange).toHaveBeenNthCalledWith(1, 19);
    expect(onChange).toHaveBeenNthCalledWith(2, 17);
  });

  it("limite la valeur au blur", () => {
    const onChange = vi.fn();
    render(<FontSizeControl value={18} onChange={onChange} />);
    const input = screen.getByRole("spinbutton");

    fireEvent.change(input, { target: { value: "40" } });
    fireEvent.blur(input);

    expect(onChange).toHaveBeenLastCalledWith(24);
  });
});
