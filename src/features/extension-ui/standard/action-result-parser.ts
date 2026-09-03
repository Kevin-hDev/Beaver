import { exact, invalid, jsonBytes, localized, plain } from "./parse-utils";
import { parseStandardView } from "./view-parser";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import type { StandardActionResult } from "./types";

const LEVELS = ["info", "success", "warning", "error"] as const;

export function parseStandardActionResult(
  extensionId: string,
  value: unknown,
): StandardActionResult {
  if (jsonBytes(value) > UI_LIMITS.maxActionResultBytes) throw invalid();
  const input = plain(value);
  if (input.type === "notification") {
    exact(input, ["type", "level", "message"]);
    const level = LEVELS.find((candidate) => candidate === input.level);
    if (!level) throw invalid();
    return {
      type: "notification",
      level,
      message: localized(input.message),
    };
  }
  if (input.type === "view") {
    exact(input, ["type", "view"]);
    return { type: "view", view: parseStandardView(extensionId, input.view) };
  }
  throw invalid();
}
