import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useChatScroll } from "../use-chat-scroll";

describe("useChatScroll", () => {
  it("initialise avec isAtBottom = true", () => {
    const { result } = renderHook(() => useChatScroll("s1", false, []));
    expect(result.current.isAtBottom).toBe(true);
  });

  it("scrollToBottom remet isAtBottom à true", () => {
    const { result } = renderHook(() => useChatScroll("s1", false, []));
    act(() => {
      result.current.scrollToBottom();
    });
    expect(result.current.isAtBottom).toBe(true);
  });

  it("scrollToBottom ne jette pas si déjà en bas", () => {
    const { result } = renderHook(() => useChatScroll("s1", false, []));
    expect(() => {
      act(() => {
        result.current.scrollToBottom();
      });
    }).not.toThrow();
  });
});
