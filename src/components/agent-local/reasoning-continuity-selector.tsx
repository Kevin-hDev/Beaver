import { useTranslation } from "react-i18next";
import type {
  ContinuityCapability,
  PreserveReasoningSetting,
} from "@/types/agent-session.generated";
import { cn } from "@/lib/utils";
import "./reasoning-continuity-selector.css";

interface ReasoningContinuitySelectorProps {
  capability?: ContinuityCapability;
  setting: PreserveReasoningSetting;
  onChange: (setting: PreserveReasoningSetting) => void;
}

const labels: Record<PreserveReasoningSetting, string> = {
  off: "agentLocal.continuityOff",
  local: "agentLocal.continuityLocal",
  remote: "agentLocal.continuityRemote",
};

export function ReasoningContinuitySelector({
  capability,
  setting,
  onChange,
}: ReasoningContinuitySelectorProps) {
  const { t } = useTranslation();
  if (!capability || capability.state === "disabled") return null;

  const locked = capability.state === "locked";
  const options = [
    capability.requirement === "optional" ? "off" : null,
    capability.local_available ? "local" : null,
    capability.remote_available ? "remote" : null,
  ].filter((option): option is PreserveReasoningSetting => option !== null);

  return (
    <fieldset className="rcs-root" aria-label={t("agentLocal.continuityTitle")}>
      <legend className="sr-only">{t("agentLocal.continuityTitle")}</legend>
      <div className="rcs-options" role="radiogroup" aria-label={t("agentLocal.continuityTitle")}>
        {options.map((option) => (
          <label
            className={cn("rcs-option", setting === option && "rcs-option-active")}
            key={option}
          >
            <input
              checked={setting === option}
              disabled={locked}
              name="reasoning-continuity"
              onChange={() => onChange(option)}
              type="radio"
              value={option}
            />
            <span>{t(labels[option])}</span>
          </label>
        ))}
      </div>
      <p className="rcs-explanation" role={locked ? "status" : undefined}>
        {t(capability.explanation_key)}
      </p>
    </fieldset>
  );
}
