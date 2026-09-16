import { LIMITS } from "./contract.mjs";

const MAX_COUNTER = Number.MAX_SAFE_INTEGER;

export function createEventDelivery(getExtensions, getActiveHandlers = () => 0) {
  const queue = [];
  const activity = {
    queued: 0,
    delivered: 0,
    dropped: 0,
    timedOut: 0,
    activeHandlers: 0,
  };
  let draining = false;

  function snapshot() {
    activity.activeHandlers = getActiveHandlers();
    return Object.freeze({ ...activity });
  }

  function enqueue(event, payload) {
    if (queue.length >= LIMITS.maxEventQueuePerHost) {
      activity.dropped = increment(activity.dropped);
      return { queued: false, activity: snapshot() };
    }
    queue.push({ event, payload });
    activity.queued = increment(activity.queued);
    if (!draining) {
      draining = true;
      queueMicrotask(drain);
    }
    return { queued: true, activity: snapshot() };
  }

  async function drain() {
    try {
      while (queue.length > 0) {
        const item = queue.shift();
        for (const extension of getExtensions()) {
          const result = await extension.context.emit(
            item.event,
            structuredClone(item.payload),
          );
          activity.delivered = add(activity.delivered, result.delivered);
          activity.dropped = add(activity.dropped, result.dropped);
          activity.timedOut = add(activity.timedOut, result.timedOut);
        }
      }
    } finally {
      draining = false;
      if (queue.length > 0) {
        draining = true;
        queueMicrotask(drain);
      }
    }
  }

  return Object.freeze({ enqueue, activity: snapshot });
}

function increment(value) {
  return value < MAX_COUNTER ? value + 1 : value;
}

function add(value, amount) {
  return Math.min(MAX_COUNTER, value + amount);
}
