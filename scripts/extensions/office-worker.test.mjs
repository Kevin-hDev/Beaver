import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import {
  callOffice,
  createOfficeHost,
  hostDirectory,
  syncOfficePlugins,
} from "./office-test-helpers.mjs";

test("isolates an uncaught PDF worker failure from its caller", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-pdf-worker-"));
  const workerPath = join(directory, "crash-worker.mjs");
  await writeFile(
    workerPath,
    `import { parentPort } from "node:worker_threads";
     parentPort.on("message", () => queueMicrotask(() => {
       throw new Error("worker failure");
     }));`,
    { mode: 0o600 },
  );
  const clientModule = pathToFileURL(
    join(hostDirectory, "builtin-plugins/pdf/worker-client.mjs"),
  );
  const { createPdfWorkerClient } = await import(clientModule.href);
  const client = createPdfWorkerClient(pathToFileURL(workerPath));
  try {
    await assert.rejects(
      client.render({ paragraphs: ["test"] }),
      (error) => error?.code === "operation_failed",
    );
  } finally {
    client.stop();
    await rm(directory, { recursive: true, force: true });
  }
});

test("reports PDF worker saturation as retryable", async () => {
  const directory = await mkdtemp(join(tmpdir(), "beaver-pdf-busy-worker-"));
  const workerPath = join(directory, "busy-worker.mjs");
  await writeFile(
    workerPath,
    `import { parentPort } from "node:worker_threads";
     parentPort.on("message", () => {});`,
    { mode: 0o600 },
  );
  const clientModule = pathToFileURL(
    join(hostDirectory, "builtin-plugins/pdf/worker-client.mjs"),
  );
  const { createPdfWorkerClient } = await import(clientModule.href);
  const client = createPdfWorkerClient(pathToFileURL(workerPath));
  const pending = Array.from({ length: 4 }, () =>
    client.render({ paragraphs: ["test"] }).catch(() => {}));
  try {
    await assert.rejects(
      client.render({ paragraphs: ["overflow"] }),
      (error) => error?.code === "too_many_requests",
    );
  } finally {
    client.stop();
    await Promise.all(pending);
    await rm(directory, { recursive: true, force: true });
  }
});

test("keeps concurrent PDF font state isolated by document", async () => {
  const workspace = await mkdtemp(join(tmpdir(), "beaver-pdf-concurrent-"));
  const host = createOfficeHost();
  const samples = ["a ≤ b", "日本語", "العربية", "नमस्ते दुनिया"];
  try {
    await syncOfficePlugins(host);
    const created = await Promise.all(samples.map((text, index) =>
      callOffice(host, workspace, "beaver.office.pdf.create", {
        path: `${index}.pdf`,
        paragraphs: [text],
      })));
    assert.equal(created.every((result) => result.isError !== true), true);
    const inspected = await Promise.all(samples.map((_, index) =>
      callOffice(host, workspace, "beaver.office.pdf.inspect", {
        path: `${index}.pdf`,
        maxPages: 1,
      })));
    inspected.forEach((result, index) => {
      const text = JSON.parse(result.content).pages[0].text;
      assert.equal(text.includes(samples[index]), true);
    });
  } finally {
    host.stop();
    await rm(workspace, { recursive: true, force: true });
  }
});
