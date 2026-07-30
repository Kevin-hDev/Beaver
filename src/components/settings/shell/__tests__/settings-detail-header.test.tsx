import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SettingsDetailHeader } from "../settings-detail-header";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

describe("SettingsDetailHeader", () => {
  afterEach(() => cleanup());

  it("ramène à la liste depuis le bouton retour", () => {
    const onBack = vi.fn();
    render(<SettingsDetailHeader title="Groq" onBack={onBack} />);

    fireEvent.click(screen.getByLabelText("common.back"));

    expect(onBack).toHaveBeenCalledTimes(1);
  });

  it("affiche le sous-titre et les actions fournies", () => {
    render(
      <SettingsDetailHeader
        title="Groq"
        subtitle="Inférence rapide"
        actions={<button type="button">Supprimer</button>}
        onBack={vi.fn()}
      />,
    );

    expect(screen.getByText("Inférence rapide")).toBeTruthy();
    expect(screen.getByText("Supprimer")).toBeTruthy();
  });

  it("omet le sous-titre quand il n'y en a pas", () => {
    const { container } = render(<SettingsDetailHeader title="Groq" onBack={vi.fn()} />);

    expect(container.querySelector(".settings-detail-title p")).toBeNull();
  });
});
