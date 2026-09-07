/* @vitest-environment jsdom */
import { createElement } from "react";
import { act, fireEvent, renderHook } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import type { AgentMessage } from "@/types/agent";
import { useConversationSearch } from "../use-conversation-search";

const messages = [{
  id: "message-1",
  role: "user",
  content: "trouver ceci",
  files: [],
  timestamp: "2026-09-07T00:00:00Z",
}] as unknown as AgentMessage[];

afterEach(() => {
  vi.restoreAllMocks();
});

describe("useConversationSearch et l'activité de surface", () => {
  it("n'ouvre pas la recherche inactive et reprend au retour", () => {
    let active = true;
    const wrapper = ({ children }: { children: React.ReactNode }) =>
      createElement(AppSurfaceActivityProvider, { active } as never, children);
    const { result, rerender } = renderHook(() => useConversationSearch(messages), { wrapper });

    fireEvent.keyDown(window, { code: "KeyF", key: "f", ctrlKey: true });
    expect(result.current.open).toBe(true);
    act(() => result.current.close());
    expect(result.current.open).toBe(false);

    active = false;
    rerender();
    fireEvent.keyDown(window, { code: "KeyF", key: "f", ctrlKey: true });
    expect(result.current.open).toBe(false);

    active = true;
    rerender();
    fireEvent.keyDown(window, { code: "KeyF", key: "f", ctrlKey: true });
    expect(result.current.open).toBe(true);
  });
});
