/* @vitest-environment jsdom */
import { afterEach, describe, expect, it, vi } from "vitest";
import { attachBrowserTransitionGuards } from "../browser-surface-transitions";

function transition(target: Element, type: string, propertyName: string) {
  const event = new Event(type, { bubbles: true });
  Object.defineProperty(event, "propertyName", { value: propertyName });
  target.dispatchEvent(event);
}

function layout() {
  document.body.innerHTML = `
    <div class="app-root">
      <div class="app-sidebar-block">
        <nav data-nav-zone="sidebar"></nav>
        <div class="app-list-panel"></div>
      </div>
      <div class="app-detail-panel">
        <div class="agent-detail-with-preview">
          <main class="agent-detail-chat"></main>
          <aside class="asp-panel">
            <div class="asp-slide-wrapper">
              <div class="ib-surface"></div>
            </div>
          </aside>
          <aside class="ft-panel"></aside>
        </div>
      </div>
    </div>
  `;
  return {
    root: document.querySelector<HTMLElement>(".app-root")!,
    host: document.querySelector<HTMLDivElement>(".ib-surface")!,
    panel: document.querySelector<HTMLElement>(".asp-panel")!,
    sidebar: document.querySelector<HTMLElement>(".app-sidebar-block")!,
    sidebarNav: document.querySelector<HTMLElement>('[data-nav-zone="sidebar"]')!,
    listPanel: document.querySelector<HTMLElement>(".app-list-panel")!,
  };
}

describe("browser surface transition guards", () => {
  afterEach(() => {
    document.body.innerHTML = "";
  });

  it("reste masqué jusqu'à la fin de toutes les transitions du panneau", () => {
    const { host, panel } = layout();
    const onBegin = vi.fn();
    const onEnd = vi.fn();
    const guards = attachBrowserTransitionGuards(host, onBegin, onEnd);

    transition(panel, "transitionrun", "transform");
    transition(panel, "transitionrun", "width");
    expect(onBegin).toHaveBeenCalledTimes(1);
    expect(guards.moving()).toBe(true);

    transition(panel, "transitionend", "transform");
    expect(onEnd).not.toHaveBeenCalled();
    expect(guards.moving()).toBe(true);

    transition(panel, "transitionend", "width");
    expect(onEnd).toHaveBeenCalledTimes(1);
    expect(guards.moving()).toBe(false);
    guards.detach();
  });

  it("masque la vue pendant le déplacement provoqué par la sidebar", () => {
    const { host, sidebar } = layout();
    const onBegin = vi.fn();
    const onEnd = vi.fn();
    const guards = attachBrowserTransitionGuards(host, onBegin, onEnd);

    transition(sidebar, "transitionrun", "margin-left");
    expect(onBegin).toHaveBeenCalledTimes(1);
    expect(guards.moving()).toBe(true);

    transition(sidebar, "transitionend", "margin-left");
    expect(onEnd).toHaveBeenCalledTimes(1);
    expect(guards.moving()).toBe(false);
    guards.detach();
  });

  it("resynchronise un changement de layout sans animation", async () => {
    const { root, host } = layout();
    const onEnd = vi.fn();
    const guards = attachBrowserTransitionGuards(host, vi.fn(), onEnd);

    root.classList.add("sidebar-hidden");
    await Promise.resolve();

    expect(onEnd).toHaveBeenCalledTimes(1);
    guards.detach();
  });

  it("resynchronise les changements de largeur internes à la sidebar", async () => {
    const { host, sidebarNav, listPanel } = layout();
    const onEnd = vi.fn();
    const guards = attachBrowserTransitionGuards(host, vi.fn(), onEnd);

    sidebarNav.style.width = "120px";
    listPanel.style.width = "320px";
    await Promise.resolve();

    expect(onEnd).toHaveBeenCalledTimes(1);
    guards.detach();
  });
});
