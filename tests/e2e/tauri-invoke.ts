interface InvocationResult {
  ok: boolean;
  value?: unknown;
}

export async function invokeTauri<T>(
  command: string,
  payload: Record<string, unknown> = {},
): Promise<T> {
  const result = await browser.execute(
    async (requestedCommand, requestedPayload): Promise<InvocationResult> => {
      try {
        const invoke = window.__TAURI__?.core?.invoke;
        if (!invoke) return { ok: false };
        const value = await invoke(requestedCommand, requestedPayload);
        return { ok: true, value };
      } catch {
        return { ok: false };
      }
    },
    command,
    payload,
  );
  if (!result.ok) throw new Error("E2E invocation failed");
  return result.value as T;
}
