function copy(value) {
  if (Array.isArray(value)) return Object.freeze(value.map(copy));
  if (value && typeof value === "object") {
    const result = Object.create(null);
    for (const [key, item] of Object.entries(value)) result[key] = copy(item);
    return Object.freeze(result);
  }
  if (["string", "number", "boolean"].includes(typeof value) || value === null) return value;
  throw new Error("invalid_contribution");
}

export function snapshotContribution(value) {
  let serialized;
  try {
    serialized = JSON.stringify(value);
  } catch {
    throw new Error("invalid_contribution");
  }
  if (typeof serialized !== "string") throw new Error("invalid_contribution");
  return copy(JSON.parse(serialized));
}
