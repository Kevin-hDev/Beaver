import { afterEach, describe, expect, it } from "vitest";
import {
  chatMinWidthForContainer,
  findPanelForResizeHandle,
  measurePreviewLayout,
  siblingPanelWidth,
} from "../file-preview-layout";

function setWidth(element: Element, width: number) {
  element.getBoundingClientRect = () => ({
    width,
    height: 0,
    top: 0,
    right: width,
    bottom: 0,
    left: 0,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  });
}

describe("file preview layout", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("compte les panneaux voisins mais pas le chat ni le panel courant", () => {
    document.body.innerHTML = `
      <div class="agent-detail-with-preview">
        <aside class="project-sidebar"></aside>
        <main class="agent-detail-chat"></main>
        <section class="asp-panel open"></section>
      </div>
    `;
    const container = document.querySelector(".agent-detail-with-preview")!;
    const sidebar = document.querySelector(".project-sidebar")!;
    const panel = document.querySelector(".asp-panel")!;
    setWidth(sidebar, 220);
    setWidth(panel, 500);

    expect(siblingPanelWidth(panel, container)).toBe(220);
  });

  it("mesure la place reservee autour du panel", () => {
    document.body.innerHTML = `
      <div class="agent-detail-with-preview">
        <aside class="project-sidebar"></aside>
        <main class="agent-detail-chat"></main>
        <section class="asp-panel open"></section>
      </div>
    `;
    const container = document.querySelector(".agent-detail-with-preview")!;
    const sidebar = document.querySelector(".project-sidebar")!;
    const panel = document.querySelector(".asp-panel")!;
    setWidth(container, 1200);
    setWidth(sidebar, 240);

    expect(measurePreviewLayout(panel, 320)).toMatchObject({
      container,
      containerWidth: 1200,
      reservedWidth: 560,
    });
  });

  it("lit le minimum de chat compact depuis le conteneur", () => {
    document.body.innerHTML = `
      <div class="agent-detail-with-preview" style="--agent-chat-min-width: 0px">
        <main class="agent-detail-chat"></main>
        <section class="asp-panel open"></section>
      </div>
    `;
    const container = document.querySelector(".agent-detail-with-preview")!;

    expect(chatMinWidthForContainer(container)).toBe(0);
  });

  it("retrouve le panneau placé juste après la poignée extérieure", () => {
    document.body.innerHTML = `
      <div class="agent-detail-with-preview">
        <main class="agent-detail-chat"></main>
        <div class="asp-resize-slot"><div class="asp-resize"></div></div>
        <aside class="asp-panel open"></aside>
      </div>
    `;
    const handle = document.querySelector<HTMLElement>(".asp-resize")!;
    const panel = document.querySelector(".asp-panel");

    expect(findPanelForResizeHandle(handle)).toBe(panel);
  });
});
