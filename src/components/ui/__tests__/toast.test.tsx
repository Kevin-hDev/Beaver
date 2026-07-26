import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { InlineToast } from "../toast";

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
