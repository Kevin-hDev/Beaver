import type { ExtensionResourceType } from "./extension-contract.generated";

/** Future API-R0 metadata; registration stays unavailable until its capability ships. */
export interface ExtensionSkill {
  id: string;
  name: string;
  description: string;
  path: string;
}

/** Future API-R0 metadata; resource content is never accepted from an extension yet. */
export interface ExtensionResource {
  id: string;
  name: string;
  description: string;
  type: ExtensionResourceType;
  path: string;
}

/** The host, not an extension, will derive MIME information from an artifact path. */
export type ExtensionResultBlock =
  | { type: "text"; text: string }
  | { type: "file"; path: string; purpose: "artifact" | "preview"; displayName?: string };

