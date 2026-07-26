import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ApiKeySecretInput } from "../api-key-secret-input";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => (key.endsWith("showKey") ? "Show" : "Hide"),
  }),
}));

describe("ApiKeySecretInput", () => {
  afterEach(() => cleanup());

  it("masque la cle par defaut puis l'affiche sur demande", () => {
    render(
      <ApiKeySecretInput
        value="sk-test"
        onChange={vi.fn()}
        placeholder="Paste key"
        inputClassName="wk-input"
      />,
    );

    const input = screen.getByDisplayValue("sk-test");
    expect(input.getAttribute("type")).toBe("password");

    fireEvent.click(screen.getByRole("button", { name: "Show" }));

    expect(input.getAttribute("type")).toBe("text");
    expect(screen.getByRole("button", { name: "Hide" })).toBeTruthy();
  });
});
