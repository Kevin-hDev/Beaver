export function createContextualApis(capabilities, callAtLevel, callMemoryWrite) {
  return Object.freeze({
    models: capabilities.includes("models")
      ? Object.freeze({
          list: (options = {}) => callAtLevel("stable", "models.list", options),
          generate: (options) => callAtLevel("stable", "models.generate", options),
        })
      : undefined,
    memory: capabilities.includes("memory")
      ? Object.freeze({
          list: (options) => callAtLevel("stable", "memory.list", options),
          read: (options) => callAtLevel("stable", "memory.read", options),
          write: callMemoryWrite,
          archive: (options) => callAtLevel("stable", "memory.archive", options),
        })
      : undefined,
    automations: capabilities.includes("automations")
      ? Object.freeze({
          list: (options = {}) => callAtLevel("stable", "automations.list", options),
          create: (options) => callAtLevel("stable", "automations.create", options),
          update: (automationId, revision, patch) => callAtLevel(
            "stable", "automations.update", { ...patch, automationId: String(automationId), revision },
          ),
          setActive: (automationId, revision, active) => callAtLevel(
            "stable", "automations.setActive", { automationId: String(automationId), revision, active },
          ),
          delete: (automationId, revision) => callAtLevel(
            "stable", "automations.delete", { automationId: String(automationId), revision },
          ),
        })
      : undefined,
    subagents: capabilities.includes("subagents")
      ? Object.freeze({
          spawn: (type, prompt) => callAtLevel("stable", "subagents.spawn", { type, prompt }),
          list: (options = {}) => callAtLevel("stable", "subagents.list", options),
          get: (subagentId) => callAtLevel("stable", "subagents.get", { subagentId }),
          send: (subagentId, prompt) => callAtLevel(
            "stable", "subagents.send", { subagentId, prompt },
          ),
          cancel: (subagentId) => callAtLevel("stable", "subagents.cancel", { subagentId }),
        })
      : undefined,
  });
}
