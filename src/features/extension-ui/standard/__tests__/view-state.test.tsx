/* @vitest-environment jsdom */
import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useStandardViewState } from "../view-state";
import type { StandardView } from "../types";

const field = (id: string): StandardView => ({
  type: "textField",
  id,
  label: { default: id },
  value: "",
});

describe("useStandardViewState", () => {
  it("replaces the bounded map atomically and never sends orphaned fields", () => {
    const first: StandardView = {
      type: "stack",
      children: [field("com.example.first"), field("com.example.orphan")],
    };
    const second = field("com.example.second");
    const { result } = renderHook(() => useStandardViewState(first));

    act(() => result.current.setValue("com.example.first", "kept only in first"));
    expect(result.current.payload().fields).toEqual({
      "com.example.first": "kept only in first",
      "com.example.orphan": "",
    });

    act(() => result.current.replaceView(second));
    expect(result.current.values.size).toBe(1);
    expect(result.current.payload().fields).toEqual({ "com.example.second": "" });
  });
});
