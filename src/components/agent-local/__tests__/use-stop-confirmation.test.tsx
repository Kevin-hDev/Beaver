import { act, renderHook } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { STOP_CONFIRMATION_TIMEOUT_MS, useStopConfirmation } from "../use-stop-confirmation";

describe("useStopConfirmation", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.clearAllMocks();
  });

  it("demande une confirmation avant d'arrêter le stream", () => {
    const onStop = vi.fn();
    const { result } = renderHook(() => useStopConfirmation(true, onStop));

    act(() => result.current.requestStop());

    expect(result.current.isConfirmingStop).toBe(true);
    expect(onStop).not.toHaveBeenCalled();

    act(() => result.current.requestStop());

    expect(result.current.isConfirmingStop).toBe(false);
    expect(onStop).toHaveBeenCalledOnce();
  });

  it("annule la confirmation après 3 secondes", () => {
    const onStop = vi.fn();
    const { result } = renderHook(() => useStopConfirmation(true, onStop));

    act(() => result.current.requestStop());
    act(() => {
      vi.advanceTimersByTime(STOP_CONFIRMATION_TIMEOUT_MS - 1);
    });

    expect(result.current.isConfirmingStop).toBe(true);
    expect(onStop).not.toHaveBeenCalled();

    act(() => {
      vi.advanceTimersByTime(1);
    });

    expect(result.current.isConfirmingStop).toBe(false);
  });

  it("masque la confirmation quand le stream se termine", () => {
    const onStop = vi.fn();
    const { result, rerender } = renderHook(
      ({ isStreaming }) => useStopConfirmation(isStreaming, onStop),
      { initialProps: { isStreaming: true } },
    );

    act(() => result.current.requestStop());

    expect(result.current.isConfirmingStop).toBe(true);

    rerender({ isStreaming: false });

    expect(result.current.isConfirmingStop).toBe(false);
  });
});
