import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useRef } from "react";
import type { BrowserTabState } from "../browser-types";
import { useBrowserTabReorder } from "../use-browser-tab-reorder";

const TAB_ONE = "11111111111111111111111111111111";
const TAB_TWO = "22222222222222222222222222222222";
const TAB_THREE = "33333333333333333333333333333333";
const OTHER_ONE = "44444444444444444444444444444444";
const OTHER_TWO = "55555555555555555555555555555555";
const OTHER_THREE = "66666666666666666666666666666666";
const originalScrollIntoView = Object.getOwnPropertyDescriptor(Element.prototype, "scrollIntoView");
const scrollIntoView = vi.fn();

interface HarnessProps {
  tabs: BrowserTabState[];
  conversationId?: string;
  enabled?: boolean;
  onReorder: (ids: string[]) => Promise<boolean>;
  onReorderError: () => void;
}

function makeTabs(ids = [TAB_ONE, TAB_TWO, TAB_THREE], suffix = ""): BrowserTabState[] {
  return ids.map((id, index) => ({
    id,
    title: `${String.fromCharCode(65 + index)}${suffix}`,
    url: `https://${index}.example/`,
    loading: false,
    canGoBack: false,
    canGoForward: false,
    released: false,
  }));
}

function ReorderHarness({
  tabs, conversationId = "conversation-a", enabled = true, onReorder, onReorderError,
}: HarnessProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const reorder = useBrowserTabReorder({
    tabs,
    activeTabId: tabs[0]?.id ?? "",
    conversationId,
    enabled,
    onReorder,
    onReorderError,
    containerRef,
  });
  const tabsById = new Map(tabs.map((tab) => [tab.id, tab]));
  return (
    <div ref={containerRef} data-pending={String(reorder.pending)}>
      {reorder.drag.order.map((id) => {
        const tab = tabsById.get(id);
        if (!tab) return null;
        return (
          <button
            key={id}
            data-testid={`tab-${id}`}
            data-browser-tab-id={id}
            {...reorder.drag.itemProps(id)}
            onPointerDown={(event) => reorder.handlePointerDown(id, event)}
            onKeyDown={(event) => reorder.handleKeyDown(id, event)}
          >
            {tab.title}
          </button>
        );
      })}
    </div>
  );
}

function deferred<T>() {
  let resolve: (value: T) => void;
  let reject: (reason?: unknown) => void;
  const promise = new Promise<T>((nextResolve, nextReject) => {
    resolve = nextResolve;
    reject = nextReject;
  });
  return { promise, resolve: resolve!, reject: reject! };
}

function moveFirstToLast(id = TAB_ONE) {
  const tab = screen.getByTestId(`tab-${id}`);
  fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
  fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
  fireEvent.pointerUp(window);
}

beforeEach(() => {
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(function (
    this: HTMLElement,
  ) {
    const ids = [TAB_ONE, TAB_TWO, TAB_THREE, OTHER_ONE, OTHER_TWO, OTHER_THREE];
    const index = ids.indexOf(this.getAttribute("data-drag-id") ?? "") % 3;
    if (index >= 0) return { left: index * 100, top: 0, width: 100, height: 32 } as DOMRect;
    return { left: 0, top: 0, width: 300, height: 32 } as DOMRect;
  });
  Object.defineProperty(Element.prototype, "scrollIntoView", { configurable: true, value: scrollIntoView });
});

afterEach(() => {
  if (originalScrollIntoView) Object.defineProperty(Element.prototype, "scrollIntoView", originalScrollIntoView);
  else Reflect.deleteProperty(Element.prototype, "scrollIntoView");
  vi.restoreAllMocks();
  scrollIntoView.mockReset();
  cleanup();
});

describe("useBrowserTabReorder", () => {
  it("réinitialise après un refus et accepte une nouvelle tentative", async () => {
    const onReorder = vi.fn().mockResolvedValueOnce(false).mockResolvedValueOnce(true);
    render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={vi.fn()} />);

    moveFirstToLast();
    await waitFor(() => expect(screen.getAllByRole("button").map((tab) => tab.getAttribute("data-drag-id")))
      .toEqual([TAB_ONE, TAB_TWO, TAB_THREE]));
    moveFirstToLast();
    await waitFor(() => expect(onReorder).toHaveBeenCalledTimes(2));
  });

  it("bloque un second départ jusqu'à la résolution IPC, même si l'ordre arrive avant", async () => {
    const first = deferred<boolean>();
    const onReorder = vi.fn(() => first.promise);
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={vi.fn()} />);

    moveFirstToLast();
    view.rerender(<ReorderHarness tabs={makeTabs([TAB_TWO, TAB_THREE, TAB_ONE])} onReorder={onReorder} onReorderError={vi.fn()} />);
    moveFirstToLast(TAB_TWO);
    expect(onReorder).toHaveBeenCalledOnce();

    first.resolve(true);
    await waitFor(() => expect(screen.getByTestId(`tab-${TAB_ONE}`).parentElement).toHaveAttribute("data-pending", "false"));
  });

  it("attend la sauvegarde en cours même si un onglet est fermé", async () => {
    const first = deferred<boolean>();
    const onReorder = vi.fn().mockImplementationOnce(() => first.promise).mockResolvedValue(true);
    const onReorderError = vi.fn();
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={onReorderError} />);
    moveFirstToLast();
    view.rerender(<ReorderHarness tabs={makeTabs([TAB_TWO, TAB_THREE])} onReorder={onReorder} onReorderError={onReorderError} />);
    const remaining = screen.getByTestId(`tab-${TAB_TWO}`);
    fireEvent.keyDown(remaining, { key: "ArrowRight", altKey: true, shiftKey: true });
    expect(onReorder).toHaveBeenCalledOnce();
    expect(remaining.parentElement).toHaveAttribute("data-pending", "true");
    first.resolve(false);
    await waitFor(() => expect(remaining.parentElement).toHaveAttribute("data-pending", "false"));
    expect(screen.queryByTestId(`tab-${TAB_ONE}`)).toBeNull();
    fireEvent.keyDown(remaining, { key: "ArrowRight", altKey: true, shiftKey: true });
    await waitFor(() => expect(onReorder).toHaveBeenCalledTimes(2));
  });

  it("garde la demande B et son ordre local malgré l'échec tardif de A", async () => {
    const first = deferred<boolean>();
    const second = deferred<boolean>();
    const onReorder = vi.fn().mockImplementationOnce(() => first.promise).mockImplementationOnce(() => second.promise);
    const onReorderError = vi.fn();
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={onReorderError} />);

    moveFirstToLast();
    view.rerender(<ReorderHarness tabs={makeTabs([OTHER_ONE, OTHER_TWO, OTHER_THREE])} conversationId="conversation-b" onReorder={onReorder} onReorderError={onReorderError} />);
    moveFirstToLast(OTHER_ONE);
    first.reject(new Error("réponse A tardive"));

    await waitFor(() => expect(screen.getAllByRole("button").map((tab) => tab.getAttribute("data-drag-id")))
      .toEqual([OTHER_TWO, OTHER_THREE, OTHER_ONE]));
    expect(screen.getByTestId(`tab-${OTHER_ONE}`).parentElement).toHaveAttribute("data-pending", "true");
    expect(onReorderError).not.toHaveBeenCalled();
    second.resolve(false);
    await waitFor(() => expect(screen.getAllByRole("button").map((tab) => tab.getAttribute("data-drag-id")))
      .toEqual([OTHER_ONE, OTHER_TWO, OTHER_THREE]));
  });

  it("annule un geste à la désactivation ou à la fermeture, sans boucle inactive", () => {
    const onReorder = vi.fn().mockResolvedValue(true);
    const idle = render(<ReorderHarness tabs={makeTabs()} enabled={false} onReorder={onReorder} onReorderError={vi.fn()} />);
    expect(idle.getByTestId(`tab-${TAB_ONE}`)).toBeTruthy();
    idle.unmount();
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={vi.fn()} />);
    const tab = screen.getByTestId(`tab-${TAB_ONE}`);

    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    view.rerender(<ReorderHarness tabs={makeTabs()} enabled={false} onReorder={onReorder} onReorderError={vi.fn()} />);
    fireEvent.pointerUp(window);
    view.rerender(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={vi.fn()} />);
    fireEvent.pointerDown(screen.getByTestId(`tab-${TAB_ONE}`), { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    view.rerender(<ReorderHarness tabs={makeTabs().slice(1)} onReorder={onReorder} onReorderError={vi.fn()} />);
    fireEvent.pointerUp(window);
    expect(onReorder).not.toHaveBeenCalled();
  });

  it("garde le geste si seul le titre reçu change", async () => {
    const onReorder = vi.fn().mockResolvedValue(true);
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={onReorder} onReorderError={vi.fn()} />);
    const tab = screen.getByTestId(`tab-${TAB_ONE}`);

    fireEvent.pointerDown(tab, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.pointerMove(window, { clientX: 250, clientY: 10 });
    view.rerender(<ReorderHarness tabs={makeTabs(undefined, " mis à jour")} onReorder={onReorder} onReorderError={vi.fn()} />);
    fireEvent.pointerUp(window);

    await waitFor(() => expect(onReorder).toHaveBeenCalledWith([TAB_TWO, TAB_THREE, TAB_ONE]));
    await waitFor(() => expect(screen.getByTestId(`tab-${TAB_ONE}`).parentElement).toHaveAttribute("data-pending", "false"));
  });

  it("signale une erreur et déplace au clavier avec focus après l'ordre reçu", async () => {
    const onReorderError = vi.fn();
    const rejected = vi.fn().mockRejectedValue(new Error("échec"));
    const view = render(<ReorderHarness tabs={makeTabs()} onReorder={rejected} onReorderError={onReorderError} />);
    moveFirstToLast();
    await waitFor(() => expect(onReorderError).toHaveBeenCalledOnce());

    const accepted = vi.fn().mockResolvedValue(true);
    view.rerender(<ReorderHarness tabs={makeTabs()} onReorder={accepted} onReorderError={onReorderError} />);
    const tab = screen.getByTestId(`tab-${TAB_TWO}`);
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowLeft", altKey: true, shiftKey: true });
    await waitFor(() => expect(accepted).toHaveBeenCalledWith([TAB_TWO, TAB_ONE, TAB_THREE]));
    view.rerender(<ReorderHarness tabs={makeTabs([TAB_TWO, TAB_ONE, TAB_THREE])} onReorder={accepted} onReorderError={onReorderError} />);

    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId(`tab-${TAB_TWO}`)));
    expect(scrollIntoView).toHaveBeenCalledWith({ block: "nearest", inline: "nearest" });
  });
});
