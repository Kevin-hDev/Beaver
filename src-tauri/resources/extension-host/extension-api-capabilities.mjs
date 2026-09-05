import { CAPABILITIES, OPTIONAL_CAPABILITIES } from "./contract.mjs";

export const ACTIVE_CAPABILITIES = Object.freeze([
  ...CAPABILITIES,
  ...OPTIONAL_CAPABILITIES,
]);
