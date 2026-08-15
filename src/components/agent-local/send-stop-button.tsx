import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { SendIcon, StopIcon, ConfirmStopIcon } from "./send-stop-icons";
import "./chat.css";

type ButtonState = "hidden" | "send" | "stop" | "confirmStop";

interface SendStopButtonProps {
  state: ButtonState;
  onSend: () => void;
  onStop: () => void;
}

export function SendStopButton({ state, onSend, onStop }: SendStopButtonProps) {
  const { t } = useTranslation();
  const isStop = state === "stop" || state === "confirmStop";
  const disabled = state === "hidden";
  return (
    <Tooltip label={isStop ? t("agentLocal.stop") : t("agentLocal.send")} align="right">
      <button
        type="button"
        aria-label={isStop ? t("agentLocal.stop") : t("agentLocal.send")}
        className="icon-btn send-btn"
        data-state={state}
        onClick={isStop ? onStop : onSend}
        disabled={disabled}
      >
        {state === "confirmStop" ? <ConfirmStopIcon /> : isStop ? <StopIcon /> : <SendIcon />}
      </button>
    </Tooltip>
  );
}
