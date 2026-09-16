import { CAPABILITIES, OPTIONAL_CAPABILITIES } from "./contract.mjs";

const LEGACY_IMPLEMENTED = Object.freeze(
  OPTIONAL_CAPABILITIES.filter((capability) =>
    ["skills", "resources", "richToolResults", "models", "memory", "automations"].includes(capability)),
);
let active = Object.freeze([...CAPABILITIES, ...LEGACY_IMPLEMENTED]);

export function negotiateCapabilities(requested) {
  const accepted = Array.isArray(requested)
    ? LEGACY_IMPLEMENTED.filter((capability) => requested.includes(capability))
    : LEGACY_IMPLEMENTED;
  active = Object.freeze([...CAPABILITIES, ...accepted]);
  return active;
}

export function activeCapabilities() {
  return active;
}
