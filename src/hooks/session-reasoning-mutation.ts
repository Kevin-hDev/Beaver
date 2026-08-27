const MAX_PENDING_REASONING_MUTATIONS = 64;

const pending = new Map<string, Promise<void>>();

export function runReasoningMutation(
  sessionId: string,
  mutation: () => Promise<void>,
): Promise<void> {
  const previous = pending.get(sessionId);
  if (!previous && pending.size >= MAX_PENDING_REASONING_MUTATIONS) {
    return Promise.reject(new Error("session-update-unavailable"));
  }
  const task = (previous ?? Promise.resolve())
    .catch(() => {})
    .then(mutation);
  pending.set(sessionId, task);
  const cleanup = () => {
    if (pending.get(sessionId) === task) pending.delete(sessionId);
  };
  void task.then(cleanup, cleanup);
  return task;
}

export async function awaitPendingReasoning(sessionId: string): Promise<void> {
  await pending.get(sessionId);
}
