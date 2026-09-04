import assert from "node:assert/strict";
import { test } from "node:test";
import {
  isTransientAuditFailure,
  runAuditWithRetry,
} from "./npm-audit-with-retry.mjs";

test("recognizes an unavailable advisory service", () => {
  assert.equal(isTransientAuditFailure("503 Service Unavailable"), true);
  assert.equal(isTransientAuditFailure("npm error audit endpoint returned an error"), true);
  assert.equal(isTransientAuditFailure("found 1 high severity vulnerability"), false);
});

test("retries one transient service failure", async () => {
  let attempts = 0;
  const reports = [];
  await runAuditWithRetry({
    execute: async () => {
      attempts += 1;
      return attempts === 1
        ? { exitCode: 1, output: "503 Service Unavailable" }
        : { exitCode: 0, output: "found 0 vulnerabilities" };
    },
    report: (message) => reports.push(message),
    wait: async () => {},
  });
  assert.equal(attempts, 2);
  assert.equal(reports.length, 1);
});

test("does not retry a reported vulnerability", async () => {
  let attempts = 0;
  await assert.rejects(
    runAuditWithRetry({
      execute: async () => {
        attempts += 1;
        return { exitCode: 1, output: "found 1 high severity vulnerability" };
      },
      wait: async () => {},
    }),
    /npm_audit_advisory_failure/u,
  );
  assert.equal(attempts, 1);
});

test("remains failed after the bounded transient retry", async () => {
  let attempts = 0;
  await assert.rejects(
    runAuditWithRetry({
      execute: async () => {
        attempts += 1;
        return { exitCode: 1, output: "ETIMEDOUT" };
      },
      report: () => {},
      wait: async () => {},
    }),
    /npm_audit_service_unavailable/u,
  );
  assert.equal(attempts, 2);
});
