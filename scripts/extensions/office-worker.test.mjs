import assert from "node:assert/strict";
import { mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { hostDirectory } from "./office-test-helpers.mjs";

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
