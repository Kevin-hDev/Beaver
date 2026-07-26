import { ClipboardText } from "@/components/ui/icons";
import { ChatMarkdown } from "./chat-markdown";
import type { AgentPlanPreview } from "@/types/agent";
import "./plan-preview-bubble.css";

interface PlanPreviewBubbleProps {
  plan: AgentPlanPreview;
}

export function PlanPreviewBubble({ plan }: PlanPreviewBubbleProps) {
  return (
    <div className="chat-bubble chat-column-surface ppb-root">
      <div className="ppb-header">
        <ClipboardText size="var(--icon-md)" weight="regular" />
        <span className="ppb-title">{plan.title}</span>
      </div>
      <div className="ppb-content chat-md">
        <ChatMarkdown content={plan.content} />
      </div>
    </div>
  );
}
