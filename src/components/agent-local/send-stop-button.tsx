import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import "./chat.css";

type ButtonState = "hidden" | "send" | "stop" | "confirmStop";

interface SendStopButtonProps {
  state: ButtonState;
  onSend: () => void;
  onStop: () => void;
}

function SendIcon() {
  return (
    <svg className="send-stop-icon" viewBox="0 0 30.2 30.1" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeMiterlimit="10">
      <path strokeWidth="1.25" d="M2.1,14.6C8.9,12,28.5,4,28.5,4l-3.9,22.6c-0.2,1.1-1.5,1.5-2.3,0.8l-6.1-5.1l-4.3,4l0.7-6.7l13-12.3l-16,10l1,5.7l-3.3-5.3l-5-1.6C1.5,15.8,1.4,14.8,2.1,14.6z" />
    </svg>
  );
}

function StopIcon() {
  return (
    <svg className="send-stop-icon" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="256" cy="256" r="240" stroke="currentColor" strokeWidth="32" />
      <rect x="154" y="154" width="204.8" height="204.8" rx="40" fill="currentColor" />
    </svg>
  );
}

function ConfirmStopIcon() {
  return (
    <svg className="send-stop-icon" viewBox="0 0 512 512" fill="none" xmlns="http://www.w3.org/2000/svg">
      <circle cx="256" cy="256" r="240" stroke="currentColor" strokeWidth="32" />
      <text x="256" y="266" className="send-stop-confirm-text" textAnchor="middle" dominantBaseline="middle">ESC</text>
    </svg>
  );
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
        className={`icon-btn send-btn ${isStop ? "stop" : "send"}${state === "confirmStop" ? " confirm-stop" : ""}`}
        onClick={isStop ? onStop : onSend}
        disabled={disabled}
      >
        {state === "confirmStop" ? <ConfirmStopIcon /> : isStop ? <StopIcon /> : <SendIcon />}
      </button>
    </Tooltip>
  );
}
