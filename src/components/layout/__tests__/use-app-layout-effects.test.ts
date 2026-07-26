import { renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  sidebarHiddenOffsetFromWidth,
  useSidebarHiddenOffset,
} from "../use-app-layout-effects";
import {
  projectedDetailWidthWithSidebarOpen,
  sidebarProjectionWidth,
  useAgentPanelsAutoSidebar,
} from "../agent-panels-auto-sidebar";

class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  readonly observe = vi.fn();
  readonly disconnect = vi.fn();

  constructor(private readonly callback: ResizeObserverCallback) {
    ResizeObserverMock.instances.push(this);
  }

  trigger() {
    this.callback([], this as unknown as ResizeObserver);
  }
}

const originalResizeObserver = globalThis.ResizeObserver;
const originalRequestAnimationFrame = window.requestAnimationFrame.bind(window);
const originalCancelAnimationFrame = window.cancelAnimationFrame.bind(window);

function setElementWidth(element: Element, width: number) {
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

function installLayoutDom(detailWidth: number, sidebarWidth = 0) {
  document.body.innerHTML = `
    <div class="app-sidebar-block"></div>
    <section class="app-detail-panel">
      <div class="agent-detail-with-preview">
        <aside class="asp-panel open"></aside>
        <aside class="ft-panel open"></aside>
      </div>
    </section>
  `;
  const detail = document.querySelector(".app-detail-panel")!;
  const sidebar = document.querySelector(".app-sidebar-block")!;
  setElementWidth(detail, detailWidth);
  setElementWidth(sidebar, sidebarWidth);
  return detail;
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;
  window.requestAnimationFrame = (callback: FrameRequestCallback) => {
    callback(0);
    return 1;
  };
  window.cancelAnimationFrame = () => {};
});

afterEach(() => {
  document.body.innerHTML = "";
  globalThis.ResizeObserver = originalResizeObserver;
  window.requestAnimationFrame = originalRequestAnimationFrame;
  window.cancelAnimationFrame = originalCancelAnimationFrame;
  vi.restoreAllMocks();
});

describe("projectedDetailWidthWithSidebarOpen", () => {
  it("projette la largeur detail quand la sidebar masquee serait reouverte", () => {
    expect(projectedDetailWidthWithSidebarOpen(1020, 260, false)).toBe(760);
    expect(projectedDetailWidthWithSidebarOpen(760, 260, true)).toBe(760);
  });

  it("projette avec la largeur expanded quand la sidebar est masquee", () => {
    installLayoutDom(1040, 232);
    const sidebar = document.querySelector(".app-sidebar-block") as HTMLElement;
    sidebar.style.setProperty("--sidebar-collapsed", "60px");
    sidebar.style.setProperty("--sidebar-expanded", "132px");

    const sidebarWidth = sidebarProjectionWidth(sidebar, false);

    expect(sidebarWidth).toBe(304);
    expect(projectedDetailWidthWithSidebarOpen(1040, sidebarWidth, false)).toBe(736);
  });
});

describe("useAgentPanelsAutoSidebar", () => {
  it("masque quand les panneaux agent sont trop serres", async () => {
    installLayoutDom(760);
    const onAutoHide = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(true, false, onAutoHide));

    await waitFor(() => expect(onAutoHide).toHaveBeenCalled());
  });

  it("ne rouvre pas automatiquement la sidebar apres un masquage automatique", () => {
    installLayoutDom(900);
    const onAutoHide = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(false, false, onAutoHide));
    ResizeObserverMock.instances[0]?.trigger();

    expect(onAutoHide).not.toHaveBeenCalled();
  });

  it("respecte la reouverture manuelle meme si l'espace reste serre", () => {
    installLayoutDom(760);
    const onAutoHide = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(true, true, onAutoHide));
    ResizeObserverMock.instances[0]?.trigger();

    expect(onAutoHide).not.toHaveBeenCalled();
  });

  it("garde l'info serree quand la sidebar est deja masquee", async () => {
    installLayoutDom(1020, 260);
    const onAutoHide = vi.fn();
    const onTightChange = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(false, false, onAutoHide, onTightChange));

    await waitFor(() => expect(onTightChange).toHaveBeenLastCalledWith(true));
    expect(onAutoHide).not.toHaveBeenCalled();
  });

  it("efface l'info serree quand la sidebar projetee rentre a nouveau", async () => {
    installLayoutDom(1120, 260);
    const onAutoHide = vi.fn();
    const onTightChange = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(false, false, onAutoHide, onTightChange));

    await waitFor(() => expect(onTightChange).toHaveBeenLastCalledWith(false));
    expect(onAutoHide).not.toHaveBeenCalled();
  });

  it("anticipe le hover expanded quand la sidebar est masquee", async () => {
    installLayoutDom(1040, 232);
    const sidebar = document.querySelector(".app-sidebar-block") as HTMLElement;
    sidebar.style.setProperty("--sidebar-collapsed", "60px");
    sidebar.style.setProperty("--sidebar-expanded", "132px");
    const onAutoHide = vi.fn();
    const onTightChange = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(false, false, onAutoHide, onTightChange));

    await waitFor(() => expect(onTightChange).toHaveBeenLastCalledWith(true));
    expect(onAutoHide).not.toHaveBeenCalled();
  });
});

describe("useSidebarHiddenOffset", () => {
  it("mesure la vraie largeur de la sidebar pour le masquage", async () => {
    installLayoutDom(900);
    const sidebar = document.querySelector(".app-sidebar-block")!;
    setElementWidth(sidebar, 318);

    const { result } = renderHook(() => useSidebarHiddenOffset(true));

    await waitFor(() => expect(result.current).toBe(sidebarHiddenOffsetFromWidth(318)));
  });

  it("actualise l'offset quand la largeur change pendant le masquage", async () => {
    installLayoutDom(900);
    const sidebar = document.querySelector(".app-sidebar-block")!;
    setElementWidth(sidebar, 318);
    const { result, rerender } = renderHook(
      ({ open }) => useSidebarHiddenOffset(open),
      { initialProps: { open: true } },
    );
    await waitFor(() => expect(result.current).toBe(sidebarHiddenOffsetFromWidth(318)));

    setElementWidth(sidebar, 120);
    rerender({ open: false });
    const lastObserver = ResizeObserverMock.instances[ResizeObserverMock.instances.length - 1];
    lastObserver?.trigger();

    expect(result.current).toBe(sidebarHiddenOffsetFromWidth(120));
  });
});
