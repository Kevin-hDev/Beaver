import type { TFunction } from "i18next";
import type { ToolErrorCategory, ToolErrorInfo } from "@/types/agent";
import { officeToolErrorMessage } from "./office-tool-errors";
import { sanitizeToolError } from "./tool-error-sanitize";
import { admissionErrorKey } from "./admission-error";

const CATEGORY_KEYS: Record<ToolErrorCategory, string> = {
  validation: "agentLocal.toolActivity.errorCategories.validation",
  permission: "agentLocal.toolActivity.errorCategories.permission",
  not_found: "agentLocal.toolActivity.errorCategories.notFound",
  conflict: "agentLocal.toolActivity.errorCategories.conflict",
  timeout: "agentLocal.toolActivity.errorCategories.timeout",
  cancelled: "agentLocal.toolActivity.errorCategories.cancelled",
  unavailable: "agentLocal.toolActivity.errorCategories.unavailable",
  external: "agentLocal.toolActivity.errorCategories.external",
  execution: "agentLocal.toolActivity.errorCategories.execution",
  internal: "agentLocal.toolActivity.errorCategories.internal",
};

const ERROR_CODE_KEYS: Readonly<Record<string, string>> = {
  web_search_runtime_unavailable: "agentLocal.toolActivity.webSearchRuntimeUnavailable",
};

export function toolErrorHasLocalizedMessage(error: ToolErrorInfo | undefined): boolean {
  return error ? error.code in ERROR_CODE_KEYS : false;
}

export function toolErrorResultIsMachineCode(result: string | undefined): boolean {
  return result !== undefined && /^searxng_[a-z_]+$/u.test(result);
}

export function toolErrorMessage(
  toolName: string,
  result: string,
  error: ToolErrorInfo | undefined,
  t: TFunction,
): string {
  const admissionKey = admissionErrorKey(result);
  if (admissionKey) return t(admissionKey);

  if (toolName.startsWith("beaver.office.")) {
    const officeMessage = officeToolErrorMessage(result, t);
    if (officeMessage) return officeMessage;
  }

  const errorCodeKey = error ? ERROR_CODE_KEYS[error.code] : undefined;
  if (errorCodeKey) return t(errorCodeKey);

  const categoryKey = error ? CATEGORY_KEYS[error.category] : undefined;
  if (categoryKey) return t(categoryKey);

  return sanitizeToolError(result) || t("errors.toolFailed");
}
