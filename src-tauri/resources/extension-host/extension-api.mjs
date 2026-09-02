import { callCore } from "./protocol.mjs";
import {
  LIMITS,
  methodKind,
  methodLevel,
  supportsEffect,
  supportsEvent,
  TIMEOUTS,
} from "./contract.mjs";

const inFlightHandlers = new Set();
const IDENTIFIER = /^[a-zA-Z0-9](?:[a-zA-Z0-9._-]{0,94}[a-zA-Z0-9])?$/;

export function createExtensionApi(specification) {
  const tools = [];
  const handlers = new Map();
  let handlerCount = 0;

  function registerTool(definition, replacesCore = false) {
    if (
      tools.length >= LIMITS.maxToolsPerExtension
      || !definition
      || typeof definition.execute !== "function"
    ) {
      throw new Error("invalid_tool");
    }
    const requestedName = String(definition.name ?? "");
    const publicName = replacesCore
      ? requestedName
      : requestedName.startsWith(`${specification.id}.`)
        ? requestedName
        : `${specification.id}.${requestedName}`;
    const tool = {
      name: publicName,
      description: String(definition.description ?? ""),
      parameters: definition.parameters ?? { type: "object" },
      effect: supportsEffect(definition.effect) ? definition.effect : "unknown",
      replacesCore,
    };
    if (
      !IDENTIFIER.test(tool.name)
      || !tool.description.trim()
      || tool.description.length > 2_000
      || !tool.parameters
      || typeof tool.parameters !== "object"
      || Array.isArray(tool.parameters)
    ) {
      throw new Error("invalid_tool");
    }
    tools.push({ metadata: tool, execute: definition.execute });
  }

  function on(eventName, eventHandler) {
    if (
      typeof eventHandler !== "function"
      || handlerCount >= LIMITS.maxEventsPerExtension
    ) {
      throw new Error("invalid_event_handler");
    }
    const event = String(eventName);
    if (!IDENTIFIER.test(event) || !supportsEvent(event)) {
      throw new Error("invalid_event_name");
    }
    const current = handlers.get(event) ?? [];
    current.push(eventHandler);
    handlers.set(event, current);
    handlerCount += 1;
    let subscribed = true;
    return () => {
      if (!subscribed) return;
      subscribed = false;
      const next = (handlers.get(event) ?? []).filter((item) => item !== eventHandler);
      if (next.length === 0) handlers.delete(event);
      else handlers.set(event, next);
      handlerCount -= 1;
    };
  }

  const api = {
    id: specification.id,
    manifest: Object.freeze({ ...specification.manifest }),
    info: () => callCore("app.info"),
    registerTool: (definition) => registerTool(definition, false),
    on,
    call: (method, params = {}) => callAtLevel("stable", method, params),
    sessions: Object.freeze({
      list: () => callCore("sessions.list"),
      get: (sessionId) => callCore("sessions.get", { sessionId: String(sessionId) }),
    }),
    projects: Object.freeze({
      list: () => callCore("projects.list"),
    }),
    mcp: Object.freeze({
      listConnectors: () => callCore("mcp.connectors.list"),
      callTool: (connectorId, toolName, arguments_ = {}) =>
        callCore("mcp.tool.call", {
          connectorId: String(connectorId),
          toolName: String(toolName),
          arguments: arguments_,
        }),
    }),
    channels: Object.freeze({
      getConfig: () => callCore("channels.config.get"),
    }),
    secrets: Object.freeze({
      getProviderKey: (providerId) =>
        callCore("secrets.provider.get", { providerId: String(providerId) }),
      getMcpOAuthToken: (connectorId) =>
        callCore("secrets.mcp.oauth.get", { connectorId: String(connectorId) }),
      getMcpEnvValue: (connectorId, envKey) =>
        callCore("secrets.mcp.env.get", {
          connectorId: String(connectorId),
          envKey: String(envKey),
        }),
      getChannelToken: (channelId, accountId, kind = "default") =>
        callCore("secrets.channel.get", {
          channelId: String(channelId),
          accountId: String(accountId),
          kind: String(kind),
        }),
    }),
    unstable: Object.freeze({
      call: (method, params = {}) => {
        if (specification.manifest.apiLevel !== "advanced") {
          throw new Error("advanced_api_required");
        }
        return callAtLevel("advanced", method, params);
      },
      registerReplacement: (definition) => {
        if (specification.manifest.apiLevel !== "advanced") {
          throw new Error("advanced_api_required");
        }
        registerTool(definition, true);
      },
    }),
  };

  return {
    api: Object.freeze(api),
    tools,
    events: handlers,
    emit: async (event, payload) => {
      for (const eventHandler of handlers.get(event) ?? []) {
        await runEventHandler(eventHandler, payload);
      }
    },
  };
}

async function runEventHandler(handler, payload) {
  if (inFlightHandlers.size >= LIMITS.maxInFlightHandlers) {
    throw new Error("too_many_event_handlers_running");
  }
  const execution = Promise.resolve().then(() => handler(payload));
  inFlightHandlers.add(execution);
  void execution.finally(() => inFlightHandlers.delete(execution)).catch(() => {});
  let timer;
  try {
    await Promise.race([
      execution,
      new Promise((resolve) => {
        timer = setTimeout(resolve, TIMEOUTS.eventHandlerTimeoutMs);
        timer.unref();
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}

function callAtLevel(level, method, params) {
  const requested = String(method);
  if (methodLevel(requested) !== level || methodKind(requested) !== "request") {
    return Promise.reject(new Error("core_method_unavailable"));
  }
  return callCore(requested, params);
}
