import { callCore } from "./protocol.mjs";
import {
  LIMITS,
  CORE_API_METHODS,
  methodKind,
  methodLevel,
  RESOURCE_TYPES,
  supportsEffect,
} from "./contract.mjs";
import { validateCoreApiParams } from "./core-api-validation.mjs";
import { createUiApi } from "./ui-api.mjs";
import { createEventHandlers } from "./event-handlers.mjs";
import { activeCapabilities } from "./extension-api-capabilities.mjs";
import { snapshotContribution } from "./contribution-snapshot.mjs";
import {
  unicodeScalarLength,
  validContribution,
  validIdentifier,
  validRelativePath,
} from "./contribution-validation.mjs";

export function createExtensionApi(specification) {
  const tools = [];
  const skills = [];
  const resources = [];
  const eventHandlers = createEventHandlers();
  const ui = createUiApi(specification);

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
      // Rust revalidates this value; normalization here only keeps host output stable.
      effect: supportsEffect(definition.effect) ? definition.effect : "unknown",
      replacesCore,
    };
    if (
      !validIdentifier(tool.name)
      || !tool.description.trim()
      || unicodeScalarLength(tool.description) > LIMITS.maxExtensionTextChars
      || !tool.parameters
      || typeof tool.parameters !== "object"
      || Array.isArray(tool.parameters)
    ) {
      throw new Error("invalid_tool");
    }
    tools.push({ metadata: tool, execute: definition.execute });
  }

  function registerSkill(definition) {
    const skill = snapshotContribution(definition);
    if (
      skills.length >= LIMITS.maxSkillsPerExtension
      || !validContribution(skill, ["id", "name", "description", "path"])
      || !validRelativePath(skill.path)
      || !["SKILL.md", "skill.md"].includes(skill.path.split("/").at(-1))
      || skills.some((item) => item.id === skill.id)
    ) throw new Error("invalid_skill");
    skills.push(skill);
  }

  function registerResource(definition) {
    const resource = snapshotContribution(definition);
    if (
      resources.length >= LIMITS.maxResourcesPerExtension
      || !validContribution(resource, ["id", "name", "description", "type", "path"])
      || !RESOURCE_TYPES.includes(resource.type)
      || !validRelativePath(resource.path)
      || resources.some((item) => item.id === resource.id)
    ) throw new Error("invalid_resource");
    resources.push(resource);
  }

  const api = {
    id: specification.id,
    manifest: Object.freeze({ ...specification.manifest }),
    capabilities: Object.freeze([...activeCapabilities()]),
    info: () => callCore("app.info"),
    registerTool: (definition) => registerTool(definition, false),
    registerSkill,
    registerResource,
    ui: ui.api,
    on: eventHandlers.on,
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
    skills,
    resources,
    events: eventHandlers.events,
    ui,
    emit: eventHandlers.emit,
  };
}

function callAtLevel(level, method, params) {
  const requested = String(method);
  if (methodLevel(requested) !== level || methodKind(requested) !== "request") {
    return Promise.reject(new Error("core_method_unavailable"));
  }
  const coreMethod = CORE_API_METHODS[requested];
  if (coreMethod && !activeCapabilities().includes(coreMethod.capability)) {
    return Promise.reject(new Error("core_method_unavailable"));
  }
  return callCore(
    requested,
    coreMethod ? validateCoreApiParams(requested, params) : params,
  );
}
