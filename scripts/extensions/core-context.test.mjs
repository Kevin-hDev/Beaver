import assert from "node:assert/strict";
import test from "node:test";
import { setTimeout as delay } from "node:timers/promises";
import {
  coreContextForTransport,
  coreContextTimeout,
  runWithCoreContext,
} from "../../src-tauri/resources/extension-host/core-context.mjs";

const first = {
  id: "123e4567-e89b-42d3-a456-426614174000",
  secret: "ab".repeat(32),
  remainingMs: 1_000,
};
const second = {
  id: "223e4567-e89b-42d3-a456-426614174000",
  secret: "cd".repeat(32),
  remainingMs: 1_000,
};

test("parallel tool promises retain only their own private scope", async () => {
  const seen = await Promise.all([
    runWithCoreContext(first, async () => {
      await delay(5);
      return coreContextForTransport();
    }),
    runWithCoreContext(second, async () => {
      await delay(1);
      return coreContextForTransport();
    }),
  ]);
  assert.equal(seen[0].id, first.id);
  assert.equal(seen[0].secret, first.secret);
  assert.equal(seen[1].id, second.id);
  assert.equal(seen[1].secret, second.secret);
  assert.equal(coreContextForTransport(), undefined);
});

test("late callbacks cannot extend the original deadline", async () => {
  await assert.rejects(
    runWithCoreContext({ ...first, remainingMs: 2 }, async () => {
      await delay(8);
      return coreContextForTransport();
    }),
    /core_context_expired/u,
  );
});

test("invalid or oversized scopes fail before extension code", () => {
  assert.throws(() => runWithCoreContext({ ...first, extensionId: "forged" }, () => {}), /invalid_tool_scope/u);
  assert.throws(() => runWithCoreContext({ ...first, secret: "x".repeat(64) }, () => {}), /invalid_tool_scope/u);
  assert.throws(() => runWithCoreContext({ ...first, remainingMs: 60_000 }, () => {}), /invalid_tool_scope/u);
  assert.equal(coreContextTimeout(30_000), 30_000);
});
