interface InvocationResult {
  error?: string;
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
      } catch (error) {
        const message = error instanceof Error
          ? error.message
          : typeof error === "string" ? error : undefined;
        return { ok: false, error: message };
      }
    },
    command,
    payload,
  );
  if (!result.ok) throw new Error(result.error ?? "E2E invocation failed");
  return result.value as T;
}
