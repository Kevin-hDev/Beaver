import { AgentImportWizard } from "@/components/agent-import/agent-import-wizard";

interface OnboardingAgentImportProps {
  onNext: () => void;
  onBack: () => void;
}

export function OnboardingAgentImport({
  onNext,
  onBack,
}: OnboardingAgentImportProps) {
  return (
    <div className="ob-page">
      <div className="aim-onboarding-shell">
        <AgentImportWizard onContinue={onNext} onBack={onBack} />
      </div>
    </div>
  );
}
