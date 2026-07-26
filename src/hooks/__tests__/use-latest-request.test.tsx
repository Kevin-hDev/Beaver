import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useLatestRequest } from "../use-latest-request";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe("useLatestRequest", () => {
  it("discards a request that finishes after a newer one", async () => {
    const first = deferred<string>();
    const second = deferred<string>();
    const { result } = renderHook(() => useLatestRequest());
    let firstResult: string | undefined;
    let secondResult: string | undefined;

    const firstRun = result.current(() => first.promise).then((value) => {
      firstResult = value;
    });
    const secondRun = result.current(() => second.promise).then((value) => {
      secondResult = value;
    });
    await act(async () => {
      second.resolve("new");
      await secondRun;
      first.resolve("old");
      await firstRun;
    });

    expect(secondResult).toBe("new");
    expect(firstResult).toBeUndefined();
  });

  it("discards a response received after unmount", async () => {
    const pending = deferred<string>();
    const { result, unmount } = renderHook(() => useLatestRequest());
    let received: string | undefined;
    const run = result.current(() => pending.promise).then((value) => {
      received = value;
    });

    unmount();
    await act(async () => {
      pending.resolve("late");
      await run;
    });

    expect(received).toBeUndefined();
  });
});
