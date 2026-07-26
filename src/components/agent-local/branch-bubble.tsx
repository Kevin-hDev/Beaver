import { GitBranch } from "@/components/ui/icons";
import { useTranslation } from "react-i18next";
import "./branch-bubble.css";

interface BranchBubbleProps {
  action: "created" | "switched";
  branchName: string;
  path?: string;
}

export function BranchBubble({ action, branchName, path }: BranchBubbleProps) {
  const { t } = useTranslation();
  const label = action === "created"
    ? t("branches.bubbleCreated")
    : t("branches.bubbleSwitched");

  return (
    <div className="chat-bubble chat-column-surface">
      <div className="bb-content">
        <GitBranch size="var(--icon-md)" className="bb-icon" />
        <span className="bb-label">{label}</span>
        <span className="bb-branch-name">{branchName}</span>
      </div>
      {path && (
        <div className="bb-content">
          <span className="bb-path">{path}</span>
        </div>
      )}
    </div>
  );
}
