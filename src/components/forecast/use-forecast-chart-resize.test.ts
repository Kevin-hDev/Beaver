import { act, renderHook } from "@testing-library/react";
import type { PointerEvent as ReactPointerEvent } from "react";
import { describe, expect, it, vi } from "vitest";
import { useForecastChartResize } from "./use-forecast-chart-resize";

describe("useForecastChartResize", () => {
  it("arrête le redimensionnement lors d'une annulation", () => {
    const { result } = renderHook(() => useForecastChartResize());
    const preventDefault = vi.fn();

    act(() => {
      result.current.startResize({
        clientY: 100,
        preventDefault,
      } as unknown as ReactPointerEvent);
      window.dispatchEvent(new MouseEvent("pointermove", { clientY: 150 }));
    });
    expect(result.current.chartHeight).toBe(450);
    expect(result.current.isResizing).toBe(true);

    act(() => {
      window.dispatchEvent(new Event("pointercancel"));
    });
    expect(result.current.isResizing).toBe(false);

    act(() => {
      window.dispatchEvent(new MouseEvent("pointermove", { clientY: 200 }));
    });
    expect(result.current.chartHeight).toBe(450);
    expect(preventDefault).toHaveBeenCalledOnce();
  });

  it("retire les écouteurs globaux au démontage", () => {
    const removeListener = vi.spyOn(window, "removeEventListener");
    const { unmount } = renderHook(() => useForecastChartResize());

    unmount();

    expect(removeListener).toHaveBeenCalledWith("pointermove", expect.any(Function));
    expect(removeListener).toHaveBeenCalledWith("pointercancel", expect.any(Function));
    expect(removeListener).toHaveBeenCalledWith("blur", expect.any(Function));
    removeListener.mockRestore();
  });
});
