import { createJiti } from "jiti";
import { fileURLToPath } from "node:url";
import { LIMITS } from "./contract.mjs";
import { createExtensionApi } from "./extension-api.mjs";
import { createDiagnostic } from "./diagnostics.mjs";

const sdkPath = fileURLToPath(new URL("./sdk/index.mjs", import.meta.url));
const jiti = createJiti(import.meta.url, {
  alias: { "@beaver/sdk": sdkPath },
  fsCache: false,
  interopDefault: true,
  moduleCache: false,
  sourceMaps: false,
});
const TOOL_TIMEOUT_MS = 55_000;
const extensions = new Map();
const tools = new Map();

export async function syncExtensions(specifications) {
  await deactivateAll();
  const loaded = [];
  for (const specification of specifications) {
    loaded.push(await loadExtension(specification));
  }
  return { extensions: loaded };
}

export async function callExtensionTool(name, arguments_, context) {
  const entry = tools.get(name);
  if (!entry) throw new Error("tool_not_found");
  const executionContext = toolExecutionContext(context);
  let timer;
  const raw = await Promise.race([
    Promise.resolve().then(() => entry.execute(arguments_ ?? {}, executionContext)),
    new Promise((_, reject) => {
      timer = setTimeout(() => reject(new Error("tool_timeout")), TOOL_TIMEOUT_MS);
      timer.unref();
    }),
  ]).finally(() => clearTimeout(timer));
  if (typeof raw === "string") {
    return { content: raw, isError: false };
  }
  if (!raw || typeof raw !== "object" || typeof raw.content !== "string") {
    throw new Error("invalid_tool_result");
  }
  return {
    content: raw.content,
    isError: raw.isError === true,
    displaySummary:
      typeof raw.displaySummary === "string" ? raw.displaySummary : undefined,
  };
}

function toolExecutionContext(context) {
  const workingDirectory = context?.workingDirectory;
  if (
    typeof workingDirectory !== "string"
    || workingDirectory.length === 0
    || workingDirectory.length > 4_096
    || workingDirectory.includes("\0")
  ) {
    throw new Error("invalid_tool_context");
  }
  return Object.freeze({ workingDirectory });
}

export async function emitExtensionEvent(event, payload) {
  for (const extension of extensions.values()) {
    try {
      await extension.context.emit(event, payload);
    } catch {
      // One extension cannot prevent delivery to the others.
    }
  }
  return { delivered: extensions.size };
}

async function loadExtension(specification) {
  let stage = "import";
  try {
    const context = createExtensionApi(specification);
    const module = await jiti.import(specification.mainPath, { default: true });
    stage = "activate";
    const activate =
      typeof module === "function"
        ? module
        : typeof module?.activate === "function"
          ? module.activate.bind(module)
          : null;
    if (!activate) throw new Error("activate_missing");
    await activate(context.api);
    stage = "register";
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
        events: [...context.events.keys()],
      },
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
}
