import { randomUUID } from "node:crypto";
import { Worker } from "node:worker_threads";
import { OFFICE_LIMITS } from "../common/constants.mjs";
import { OfficePluginError } from "../common/errors.mjs";

const DEFAULT_WORKER = new URL("./create-worker.mjs", import.meta.url);

export function createPdfWorkerClient(workerUrl = DEFAULT_WORKER) {
  const pending = new Map();
  let worker;

  function render(payload) {
    if (pending.size >= OFFICE_LIMITS.maxPdfRenderRequests) {
      return Promise.reject(new OfficePluginError("too_many_requests"));
    }
    const activeWorker = ensureWorker();
    const id = randomUUID();
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        failWorker(activeWorker);
      }, OFFICE_LIMITS.pdfRenderTimeoutMs);
      timer.unref();
      pending.set(id, { resolve, reject, timer });
      try {
        activeWorker.postMessage({ id, payload });
      } catch {
        failWorker(activeWorker);
      }
    });
  }

  function ensureWorker() {
    if (worker) return worker;
    const created = new Worker(workerUrl);
    created.unref();
    created.on("message", (message) => settle(created, message));
    created.on("messageerror", () => failWorker(created));
    created.on("error", () => failWorker(created));
    created.on("exit", () => failWorker(created));
    worker = created;
    return created;
  }

  function settle(source, message) {
    if (source !== worker || typeof message?.id !== "string") {
      failWorker(source);
      return;
    }
    const request = pending.get(message.id);
    if (!request) return;
    clearTimeout(request.timer);
    pending.delete(message.id);
    if (message.error) {
      request.reject(new OfficePluginError(message.error, message.details));
      return;
    }
    if (
      !(message.bytes instanceof Uint8Array)
      || message.bytes.length === 0
      || message.bytes.length > OFFICE_LIMITS.maxOutputBytes
      || !Number.isInteger(message.pages)
      || message.pages < 1
      || message.pages > OFFICE_LIMITS.maxPdfPages
    ) {
      request.reject(new OfficePluginError("operation_failed"));
      failWorker(source);
      return;
    }
    request.resolve({
      bytes: Buffer.from(message.bytes),
      pages: message.pages,
    });
  }

  function failWorker(source) {
    if (source !== worker) return;
    worker = undefined;
    void source.terminate();
    for (const request of pending.values()) {
      clearTimeout(request.timer);
      request.reject(new OfficePluginError("operation_failed"));
    }
    pending.clear();
  }

  function stop() {
    if (worker) failWorker(worker);
  }

  return Object.freeze({ render, stop });
}

const defaultClient = createPdfWorkerClient();

export function renderPdfInWorker(payload) {
  return defaultClient.render(payload);
}
