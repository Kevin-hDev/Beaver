import { createRef } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { BranchSelectorCreateForm } from "../branch-selector-create-form";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  Check: () => <span data-testid="check-icon" />,
}));

function renderForm(overrides: Partial<React.ComponentProps<typeof BranchSelectorCreateForm>> = {}) {
  const props = {
    inputRef: createRef<HTMLInputElement>(),
    value: "feature/foo",
    error: "",
    isCreating: false,
    placeholder: "branch name",
    onValueChange: vi.fn(),
    onSubmit: vi.fn(),
    onCancel: vi.fn(),
    ...overrides,
  };
  render(<BranchSelectorCreateForm {...props} />);
  return props;
}

describe("BranchSelectorCreateForm", () => {
  it("active le submit quand le nom est valide", () => {
    renderForm();

    expect(screen.getByLabelText("branches.createSubmit")).not.toBeDisabled();
  });

  it("désactive le submit et affiche une erreur quand le nom est invalide", () => {
    renderForm({ value: "foo..bar" });

    expect(screen.getByLabelText("branches.createSubmit")).toBeDisabled();
    expect(screen.getByText("branches.errorInvalidName")).toBeTruthy();
  });

  it("désactive l'input et le submit pendant la création", () => {
    renderForm({ isCreating: true });

    expect(screen.getByPlaceholderText("branch name")).toBeDisabled();
    expect(screen.getByLabelText("branches.createSubmit")).toBeDisabled();
  });

  it("soumet avec Entrée quand le nom est valide", () => {
    const props = renderForm();

    fireEvent.keyDown(screen.getByPlaceholderText("branch name"), { key: "Enter" });

    expect(props.onSubmit).toHaveBeenCalledOnce();
  });

  it("ne soumet pas pendant isCreating même avec Entrée", () => {
    const props = renderForm({ isCreating: true });

    fireEvent.keyDown(screen.getByPlaceholderText("branch name"), { key: "Enter" });

    expect(props.onSubmit).not.toHaveBeenCalled();
  });
});
