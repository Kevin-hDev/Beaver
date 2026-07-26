type TauriCleanup = () => void | Promise<void>;

export type TauriUnlisten = Promise<TauriCleanup>;

const cleaned = new WeakSet<TauriUnlisten>();
let listenerCleanupGuardInstalled = false;

function isTauriListenerCleanupError(reason: unknown): boolean {
  if (!(reason instanceof Error)) return false;
  return reason.message.includes("listeners[eventId].handlerId")
    && (reason.stack?.includes("_unlisten") ?? false);
}

function ignoreCleanupFailure(result: void | Promise<void>) {
  if (result && typeof result.then === "function") {
    void result.catch(() => {});
  }
}

export function cleanupTauriListener(unlisten: TauriUnlisten) {
  if (cleaned.has(unlisten)) return;
  cleaned.add(unlisten);
  void unlisten
    .then((fn) => {
      try {
        ignoreCleanupFailure(fn());
      } catch {
        // Listener cleanup must never crash the app.
      }
    })
    .catch(() => {});
}

export function installTauriListenerCleanupGuard() {
  if (listenerCleanupGuardInstalled || typeof window === "undefined") return;
  listenerCleanupGuardInstalled = true;
  window.addEventListener("unhandledrejection", (event) => {
    if (isTauriListenerCleanupError(event.reason)) {
      event.preventDefault();
    }
  });
}
