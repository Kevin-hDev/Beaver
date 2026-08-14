import assert from "node:assert/strict";
import test from "node:test";

import {
  createNativeJourney,
  NATIVE_CEF_STAGE_CEILINGS_MS,
  NATIVE_JOURNEY_MOCHA_TIMEOUT_MS,
  NATIVE_JOURNEY_TIMEOUT_MS,
} from "./native-journey-deadline.mjs";

test("spent journey time cannot be reissued to a later stage", async () => {
  let currentTime = 100;
  const receivedBudgets = [];
  const journey = createNativeJourney({
    timeoutMs: 1_000,
    cleanupTimeoutMs: 100,
    stageCeilings: { first: 900, second: 900 },
    now: () => currentTime,
    report: () => {},
  });

  await journey.run("first", async ({ timeoutMs }) => {
    receivedBudgets.push(timeoutMs);
    currentTime = 500;
  });
  await journey.run("second", async ({ timeoutMs }) => {
    receivedBudgets.push(timeoutMs);
  });

  assert.deepEqual(receivedBudgets, [900, 600]);
});

test("CEF helper turnover is bounded by its stage and the original journey", async () => {
  let currentTime = 0;
  const journey = createNativeJourney({
    now: () => currentTime,
    report: () => {},
  });
  currentTime = 55_000;

  const receivedBudget = await journey.run(
    "cef_helper_turnover",
    async ({ timeoutMs }) => timeoutMs,
  );

  assert.equal(NATIVE_CEF_STAGE_CEILINGS_MS.cef_helper_turnover, 20_000);
  assert.equal(receivedBudget, 5_000);
});

test("the validated stage policy cannot be replaced after journey creation", async () => {
  const stageCeilings = { stable_stage: 50 };
  const journey = createNativeJourney({
    timeoutMs: 100,
    cleanupTimeoutMs: 20,
    stageCeilings,
    now: () => 0,
    report: () => {},
  });
  stageCeilings.stable_stage = 500;

  const receivedBudget = await journey.run(
    "stable_stage",
    async ({ timeoutMs }) => timeoutMs,
  );

  assert.equal(receivedBudget, 50);
});

test("a suspended stage fails at its ceiling and names the failed boundary", async () => {
  const events = [];
  const journey = createNativeJourney({
    timeoutMs: 100,
    cleanupTimeoutMs: 20,
    stageCeilings: { page_load: 10 },
    report: (event) => events.push(event),
  });

  await assert.rejects(
    journey.run("page_load", () => new Promise(() => {})),
    (error) => error.code === "stage-timeout" && error.stage === "page_load",
  );

  assert.deepEqual(
    events.map(({ stage, state, code }) => ({ stage, state, code })),
    [
      { stage: "page_load", state: "started", code: undefined },
      { stage: "page_load", state: "failed", code: "stage-timeout" },
    ],
  );
});

test("an operation error stays available as the technical cause", async () => {
  const technical = new Error("private technical detail");
  const events = [];
  const journey = createNativeJourney({
    timeoutMs: 100,
    cleanupTimeoutMs: 20,
    stageCeilings: { native_observation: 50 },
    report: (event) => events.push(event),
  });

  await assert.rejects(
    journey.run("native_observation", async () => { throw technical; }),
    (error) => error.code === "stage-error"
      && error.stage === "native_observation"
      && error.cause === technical,
  );
  assert.equal(JSON.stringify(events).includes("private technical detail"), false);
  assert.equal(events.at(-1)?.code, "stage-error");
});

test("cleanup keeps its bounded grace after the work deadline expires", async () => {
  let currentTime = 0;
  const journey = createNativeJourney({
    timeoutMs: 10,
    cleanupTimeoutMs: 20,
    stageCeilings: { work: 10 },
    now: () => currentTime,
    report: () => {},
  });
  currentTime = 11;

  const result = await journey.cleanup("page_server_close", async ({ timeoutMs }) => timeoutMs);

  assert.equal(result, 20);
});

test("unknown or unbounded stage names fail before execution", async () => {
  let called = false;
  const journey = createNativeJourney({
    timeoutMs: 100,
    cleanupTimeoutMs: 20,
    stageCeilings: { known_stage: 50 },
    report: () => {},
  });

  await assert.rejects(
    journey.run("unknown-stage", async () => { called = true; }),
    /Native journey stage is invalid/u,
  );
  assert.equal(called, false);
});

test("the external Mocha guard includes the bounded cleanup grace", () => {
  assert.ok(NATIVE_JOURNEY_MOCHA_TIMEOUT_MS > NATIVE_JOURNEY_TIMEOUT_MS);
});
