import { LIMITS, RESULT_FILE_PURPOSES } from "./contract.mjs";

function frozenRecord(fields) { return Object.freeze(Object.assign(Object.create(null), fields)); }

export function snapshotToolResult(value) {
  if (typeof value === "string") return frozenRecord({ content: value, isError: false });
  if (!value || typeof value !== "object") throw new Error("invalid_tool_result");
  const raw = snapshotResult(value);
  const content = snapshotValue(raw.content);
  if (typeof content !== "string" && !Array.isArray(content)) throw new Error("invalid_tool_result");
  return frozenRecord({
    content,
    isError: raw.isError === true,
    displaySummary: typeof raw.displaySummary === "string" ? raw.displaySummary : undefined,
    truncated: raw.truncated === true,
  });
}

function snapshotValue(value) {
  if (typeof value === "string") return value;
  if (!Array.isArray(value)) throw new Error("invalid_tool_result");
  const length = value.length;
  if (!Number.isSafeInteger(length) || length > LIMITS.maxResultBlocks) {
    throw new Error("invalid_tool_result");
  }
  let files = 0;
  let textBytes = 0;
  const blocks = new Array(length);
  for (let index = 0; index < length; index += 1) {
    const block = value[index];
    const snapshot = snapshotBlock(block);
    if (snapshot.type === "text") {
      textBytes += Buffer.byteLength(snapshot.text, "utf8");
      if (textBytes > LIMITS.maxResultTextBytes) throw new Error("invalid_tool_result");
    }
    if (snapshot.type === "file" && ++files > LIMITS.maxResultFiles) throw new Error("invalid_tool_result");
    blocks[index] = snapshot;
  }
  return Object.freeze(blocks);
}

function snapshotBlock(block) {
  if (!block || typeof block !== "object") throw new Error("invalid_tool_result");
  const type = block.type;
  if (type === "text") {
    const text = block.text;
    if (typeof text === "string") return frozenRecord({ type, text });
  }
  if (type === "file") {
    const path = block.path;
    const purpose = block.purpose;
    const displayName = block.displayName;
    if (typeof path === "string" && path.length > 0 && Array.from(path).length <= LIMITS.maxPathChars && validPurpose(purpose) && validDisplayName(displayName)) return frozenRecord({ type, path, purpose, displayName });
  }
  throw new Error("invalid_tool_result");
}

function validPurpose(value) {
  if (typeof value !== "string") return false;
  for (let index = 0; index < RESULT_FILE_PURPOSES.length; index += 1) {
    if (RESULT_FILE_PURPOSES[index] === value) return true;
  }
  return false;
}

function snapshotResult(value) {
  return Object.assign(Object.create(null), {
    content: value.content,
    isError: value.isError,
    displaySummary: value.displaySummary,
    truncated: value.truncated,
  });
}

function validDisplayName(value) {
  return value === undefined || (
    typeof value === "string"
    && value.length > 0
    && Array.from(value).length <= LIMITS.maxExtensionNameChars
    && !/[\p{Cc}]/u.test(value)
  );
}
