import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { OperationProgressAction } from "./operation-progress-action";
import { UpdateProgressAction } from "../updates/update-progress-action";

afterEach(cleanup);
const labels = { cancelLabel: "Annuler", cancellingLabel: "Annulation", phaseLabel: "Installation" };

describe("operation progress", () => {
  it("does not invent a percentage and allows cancellation by keyboard", () => {
    const cancel = vi.fn();
    render(<OperationProgressAction {...labels} percent={null} cancelling={false} canCancel onCancel={cancel} />);
    expect(screen.getByRole("progressbar", { name: "Installation" })).not.toHaveAttribute("aria-valuenow");
    const button = screen.getByRole("button", { name: "Annuler" });
    button.focus();
    expect(button).toHaveFocus();
    fireEvent.click(button);
    expect(cancel).toHaveBeenCalledOnce();
  });
  it("keeps the updater contract and clamps known totals", () => {
    render(<UpdateProgressAction {...labels} percent={120} cancelling={false} onCancel={vi.fn()} compact />);
    expect(screen.getByRole("progressbar")).toHaveAttribute("aria-valuenow", "100");
    expect(screen.getByText("100%")).toBeVisible();
  });
  it("disables repeated cancellation and stops the indeterminate animation", () => {
    const cancel = vi.fn();
    const { container } = render(<OperationProgressAction {...labels} percent={null} cancelling canCancel onCancel={cancel} />);
    const button = screen.getByRole("button", { name: "Annulation" });
    expect(button).toBeDisabled();
    fireEvent.click(button);
    expect(cancel).not.toHaveBeenCalled();
    expect(container.querySelector(".opa-stopped")).toBeTruthy();
  });
  it("hides cancellation where the backend refuses it", () => {
    render(<OperationProgressAction {...labels} percent={Number.NaN} cancelling={false} canCancel={false} onCancel={vi.fn()} />);
    expect(screen.queryByRole("button")).toBeNull();
    expect(screen.getByRole("progressbar")).not.toHaveAttribute("aria-valuenow");
  });
});
