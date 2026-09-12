import type { ChatState } from "./agent-chat-stream-types";
import { useContextProgress } from "./use-context-progress";
import { useContextUsage } from "./use-context-usage";

interface ChatContextArgs {
  sessionId: string;
  model: string;
  provider: string;
  chat: ChatState;
  workingDir?: string;
  permissionMode: string;
  supportsTools?: boolean;
  includesReasoning?: boolean;
}

export function useChatContext(args: ChatContextArgs) {
  const progress = useContextProgress(
    args.model, args.chat.sessionTokenCount, args.provider, args.chat.contextUsageRecord,
  );
  const breakdown = useContextUsage({
    sessionId: args.sessionId,
    model: args.model,
    provider: args.provider,
    messages: args.chat.messages,
    stream: args.chat,
    workingDir: args.workingDir,
    permissionMode: args.permissionMode,
    planMode: args.chat.planModeEnabled,
    supportsTools: args.supportsTools,
    contextUsageIncludesReasoning: args.includesReasoning,
  });
  return { breakdown, summary: progress.summary, max: progress.summary?.max ?? 0 };
}
