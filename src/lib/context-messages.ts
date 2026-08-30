import type { AgentMessage } from "@/types/agent";

const CLONE_SUMMARY_PREFIX = "This cloned session includes hidden branch context:";
const SUBAGENT_REPORT_CONTEXT_PREFIX = "Subagent report context:";
const SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX = "Subagent orchestration context:";

export function isCompressionSummaryMessage(message: AgentMessage): boolean {
  return message.message_kind === "compression_checkpoint";
}

export function isCompressionContextOnlyMessage(message: AgentMessage): boolean {
  return isCompressionSummaryMessage(message)
    || message.message_kind === "compression_boundary"
    || isCloneSummaryContextMessage(message)
    || message.content.trimStart().startsWith(SUBAGENT_REPORT_CONTEXT_PREFIX)
    || message.content.trimStart().startsWith(SUBAGENT_ORCHESTRATION_CONTEXT_PREFIX);
}

export function isCloneSummaryContextMessage(message: AgentMessage): boolean {
  return message.content.trimStart().startsWith(CLONE_SUMMARY_PREFIX);
}
