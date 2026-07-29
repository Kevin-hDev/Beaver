import type { AgentInteractiveChoiceRequest } from "@/types/agent";
import { InteractiveChoicePanelInner } from "./interactive-choice-panel-inner";
import { PlanApprovalPanel } from "./plan-approval-panel";

interface InteractiveChoicePanelProps {
  request?: AgentInteractiveChoiceRequest;
  onResolved?: () => void;
  onError?: () => void;
}

export function InteractiveChoicePanel({
  request,
  onResolved,
  onError,
}: InteractiveChoicePanelProps) {
  if (!request) return null;
  if (request.kind === "plan_approval") {
    return (
      <PlanApprovalPanel
        key={request.id}
        request={request}
        onResolved={onResolved}
        onError={onError}
      />
    );
  }
  return (
    <InteractiveChoicePanelInner
      key={request.id}
      request={request}
      onResolved={onResolved}
      onError={onError}
    />
  );
}
