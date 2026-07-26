import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useFloatingMenuPosition } from "./use-floating-menu-position";

function FloatingFixture() {
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(true, "right", 6, "auto");
  return (
    <>
      <button ref={(node) => { anchorRef.current = node; }} data-anchor>
        anchor
      </button>
      <div ref={floatingRef} style={floatingStyle}>menu</div>
    </>
  );
}

function MatchingWidthFixture() {
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(true, "left", 4, "below", true);
  return (
    <>
      <button ref={(node) => { anchorRef.current = node; }} data-anchor>
        anchor
      </button>
      <div ref={floatingRef} style={floatingStyle}>matching menu</div>
    </>
  );
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("useFloatingMenuPosition", () => {
  it("keeps a large menu inside a small webview", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect")
      .mockImplementation(function getRect(this: HTMLElement) {
        if (this.dataset.anchor !== undefined) {
          return {
            x: 250, y: 8, top: 8, right: 280, bottom: 36, left: 250,
            width: 30, height: 28, toJSON: () => ({}),
          };
        }
        return {
          x: 0, y: 0, top: 0, right: 340, bottom: 420, left: 0,
          width: 340, height: 420, toJSON: () => ({}),
        };
      });
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(340);
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(420);
    vi.stubGlobal("innerWidth", 300);
    vi.stubGlobal("innerHeight", 240);

    render(<FloatingFixture />);

    await waitFor(() => expect(screen.getByText("menu")).toHaveStyle({
      top: "42px",
      left: "12px",
      maxWidth: "276px",
      maxHeight: "186px",
      visibility: "visible",
    }));
  });

  it("can keep a selector menu at least as wide as its trigger", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect")
      .mockReturnValue({
        x: 20, y: 20, top: 20, right: 180, bottom: 48, left: 20,
        width: 160, height: 28, toJSON: () => ({}),
      });
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(80);
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(100);
    vi.stubGlobal("innerWidth", 600);
    vi.stubGlobal("innerHeight", 400);

    render(<MatchingWidthFixture />);

    await waitFor(() => expect(screen.getByText("matching menu")).toHaveStyle({
      left: "20px",
      minWidth: "160px",
      visibility: "visible",
    }));
  });

  it("anchors an above menu by its bottom edge so expansion cannot move it", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect")
      .mockReturnValue({
        x: 100, y: 300, top: 300, right: 180, bottom: 328, left: 100,
        width: 80, height: 28, toJSON: () => ({}),
      });
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(160);
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(60);
    vi.stubGlobal("innerWidth", 600);
    vi.stubGlobal("innerHeight", 500);

    render(<FloatingFixture />);

    await waitFor(() => expect(screen.getByText("menu")).toHaveStyle({
      bottom: "206px",
      top: "auto",
      maxHeight: "282px",
    }));
  });
});
