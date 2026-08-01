import type { TFunction } from "i18next";
import type { ToolErrorCategory, ToolErrorInfo } from "@/types/agent";
import { extensionErrorKey } from "./extension-errors";
import { officeToolErrorMessage } from "./office-tool-errors";
import { sanitizeToolError } from "./tool-error-sanitize";

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

export function toolErrorMessage(
  toolName: string,
  result: string,
  error: ToolErrorInfo | undefined,
  t: TFunction,
): string {
  if (toolName.startsWith("beaver.office.")) {
    const officeMessage = officeToolErrorMessage(result, t);
    if (officeMessage) return officeMessage;
  }

  const extensionKey = extensionErrorKey(error?.code ?? "", "");
  if (extensionKey) return t(extensionKey);

  const categoryKey = error ? CATEGORY_KEYS[error.category] : undefined;
  if (categoryKey) return t(categoryKey);

  return sanitizeToolError(result) || t("errors.toolFailed");
}
