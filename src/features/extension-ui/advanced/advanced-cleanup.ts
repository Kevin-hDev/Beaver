import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import type { AdvancedCleanup, AdvancedCleanupPlan } from "./advanced-types";

/** Detach every Beaver-owned resource before invoking any extension callback. */
export function cleanupAdvancedPlans(plans: readonly AdvancedCleanupPlan[]): Promise<void> {
  const deadline = performance.now() + UI_LIMITS.maxAdvancedCleanupMs;
  const callbacks = [...plans].reverse().flatMap((plan) => plan.detach());
  return runAdvancedCleanups(callbacks, deadline);
}

/** The deadline ends our wait; it cannot interrupt synchronous extension JavaScript. */
export async function runAdvancedCleanups(
  callbacks: readonly AdvancedCleanup[],
  deadline: number,
): Promise<void> {
  if (!callbacks.length) return;
  let timeout: ReturnType<typeof setTimeout> | undefined;
  const expired = new Promise<void>((resolve) => {
    timeout = setTimeout(resolve, Math.max(0, deadline - performance.now()));
  });
  // Independent callbacks must all run even if an earlier one rejects or never settles.
  const tasks = callbacks.map((callback) => Promise.resolve().then(callback).catch(() => {
    // Teardown failure is isolated: the context is already closed and DOM detached.
  }));
  try {
    await Promise.race([Promise.all(tasks), expired]);
  } finally {
    clearTimeout(timeout);
  }
}
