import extensionContract from "../../src-tauri/resources/extension-host/contract.json";

export type ExtensionInstallSource = "git" | "npm";

export interface ExtensionInstallOutcome {
  jobId: string | null;
  errorKey: string | null;
}

export const EXTENSION_INSTALL_LIMITS = Object.freeze({
  git: extensionContract.limits.maxGitLocatorChars,
  npm: extensionContract.limits.maxNpmSpecChars,
});
