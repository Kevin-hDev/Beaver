import assert from "node:assert/strict";
import test from "node:test";

import { createEventDelivery } from "../../src-tauri/resources/extension-host/event-delivery.mjs";
import { createExtensionApi } from "../../src-tauri/resources/extension-host/extension-api.mjs";
import { LIMITS } from "../../src-tauri/resources/extension-host/contract.mjs";

test("slow_observer_does_not_block_or_grow_unbounded", async () => {
  let release;
  const blocked = new Promise((resolve) => { release = resolve; });
  const delivery = createEventDelivery(() => [{
    context: { emit: () => blocked.then(() => ({ delivered: 1, dropped: 0, timedOut: 0, activeHandlers: 0 })) },
  }]);

  const first = delivery.enqueue("session.turn.started", { status: "started" });
  assert.equal(first.queued, true);
  for (let index = 0; index < LIMITS.maxEventQueuePerHost; index += 1) {
    delivery.enqueue("session.turn.started", { status: "started" });
  }
  assert.equal(delivery.enqueue("session.turn.started", {}).queued, false);
  release();
  await new Promise((resolve) => setImmediate(resolve));
});

test("event payloads are isolated between extension handlers", async () => {
  const seen = [];
  const delivery = createEventDelivery(() => [
    { context: { emit: async (_event, payload) => {
      payload.status = "mutated";
      return { delivered: 1, dropped: 0, timedOut: 0, activeHandlers: 0 };
    } } },
    { context: { emit: async (_event, payload) => {
      seen.push(payload.status);
      return { delivered: 1, dropped: 0, timedOut: 0, activeHandlers: 0 };
    } } },
  ]);
  delivery.enqueue("session.turn.started", { status: "started" });
  await new Promise((resolve) => setImmediate(resolve));
  assert.deepEqual(seen, ["started"]);
});

test("host activity reports real handler outcomes", async () => {
  const snapshots = [];
  let delivery;
  const context = createExtensionApi(
    { id: "com.beaver.activity-test", manifest: { apiLevel: "stable" } },
    (activeHandlers) => delivery.handlerActivity(activeHandlers),
  );
  context.api.on("session.turn.started", () => {});
  delivery = createEventDelivery(
    () => [{ context }],
    (activity) => snapshots.push(activity),
  );

  delivery.enqueue("session.turn.started", {});
  await new Promise((resolve) => setImmediate(resolve));

  assert.equal(snapshots.some((activity) => activity.activeHandlers === 1), true);
  assert.deepEqual(snapshots.at(-1), {
    queued: 1,
    delivered: 1,
    dropped: 0,
    timedOut: 0,
    activeHandlers: 0,
  });
});

test("a callback that never finishes keeps its bounded active slot", async () => {
  let release;
  const blocked = new Promise((resolve) => { release = resolve; });
  const context = extensionContext();
  context.api.on("session.turn.started", () => blocked);
  const pending = Array.from(
    { length: LIMITS.maxInFlightHandlers },
    () => context.emit("session.turn.started", {}),
  );
  const overflow = await context.emit("session.turn.started", {});
  assert.equal(overflow.dropped, 1);
  release();
  await Promise.all(pending);
});

test("unsubscribe during delivery only affects later events", async () => {
  const context = extensionContext();
  const seen = [];
  let unsubscribeSecond;
  context.api.on("session.turn.started", () => {
    seen.push("first");
    unsubscribeSecond();
  });
  unsubscribeSecond = context.api.on("session.turn.started", () => seen.push("second"));

  await context.emit("session.turn.started", {});
  await context.emit("session.turn.started", {});
  assert.deepEqual(seen, ["first", "second", "first"]);
});

function extensionContext() {
  return createExtensionApi({
    id: "com.beaver.events-test",
    manifest: { apiLevel: "stable" },
  });
}
