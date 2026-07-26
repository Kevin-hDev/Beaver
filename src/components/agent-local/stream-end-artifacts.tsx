import { FileChangeBubble } from "./file-change-bubble";
import { SubagentBubble } from "./subagent-bubble";
import { collectFileOperations } from "@/lib/file-preview-utils";
import { collectMessagesSubagents } from "@/lib/message-subagents";
import type { AgentMessage, SubagentInfo } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

interface StreamEndArtifactsProps {
  messages: AgentMessage[];
  projectPath?: string;
  knownSubagents: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
  onFileReview?: (operation: FileOperation) => void;
}

export function StreamEndArtifacts({
  messages,
  projectPath,
  knownSubagents,
  onOpenSubagent,
  onFileReview,
}: StreamEndArtifactsProps) {
  const subagents = collectMessagesSubagents(messages, knownSubagents);
  const files = collectFileOperations(messages, { baseDir: projectPath });
  return (
    <>
      <SubagentBubble subagents={subagents} onOpen={(id) => onOpenSubagent?.(id)} />
      <FileChangeBubble operations={files} baseDir={projectPath} onReview={onFileReview} />
    </>
  );
}
