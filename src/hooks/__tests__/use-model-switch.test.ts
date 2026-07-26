import { describe, it, expect, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useModelSwitch } from "../use-model-switch";

const defaultParams = {
  currentModel: "gpt-4",
  currentProvider: "openai",
  messagesLength: 0,
  onApplySwitch: vi.fn(),
  onNewSession: vi.fn(),
};

describe("useModelSwitch", () => {
  it("initialise pendingSwitch à null", () => {
    const { result } = renderHook(() => useModelSwitch(defaultParams));
    expect(result.current.pendingSwitch).toBeNull();
  });

  it("ne fait rien si le modèle est identique", () => {
    const onApplySwitch = vi.fn();
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, onApplySwitch }),
    );

    act(() => {
      result.current.handleModelSelect("gpt-4", "openai");
    });

    expect(onApplySwitch).not.toHaveBeenCalled();
    expect(result.current.pendingSwitch).toBeNull();
  });

  it("appelle onApplySwitch directement si pas de messages", () => {
    const onApplySwitch = vi.fn();
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, messagesLength: 0, onApplySwitch }),
    );

    act(() => {
      result.current.handleModelSelect("claude-3", "anthropic");
    });

    expect(onApplySwitch).toHaveBeenCalledWith("claude-3", "anthropic");
    expect(result.current.pendingSwitch).toBeNull();
  });

  it("ouvre le dialog si messages > 0 et aucun remember", () => {
    const onApplySwitch = vi.fn();
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, messagesLength: 3, onApplySwitch }),
    );

    act(() => {
      result.current.handleModelSelect("claude-3", "anthropic");
    });

    expect(onApplySwitch).not.toHaveBeenCalled();
    expect(result.current.pendingSwitch).toEqual({
      model: "claude-3",
      provider: "anthropic",
    });
  });

  it("applique directement si rememberedRef = continue", () => {
    const onApplySwitch = vi.fn();
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, messagesLength: 5, onApplySwitch }),
    );

    act(() => {
      result.current.rememberedRef.current = "continue";
      result.current.handleModelSelect("claude-3", "anthropic");
    });

    expect(onApplySwitch).toHaveBeenCalledWith("claude-3", "anthropic");
    expect(result.current.pendingSwitch).toBeNull();
  });

  it("crée nouvelle session si rememberedRef = new", () => {
    const onNewSession = vi.fn();
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, messagesLength: 5, onNewSession }),
    );

    act(() => {
      result.current.rememberedRef.current = "new";
      result.current.handleModelSelect("claude-3", "anthropic");
    });

    expect(onNewSession).toHaveBeenCalledWith("claude-3", "anthropic");
    expect(result.current.pendingSwitch).toBeNull();
  });

  it("setPendingSwitch permet de fermer le dialog", () => {
    const { result } = renderHook(() =>
      useModelSwitch({ ...defaultParams, messagesLength: 2 }),
    );

    act(() => {
      result.current.handleModelSelect("claude-3", "anthropic");
    });

    expect(result.current.pendingSwitch).not.toBeNull();

    act(() => {
      result.current.setPendingSwitch(null);
    });

    expect(result.current.pendingSwitch).toBeNull();
  });
});
