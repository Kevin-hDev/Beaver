import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  isTransientAuditFailure,
  runAuditWithRetry,
  SERVICE_UNAVAILABLE_EXIT_CODE,
} from "./npm-audit-with-retry.mjs";

const auditWorkflow = readFileSync(
  new URL("../../.github/workflows/audit.yml", import.meta.url),
  "utf8",
);

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
    (error) => {
      assert.match(error.message, /npm_audit_service_unavailable/u);
      assert.equal(error.exitCode, SERVICE_UNAVAILABLE_EXIT_CODE);
      return true;
    },
  );
  assert.equal(attempts, 2);
});

test("the workflow falls back only when npm's advisory service is unavailable", () => {
  assert.match(auditWorkflow, /frontend_status[^]*host_status/u);
  assert.match(auditWorkflow, /service-unavailable=true/u);
  assert.match(
    auditWorkflow,
    /if: steps\.npm-audit\.outputs\.service-unavailable == 'true'/u,
  );
  assert.match(
    auditWorkflow,
    /google\/osv-scanner-action\/osv-scanner-action@baa4139e56d6312335d899e6ba045fa16d1d3d0b/u,
  );
  assert.match(auditWorkflow, /--lockfile=package-lock\.json/u);
  assert.match(
    auditWorkflow,
    /--lockfile=src-tauri\/resources\/extension-host\/package-lock\.json/u,
  );
});
