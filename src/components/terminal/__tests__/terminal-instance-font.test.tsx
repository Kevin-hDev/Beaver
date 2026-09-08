import { act, render } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { TerminalInstance } from "../terminal-instance";

const STACK = "'Beaver Mono', ui-monospace, monospace";
const doubles = vi.hoisted(() => ({
  options: { fontFamily: "" },
  fit: vi.fn(),
  dispose: vi.fn(),
  start: vi.fn(),
}));

vi.mock("../terminal-theme", () => ({
  readTerminalFont: () => "'Beaver Mono', ui-monospace, monospace",
}));
vi.mock("../terminal-pty-bridge", () => ({
  createTerminalPtyBridge: () => ({ dispose() {}, resize() {}, start: doubles.start }),
}));
vi.mock("@xterm/xterm", () => ({
  Terminal: class {
    cols = 80;
    rows = 24;
    options: { fontFamily: string };
    constructor(options: { fontFamily: string }) {
      this.options = options;
      doubles.options = options;
    }
    attachCustomKeyEventHandler() {}
    dispose = doubles.dispose;
    focus() {}
    loadAddon() {}
    onResize() { return { dispose() {} }; }
    open() {}
  },
}));
vi.mock("@xterm/addon-fit", () => ({ FitAddon: class { fit = doubles.fit; } }));
vi.mock("@xterm/xterm/css/xterm.css", () => ({}));

let completeFont: () => void;
const originalFonts = Object.getOwnPropertyDescriptor(document, "fonts");

beforeEach(() => {
  vi.clearAllMocks();
  const loaded = new Promise<void>((resolve) => { completeFont = resolve; });
  Object.defineProperty(document, "fonts", {
    configurable: true,
    value: { check: () => false, load: () => loaded },
  });
  vi.stubGlobal("ResizeObserver", class { disconnect() {} observe() {} });
});

afterEach(() => {
  if (originalFonts) Object.defineProperty(document, "fonts", originalFonts);
  else Reflect.deleteProperty(document, "fonts");
  vi.unstubAllGlobals();
});

function mountTerminal() {
  const view = render(
    <TerminalInstance
      tabId="font-tab" groupKey="project" theme={{}} isVisible={false}
      onActivity={vi.fn()} onExit={vi.fn()} onPtyReady={vi.fn()} onTogglePanel={vi.fn()}
    />,
  );
  const host = view.container.firstElementChild!;
  Object.defineProperties(host, {
    offsetWidth: { configurable: true, value: 800 },
    offsetHeight: { configurable: true, value: 300 },
  });
  return { ...view, host };
}

it("applique la police chargée et recalcule la taille sans retarder le shell", async () => {
  mountTerminal();
  expect(doubles.options.fontFamily).toBe("ui-monospace, monospace");
  expect(doubles.start).toHaveBeenCalledOnce();
  doubles.fit.mockClear();

  await act(async () => { completeFont(); await Promise.resolve(); });

  expect(doubles.options.fontFamily).toBe(STACK);
  expect(doubles.fit).toHaveBeenCalledOnce();
});

it("ne mesure pas un panneau de taille nulle quand la police arrive", async () => {
  const { host } = mountTerminal();
  Object.defineProperty(host, "offsetHeight", { value: 0 });
  doubles.fit.mockClear();

  await act(async () => { completeFont(); await Promise.resolve(); });

  expect(doubles.options.fontFamily).toBe(STACK);
  expect(doubles.fit).not.toHaveBeenCalled();
});

it("ne touche plus au terminal démonté quand la police arrive", async () => {
  const view = mountTerminal();
  view.unmount();
  expect(doubles.dispose).toHaveBeenCalledOnce();
  doubles.fit.mockClear();

  await act(async () => { completeFont(); await Promise.resolve(); });

  expect(doubles.options.fontFamily).toBe("ui-monospace, monospace");
  expect(doubles.fit).not.toHaveBeenCalled();
});
