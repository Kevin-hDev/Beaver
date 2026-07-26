import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ErrorBubble } from "../error-bubble";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

afterEach(cleanup);

describe("ErrorBubble", () => {
  it("affiche le bouton Réessayer pour une erreur finale non connexion", () => {
    const onRetry = vi.fn();
    render(<ErrorBubble message="Erreur" onRetry={onRetry} />);

    fireEvent.click(screen.getByRole("button", { name: "agentLocal.retry.button" }));
    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  it("masque le bouton pour une erreur de connexion", () => {
    render(<ErrorBubble message="Connexion perdue" isConnection onRetry={vi.fn()} />);

    expect(screen.queryByRole("button")).toBeNull();
  });
});
