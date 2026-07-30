import { Check, GitBranch, Spinner } from "@/components/ui/icons";
import "./clone-git-branch-button.css";

interface CloneGitBranchButtonProps {
  state: "idle" | "loading" | "success";
  label: string;
  disabled?: boolean;
  onClick: () => void;
}

export function CloneGitBranchButton({ state, label, disabled, onClick }: CloneGitBranchButtonProps) {
  const Icon = state === "success" ? Check : state === "loading" ? Spinner : GitBranch;
  return (
    <button
      type="button"
      className={`btn btn-sm btn-secondary cgb-btn ${state === "loading" ? "cgb-loading" : ""}`}
      onClick={onClick}
      disabled={disabled || state !== "idle"}
    >
      <Icon size="var(--icon-sm)" />
      <span>{label}</span>
    </button>
  );
}
