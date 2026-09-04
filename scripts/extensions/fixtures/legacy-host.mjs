export function createLegacyHostContext(specification) {
  const tools = [];
  return {
    api: Object.freeze({
      id: specification.id,
      manifest: Object.freeze({ ...specification.manifest }),
      info: async () => ({}),
      registerTool(definition) {
        tools.push({
          metadata: {
            name: `${specification.id}.${definition.name}`,
            description: definition.description,
            parameters: definition.parameters,
            effect: definition.effect ?? "unknown",
            replacesCore: false,
          },
          execute: definition.execute,
        });
      },
      on() { return () => {}; },
    }),
    tools,
    skills: [],
    resources: [],
    events: new Map(),
    ui: { contributions: [], diagnostics: [] },
  };
}
