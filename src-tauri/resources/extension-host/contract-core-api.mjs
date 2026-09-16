export function parseCoreApiMethods(hostMethods, optionalCapabilities, effects, limits) {
  const capabilities = new Set(optionalCapabilities);
  const allowedEffects = new Set(effects);
  const metadata = {};
  for (const method of hostMethods) {
    if (method.kind === "request" && typeof method.idempotent !== "boolean") {
      throw new Error("invalid_extension_contract");
    }
    if (method.capability === undefined) continue;
    const methodKeys = [
      "name", "level", "kind", "rustBudgetMs", "capability", "requiresContext",
      "idempotent", "effects", "params", "result",
    ];
    if (
      Object.keys(method).some((key) => !methodKeys.includes(key))
      || !capabilities.has(method.capability)
      || method.kind !== "request"
      || method.level !== "stable"
      || typeof method.requiresContext !== "boolean"
      || !Array.isArray(method.effects)
      || method.effects.length < 1
      || new Set(method.effects).size !== method.effects.length
      || method.effects.some((effect) => !allowedEffects.has(effect))
      || !Array.isArray(method.params)
      || method.params.length > 16
      || typeof method.result !== "string"
    ) throw new Error("invalid_extension_contract");
    const names = new Set();
    for (const param of method.params) {
      if (
        !param
        || Object.keys(param).some((key) => !["name", "type", "required", "limit"].includes(key))
        || typeof param.name !== "string"
        || param.name.length < 1
        || param.name.length > 64
        || names.has(param.name)
        || !["string", "integer", "boolean", "object", "memoryScope", "subagentType"].includes(param.type)
        || typeof param.required !== "boolean"
        || (param.limit !== undefined && !Number.isSafeInteger(limits[param.limit]))
      ) throw new Error("invalid_extension_contract");
      names.add(param.name);
    }
    metadata[method.name] = Object.freeze({
      capability: method.capability,
      requiresContext: method.requiresContext,
      idempotent: method.idempotent,
      effects: Object.freeze([...method.effects]),
      params: Object.freeze(method.params.map((param) => Object.freeze({ ...param }))),
      result: method.result,
    });
  }
  return Object.freeze(metadata);
}
