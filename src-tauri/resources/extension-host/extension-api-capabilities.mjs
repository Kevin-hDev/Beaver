import { CAPABILITIES, OPTIONAL_CAPABILITIES } from "./contract.mjs";

// richToolResults remains unavailable until its end-to-end result path exists.
export const ACTIVE_CAPABILITIES = Object.freeze([
  ...CAPABILITIES,
  ...OPTIONAL_CAPABILITIES.filter((capability) => capability !== "richToolResults"),
]);
