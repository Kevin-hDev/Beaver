import { LIMITS } from "./contract.mjs";

const MAX_COUNTER = Number.MAX_SAFE_INTEGER;

export function createEventDelivery(getExtensions, publishActivity = () => {}) {
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
    return Object.freeze({ ...activity });
  }

  function publish() {
    // Callback outcomes exist only in this process, so publish snapshots instead of recreating them in Rust.
    publishActivity(snapshot());
  }

  function enqueue(event, payload) {
    if (queue.length >= LIMITS.maxEventQueuePerHost) {
      activity.dropped = increment(activity.dropped);
      publish();
      return { queued: false };
    }
    queue.push({ event, payload });
    activity.queued = increment(activity.queued);
    publish();
    if (!draining) {
      draining = true;
      queueMicrotask(drain);
    }
    return { queued: true };
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
          publish();
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

  function handlerActivity(activeHandlers) {
    activity.activeHandlers = activeHandlers;
    publish();
  }

  return Object.freeze({ enqueue, handlerActivity });
}

function increment(value) {
  return value < MAX_COUNTER ? value + 1 : value;
}

function add(value, amount) {
  return Math.min(MAX_COUNTER, value + amount);
}
