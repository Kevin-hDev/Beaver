import { act, fireEvent, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { describe, expect, it, vi } from "vitest";
import { InlineToast, ToastProvider, useToast } from "../toast";

describe("InlineToast", () => {
  it("utilise la variante d'erreur compacte et accessible", () => {
    render(
      <InlineToast type="error" compact className="test-class">
        Échec
      </InlineToast>,
    );

    expect(screen.getByRole("alert")).toHaveClass(
      "toast",
      "toast-inline",
      "toast-error",
      "toast-inline-compact",
      "test-class",
    );
  });

  it("conserve le bandeau d'avertissement existant", () => {
    render(<InlineToast type="warning">Avertissement</InlineToast>);

    expect(screen.getByRole("status")).toHaveClass(
      "toast",
      "toast-inline",
      "toast-warning",
    );
  });
});

function ActionToast({ action }: { action: () => void }) {
  const { show } = useToast();
  useEffect(() => {
    show("Navigateur indisponible", "error", 10_000, {
      action: { label: "Redémarrer", onClick: action },
      dismissLabel: "Fermer",
    });
  }, [action, show]);
  return null;
}

describe("ToastProvider actions", () => {
  it("reprend le toast existant, reste fermable et retire aussi l'action à 10 secondes", () => {
    vi.useFakeTimers();
    const action = vi.fn();
    render(
      <ToastProvider>
        <ActionToast action={action} />
      </ToastProvider>,
    );

    expect(screen.getByRole("alert")).toHaveClass("toast", "toast-error");
    fireEvent.click(screen.getByRole("button", { name: "Redémarrer" }));
    expect(action).toHaveBeenCalledOnce();
    expect(screen.queryByText("Navigateur indisponible")).not.toBeInTheDocument();

    render(
      <ToastProvider>
        <ActionToast action={() => {}} />
      </ToastProvider>,
    );
    expect(screen.getByRole("button", { name: "Fermer" })).toBeEnabled();
    void act(() => vi.advanceTimersByTime(10_000));
    expect(screen.queryByText("Navigateur indisponible")).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Redémarrer" })).not.toBeInTheDocument();
    vi.useRealTimers();
  });
});
