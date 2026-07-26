import type { AgentMessage } from "@/types/agent";

const COMPRESSION_SUMMARY_PREFIX =
  "This session is being continued from a previous conversation";
const RECENT_FILE_CONTEXT_PREFIX = "Recent file context preserved across compression:";
const CLONE_SUMMARY_PREFIX = "This cloned session includes hidden branch context:";
const SUBAGENT_REPORT_CONTEXT_PREFIX = "Subagent report context:";
const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX = "Subagent orchestration context:";

export function isCompressionSummaryMessage(message: AgentMessage): boolean {
  return message.content.trimStart().startsWith(COMPRESSION_SUMMARY_PREFIX);
}

export function isCompressionContextOnlyMessage(message: AgentMessage): boolean {
  const content = message.content.trimStart();
  return isCompressionSummaryMessage(message)
    || isCloneSummaryContextMessage(message)
    || content.startsWith(SUBAGENT_REPORT_CONTEXT_PREFIX)
    || content.startsWith(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX)
    || content.startsWith(RECENT_FILE_CONTEXT_PREFIX);
}

export function isCloneSummaryContextMessage(message: AgentMessage): boolean {
  return message.content.trimStart().startsWith(CLONE_SUMMARY_PREFIX);
}
