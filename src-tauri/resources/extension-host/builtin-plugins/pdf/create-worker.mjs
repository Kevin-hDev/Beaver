import { parentPort } from "node:worker_threads";
import { OFFICE_LIMITS } from "../common/constants.mjs";
import { renderPdf } from "./render.mjs";

if (!parentPort) throw new Error("pdf_worker_unavailable");

parentPort.on("message", async (request) => {
  if (!validRequest(request)) {
    parentPort.postMessage({ id: safeId(request), error: "operation_failed" });
    return;
  }
  try {
    const result = await renderPdf(request.payload);
    if (
      result.bytes.length === 0
      || result.bytes.length > OFFICE_LIMITS.maxOutputBytes
    ) {
      throw new Error("pdf_output_invalid");
    }
    const bytes = Uint8Array.from(result.bytes);
    parentPort.postMessage(
      { id: request.id, bytes, pages: result.pages },
      [bytes.buffer],
    );
  } catch (error) {
    const code = error?.code === "unsupported_character"
      ? "unsupported_character"
      : "operation_failed";
    parentPort.postMessage({ id: request.id, error: code });
  }
});

function validRequest(request) {
  const title = request?.payload?.title;
  const paragraphs = request?.payload?.paragraphs;
  return typeof request?.id === "string"
    && /^[0-9a-f-]{36}$/u.test(request.id)
    && (title === undefined || (typeof title === "string" && title.length <= 300))
    && Array.isArray(paragraphs)
    && paragraphs.length > 0
    && paragraphs.length <= OFFICE_LIMITS.maxBlocks
    && paragraphs.every(
      (value) => typeof value === "string" && value.length <= 32_767,
    )
    && paragraphs.reduce((sum, value) => sum + value.length, 0)
      <= OFFICE_LIMITS.maxTextChars;
}

function safeId(request) {
  return typeof request?.id === "string" && request.id.length <= 64
    ? request.id
    : "";
}
