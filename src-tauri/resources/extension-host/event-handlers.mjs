import { LIMITS, supportsEvent, TIMEOUTS } from "./contract.mjs";
import { validIdentifier } from "./contribution-validation.mjs";

const inFlightHandlers = new Set();

export function createEventHandlers(onActivity = () => {}) {
  const events = new Map();
  let handlerCount = 0;

  function on(eventName, handler) {
    if (typeof handler !== "function" || handlerCount >= LIMITS.maxEventsPerExtension) {
      throw new Error("invalid_event_handler");
    }
    const event = String(eventName);
    if (!validIdentifier(event) || !supportsEvent(event)) {
      throw new Error("invalid_event_name");
    }
    events.set(event, [...(events.get(event) ?? []), handler]);
    handlerCount += 1;
    let subscribed = true;
    return () => {
      if (!subscribed) return;
      subscribed = false;
      const next = (events.get(event) ?? []).filter((item) => item !== handler);
      if (next.length === 0) events.delete(event);
      else events.set(event, next);
      handlerCount -= 1;
    };
  }

  async function emit(event, payload) {
    const result = { delivered: 0, dropped: 0, timedOut: 0, activeHandlers: 0 };
    for (const handler of [...(events.get(event) ?? [])]) {
      const outcome = await run(handler, payload, onActivity);
      if (outcome === "delivered") result.delivered += 1;
      else if (outcome === "timedOut") result.timedOut += 1;
      else result.dropped += 1;
      result.activeHandlers = inFlightHandlers.size;
    }
    return result;
  }

  return Object.freeze({ events, on, emit });
}

async function run(handler, payload, onActivity) {
  if (inFlightHandlers.size >= LIMITS.maxInFlightHandlers) return "dropped";
  const execution = Promise.resolve().then(() => handler(payload));
  inFlightHandlers.add(execution);
  onActivity(inFlightHandlers.size);
  void execution.finally(() => {
    inFlightHandlers.delete(execution);
    onActivity(inFlightHandlers.size);
  }).catch(() => {});
  let timer;
  try {
    return await Promise.race([
      execution.then(() => "delivered", () => "dropped"),
      new Promise((resolve) => {
        timer = setTimeout(() => resolve("timedOut"), TIMEOUTS.eventHandlerTimeoutMs);
        timer.unref();
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}
