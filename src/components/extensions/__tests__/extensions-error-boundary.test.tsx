import { useState } from "react";
import type { ReactNode } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ExtensionsErrorBoundary } from "../extensions-error-boundary";

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key },
}));

function BrokenView(): ReactNode {
  throw new Error("private extension failure");
}

function BoundaryHarness() {
  const [broken, setBroken] = useState(true);
  return (
    <ExtensionsErrorBoundary
      resetKey={broken ? "broken" : "safe"}
      onReset={() => setBroken(false)}
    >
      {broken ? <BrokenView /> : <p>safe view</p>}
    </ExtensionsErrorBoundary>
  );
}

afterEach(() => {
  vi.restoreAllMocks();
});

describe("ExtensionsErrorBoundary", () => {
  it("garde le crash dans l’onglet et permet de revenir à une vue sûre", () => {
    vi.spyOn(console, "error").mockImplementation(() => undefined);
    render(<BoundaryHarness />);

    expect(screen.getByRole("alert"))
      .toHaveTextContent("extensions.errors.view");
    fireEvent.click(screen.getByRole("button", {
      name: "extensions.actions.back",
    }));

    expect(screen.getByText("safe view")).toBeInTheDocument();
  });
});
