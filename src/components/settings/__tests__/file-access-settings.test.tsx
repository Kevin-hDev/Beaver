import { act, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { FILE_ACCESS_HIGHLIGHT_MS, FileAccessSettings } from "../file-access-settings";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("FileAccessSettings", () => {
  it.each(["dark", "light"])("utilise la carte existante avec le thème %s", (theme) => {
    const { container } = render(
      <div data-theme={theme}>
        <FileAccessSettings
          paths={["/"]}
          focusRequested={false}
          onPathsChange={vi.fn()}
          onFocusHandled={vi.fn()}
        />
      </div>,
    );

    expect(container.querySelector(`[data-theme="${theme}"] .settings-card.fas-card`)).toBeTruthy();
  });

  it("fait défiler, met en évidence puis consomme la cible de navigation", () => {
    vi.useFakeTimers();
    const scrollIntoView = vi.fn();
    Object.defineProperty(HTMLElement.prototype, "scrollIntoView", {
      configurable: true,
      value: scrollIntoView,
    });
    const onFocusHandled = vi.fn();
    const { container } = render(
      <FileAccessSettings
        paths={["/"]}
        focusRequested
        onPathsChange={vi.fn()}
        onFocusHandled={onFocusHandled}
      />,
    );

    expect(container.querySelector(".fas-root.fas-targeted")).toBeTruthy();
    act(() => {
      vi.advanceTimersByTime(FILE_ACCESS_HIGHLIGHT_MS);
    });

    expect(scrollIntoView).toHaveBeenCalledOnce();
    expect(onFocusHandled).toHaveBeenCalledOnce();
    expect(container.querySelector(".fas-root.fas-targeted")).toBeNull();
  });
});
