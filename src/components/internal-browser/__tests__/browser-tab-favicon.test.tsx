import { fireEvent, render } from "@testing-library/react";
import { expect, it } from "vitest";
import { BrowserTabFavicon } from "../browser-tab-favicon";

it("keeps the globe for missing or failed icons and retries a replacement", () => {
  const { container, rerender } = render(<BrowserTabFavicon />);
  const globe = container.querySelector("img")!.getAttribute("src");
  const png = "iVBORw0KGgo=";
  rerender(<BrowserTabFavicon pngBase64={png} />);
  const image = container.querySelector("img")!;
  expect(image.src).toBe(`data:image/png;base64,${png}`);
  expect(image.draggable).toBe(false);
  expect(image.alt).toBe("");
  expect(image.getAttribute("aria-hidden")).toBe("true");
  fireEvent.error(image);
  expect(container.querySelector("img")!.getAttribute("src")).toBe(globe);
  rerender(<BrowserTabFavicon pngBase64="iVBORw0KGgoAAA==" />);
  expect(container.querySelector("img")!.src).toContain("data:image/png;base64,");
  rerender(<BrowserTabFavicon />);
  rerender(<BrowserTabFavicon pngBase64={png} />);
  expect(container.querySelector("img")!.src).toBe(`data:image/png;base64,${png}`);
});
