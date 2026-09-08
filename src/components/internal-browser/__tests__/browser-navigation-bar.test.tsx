import { useState } from "react";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { BrowserTabState } from "../browser-types";
import { BrowserNavigationBar } from "../browser-navigation-bar";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => ({
      "browser.back": "Retour",
      "browser.forward": "Avancer",
      "browser.reload": "Recharger",
      "browser.stop": "Arrêter",
      "browser.addressLabel": "Adresse du navigateur",
      "browser.addressPlaceholder": "Saisir une URL",
      "browser.openAddress": "Ouvrir l’adresse",
      "filePreview.fullscreen": "Plein écran",
      "filePreview.reduce": "Réduire",
    }[key] ?? key),
  }),
}));

const tab: BrowserTabState = {
  id: "11111111111111111111111111111111",
  title: "A",
  url: "https://initial.example/",
  loading: false,
  canGoBack: false,
  canGoForward: false,
  released: false,
};

interface HarnessProps {
  address: string;
  onFocus?: () => void;
  onBlur?: () => void;
  onChange?: (value: string) => void;
  onSubmit?: () => void;
}

function NavigationHarness({
  address,
  onFocus = vi.fn(),
  onBlur = vi.fn(),
  onChange = vi.fn(),
  onSubmit = vi.fn(),
}: HarnessProps) {
  const [value, setValue] = useState(address);
  return (
    <BrowserNavigationBar
      tab={tab}
      address={value}
      invalid={false}
      fullscreen={false}
      onAddressFocus={onFocus}
      onAddressBlur={onBlur}
      onAddressChange={(next) => {
        onChange(next);
        setValue(next);
      }}
      onSubmit={onSubmit}
      onAction={vi.fn()}
      onFullscreenChange={vi.fn()}
    />
  );
}

function input(): HTMLInputElement {
  return screen.getByRole("textbox");
}

describe("BrowserNavigationBar", () => {
  it("sélectionne l'adresse au premier clic complet, pas au second", async () => {
    const user = userEvent.setup();
    render(<NavigationHarness address="https://initial.example/" />);

    await user.click(input());
    expect(input().selectionStart).toBe(0);
    expect(input().selectionEnd).toBe(input().value.length);
    input().setSelectionRange(5, 5);
    await user.click(input());
    expect(input().selectionStart).toBe(input().selectionEnd);
  });

  it("sélectionne au Tab, après un blur et pour une adresse vide", async () => {
    const user = userEvent.setup();
    render(
      <>
        <button type="button">Avant</button>
        <NavigationHarness address="https://initial.example/" />
      </>,
    );
    screen.getByRole("button", { name: "Avant" }).focus();

    await user.tab();
    await user.tab();
    expect(input()).toHaveFocus();
    expect(input().selectionStart).toBe(0);
    expect(input().selectionEnd).toBe(input().value.length);
    await user.click(screen.getByRole("button", { name: "Recharger" }));
    await user.click(input());
    expect(input().selectionEnd).toBe(input().value.length);

    render(<NavigationHarness address="" />);
    const empty = screen.getAllByRole<HTMLInputElement>("textbox")[1];
    await user.click(empty);
    expect(empty.selectionStart).toBe(0);
    expect(empty.selectionEnd).toBe(0);
  });

  it("remplace la sélection et soumet avec Entrée ou le bouton", async () => {
    const user = userEvent.setup();
    const onSubmit = vi.fn();
    render(<NavigationHarness address="https://initial.example/" onSubmit={onSubmit} />);

    await user.click(input());
    await user.keyboard("https://next.example/");
    expect(input()).toHaveValue("https://next.example/");
    await user.keyboard("{Enter}");
    await user.click(screen.getByRole("button", { name: "Ouvrir l’adresse" }));
    expect(onSubmit).toHaveBeenCalledTimes(2);
  });

  it("laisse un geste de sélection partielle remplacer seulement la sous-chaîne", async () => {
    const user = userEvent.setup();
    const onFocus = vi.fn();
    const onBlur = vi.fn();
    const onChange = vi.fn();
    render(<NavigationHarness address="abcdef" onFocus={onFocus} onBlur={onBlur} onChange={onChange} />);

    await user.click(input());
    await user.pointer([
      { keys: "[MouseLeft>]", target: input(), offset: 1 },
      { target: input(), offset: 4 },
      "[/MouseLeft]",
    ]);
    expect(input().selectionStart).toBe(1);
    expect(input().selectionEnd).toBe(4);
    await user.keyboard("X");
    expect(input()).toHaveValue("aXef");
    fireEvent.blur(input());
    expect(onFocus).toHaveBeenCalledOnce();
    expect(onBlur).toHaveBeenCalledOnce();
    expect(onChange).toHaveBeenCalled();
  });

  it("resélectionne au retour de CEF malgré un activeElement resté sur l'adresse", () => {
    render(<NavigationHarness address="https://initial.example/" />);
    const address = input();
    address.focus();
    address.setSelectionRange(address.value.length, address.value.length);
    const hasFocus = vi.spyOn(document, "hasFocus").mockReturnValue(false);

    fireEvent.pointerDown(address, { button: 0 });
    hasFocus.mockReturnValue(true);
    fireEvent.focus(window);
    fireEvent.focus(address);
    address.setSelectionRange(address.value.length, address.value.length);
    fireEvent.click(address);

    expect(address.selectionStart).toBe(0);
    expect(address.selectionEnd).toBe(address.value.length);
    address.setSelectionRange(5, 5);
    fireEvent.pointerDown(address, { button: 0 });
    fireEvent.click(address);
    expect(address.selectionStart).toBe(5);
    expect(address.selectionEnd).toBe(5);
    hasFocus.mockRestore();
  });

  it("oublie le premier clic interrompu par pointercancel", () => {
    render(<NavigationHarness address="https://initial.example/" />);
    const address = input();
    screen.getByRole("button", { name: "Recharger" }).focus();

    fireEvent.pointerDown(address, { button: 0 });
    fireEvent.pointerCancel(address);
    address.focus();
    address.setSelectionRange(4, 4);
    fireEvent.click(address);

    expect(address.selectionStart).toBe(address.selectionEnd);
  });
});
