import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import { ErrorBoundary } from "../error-boundary";

function Crash(): never {
  throw new Error("active_view_subscription_limit_reached");
}

describe("ErrorBoundary", () => {
  it("affiche le message traduit sans exposer l'erreur interne", () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => undefined);

    render(<ErrorBoundary><Crash /></ErrorBoundary>);

    expect(screen.getByText(i18n.t("errors.crashMessage"))).toBeInTheDocument();
    expect(screen.queryByText("active_view_subscription_limit_reached")).not.toBeInTheDocument();
    error.mockRestore();
  });
});
