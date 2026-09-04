import { createJiti } from "jiti";
import { fileURLToPath } from "node:url";
import { HOST_LOAD_STAGE_METHOD, LIMITS, LOAD_STAGES, supportsEvent, TIMEOUTS } from "./contract.mjs";
import { createExtensionApi } from "./extension-api.mjs";
import { createDiagnostic } from "./diagnostics.mjs";
import { notifyCore } from "./protocol.mjs";
import { clearUiActions, invokeUiAction } from "./ui-actions.mjs";

const sdkPath = fileURLToPath(new URL("./sdk/index.mjs", import.meta.url));
const jiti = createJiti(import.meta.url, {
  alias: { "@beaver/sdk": sdkPath },
  fsCache: false,
  interopDefault: true,
  moduleCache: false,
  sourceMaps: false,
});
const extensions = new Map();
const tools = new Map();

export async function resetExtensions() {
  await deactivateAll();
  return { reset: true };
}

export async function callExtensionTool(name, arguments_, context) {
  const entry = tools.get(name);
  if (!entry) throw new Error("tool_not_found");
  const executionContext = toolExecutionContext(context);
  let timer;
  const raw = await Promise.race([
    Promise.resolve().then(() => entry.execute(arguments_ ?? {}, executionContext)),
    new Promise((_, reject) => {
      timer = setTimeout(
        () => reject(new Error("tool_timeout")),
        TIMEOUTS.toolCallTimeoutMs,
      );
      timer.unref();
    }),
  ]).finally(() => clearTimeout(timer));
  if (typeof raw === "string") {
    return boundedToolResult({ content: raw, isError: false });
  }
  if (!raw || typeof raw !== "object" || typeof raw.content !== "string") {
    throw new Error("invalid_tool_result");
  }
  return boundedToolResult({
    content: raw.content,
    isError: raw.isError === true,
    displaySummary:
      typeof raw.displaySummary === "string" ? raw.displaySummary : undefined,
    truncated: raw.truncated === true,
  });
}

function toolExecutionContext(context) {
  const workingDirectory = context?.workingDirectory;
  if (
    typeof workingDirectory !== "string"
    || Array.from(workingDirectory).length === 0
    || Array.from(workingDirectory).length > LIMITS.maxWorkingDirectoryChars
    || workingDirectory.includes("\0")
  ) {
    throw new Error("invalid_tool_context");
  }
  return Object.freeze({ workingDirectory });
}

function boundedToolResult(result) {
  const content = safeSlice(result.content, LIMITS.maxMessageBytes);
  const displaySummary = typeof result.displaySummary === "string"
    ? safeSlice(result.displaySummary, 1_024)
    : undefined;
  return {
    content,
    isError: result.isError,
    displaySummary,
    truncated: result.truncated === true || content.length < result.content.length,
  };
}

function safeSlice(value, maxChars) {
  if (value.length <= maxChars) return value;
  const end = isHighSurrogate(value.charCodeAt(maxChars - 1))
    && isLowSurrogate(value.charCodeAt(maxChars))
    ? maxChars - 1
    : maxChars;
  return value.slice(0, end);
}

function isHighSurrogate(value) {
  return value >= 0xd800 && value <= 0xdbff;
}

function isLowSurrogate(value) {
  return value >= 0xdc00 && value <= 0xdfff;
}

export async function emitExtensionEvent(event, payload) {
  if (!supportsEvent(event)) throw new Error("invalid_event_name");
  for (const extension of extensions.values()) {
    try {
      await extension.context.emit(event, payload);
    } catch {
      // One extension cannot prevent delivery to the others.
    }
  }
  return { delivered: extensions.size };
}

export async function callExtensionUiAction(params) {
  return invokeUiAction(extensions.get(params?.extensionId), params);
}

export async function loadExtension(specification) {
  return loadExtensionWithApi(specification, createExtensionApi);
}

export async function loadExtensionWithApi(specification, createApi) {
  let stage = LOAD_STAGES[0];
  try {
    if (!specification || typeof specification !== "object") {
      throw new Error("invalid_extension_specification");
    }
    notifyCore(HOST_LOAD_STAGE_METHOD, { stage });
    const context = createApi(specification);
    const module = await jiti.import(specification.mainPath, { default: true });
    stage = LOAD_STAGES[1];
    notifyCore(HOST_LOAD_STAGE_METHOD, { stage });
    const activate =
      typeof module === "function"
        ? module
        : typeof module?.activate === "function"
          ? module.activate.bind(module)
          : null;
    if (!activate) throw new Error("activate_missing");
    await activate(context.api);
    stage = LOAD_STAGES[2];
    notifyCore(HOST_LOAD_STAGE_METHOD, { stage });
    ensureUniqueTools(context.tools);
    for (const tool of context.tools) {
      tools.set(tool.metadata.name, {
        extensionId: specification.id,
        execute: tool.execute,
      });
    }
    extensions.set(specification.id, { module, context });
    return {
      id: specification.id,
      contributions: {
        tools: context.tools.map((tool) => tool.metadata),
        skills: context.skills,
        resources: context.resources,
        events: [...context.events.keys()],
        ui: context.ui.contributions,
      },
      uiDiagnostics: context.ui.diagnostics,
    };
  } catch (error) {
    return {
      id: specification.id,
      error: "load_failed",
      diagnostic: createDiagnostic(error, stage, specification.mainPath),
    };
  }
}

function ensureUniqueTools(newTools) {
  if (tools.size + newTools.length > LIMITS.maxTools) {
    throw new Error("too_many_tools");
  }
  const names = new Set();
  for (const tool of newTools) {
    if (names.has(tool.metadata.name) || tools.has(tool.metadata.name)) {
      throw new Error("tool_name_conflict");
    }
    names.add(tool.metadata.name);
  }
}

async function deactivateAll() {
  for (const extension of extensions.values()) {
    try {
      if (typeof extension.module?.deactivate === "function") {
        await extension.module.deactivate();
      }
    } catch {
      // Reload continues even when an extension cleanup fails.
    }
  }
  extensions.clear();
  tools.clear();
  clearUiActions();
}
