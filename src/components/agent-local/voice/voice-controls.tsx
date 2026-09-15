import { useTranslation } from "react-i18next";
import { Tooltip } from "@/components/ui/tooltip";
import { OperationProgressAction } from "@/components/ui/operation-progress-action";
import { X } from "@/components/ui/icons";
import { useVoiceController } from "@/features/voice/use-voice-controller";
import { VoiceSignal } from "./voice-signal";
import { VoiceFirstUseDialog } from "./voice-first-use-dialog";
import { VoiceLanguageDialog } from "./voice-language-dialog";
import "./voice-controls.css";

export function VoiceControls({ draftKey }: { draftKey: string }) {
  const { t } = useTranslation();
  const voice = useVoiceController(draftKey);
  if (!voice.available && !voice.origin && !voice.activeElsewhere) return null;
  const operation = voice.snapshot?.operation;
  return (
    <>
      {voice.modelDownload?.status === "suspended" ? (
        <button type="button" className="btn btn-sm btn-secondary" onClick={() => void voice.resumeDownload(voice.modelDownload!.id)}>{t("modelDownloads.resume")}</button>
      ) : voice.modelDownload ? (
        <OperationProgressAction compact percent={voice.modelDownload.status === "queued" ? null : voice.modelDownload.percent}
          phaseLabel={t(voice.modelDownload.status === "queued" ? "modelDownloads.queued" : "voice.settings.installing")}
          cancelling={voice.modelDownload.status === "cancelling"} canCancel
          cancelLabel={t("common.cancel")} cancellingLabel={t("voice.settings.cancelling")}
          onCancel={() => void voice.cancelDownload(voice.modelDownload!.id)} />
      ) : voice.origin && operation ? (
        <div className="vc-active">
          {voice.snapshot?.phase === "listening" && <VoiceSignal level={operation.level} />}
          <button type="button" className="icon-btn vc-cancel" aria-label={t("voice.cancel")} onClick={() => void voice.cancel()}><X size="var(--icon-sm)" /></button>
          {voice.snapshot?.phase === "listening" && (
            <button type="button" className="icon-btn vc-validate" aria-label={t("voice.validate")} onClick={() => void voice.validate()}><span /></button>
          )}
        </div>
      ) : voice.activeElsewhere ? (
        <button type="button" className="vc-elsewhere" onClick={() => {
          const destination = voice.snapshot?.operation?.destination;
          if (destination?.kind === "draft") window.dispatchEvent(new CustomEvent("beaver:voice-go-to-draft", { detail: destination.draft_key }));
        }}>{t("voice.activeElsewhere")}</button>
      ) : (
        <Tooltip label={t("voice.start")} align="right">
          <button type="button" className="icon-btn vc-mic" aria-label={t("voice.start")} disabled={voice.pending} onClick={voice.begin}><span /></button>
        </Tooltip>
      )}
      {voice.dialog === "first-use" && <VoiceFirstUseDialog onAccept={() => void voice.acceptExplanation()} onClose={voice.closeDialog} />}
      {voice.dialog === "language" && <VoiceLanguageDialog onStart={(language) => void voice.start(language)} onClose={voice.closeDialog} />}
    </>
  );
}
