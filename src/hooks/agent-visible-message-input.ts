import type { AgentMessage, VisibleMessageInput } from "@/types/agent";

/** Shim visible temporaire, supprimé avec add_messages_to_session en Task 9 Step 4. */
export function toVisibleMessageInput(message: AgentMessage): VisibleMessageInput {
  return {
    id: message.id,
    role: message.role,
    content: message.content,
    thinking: message.thinking,
    tool_calls: message.tool_calls?.map((call) => ({
      id: call.id ?? "",
      function: {
        name: call.function.name,
        arguments: call.function.arguments,
      },
    })),
    tool_name: message.tool_name,
    tool_call_id: message.tool_call_id,
    tool_activities: message.tool_activities,
    segments: message.segments,
    files: message.files,
    timestamp: message.timestamp,
    tokens: message.tokens ?? 0,
    work_duration_ms: message.work_duration_ms,
    skill_names: message.skill_names,
    stream_run_id: message.stream_run_id,
    stream_part: message.stream_part,
  };
}
