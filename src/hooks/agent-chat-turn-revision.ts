import { invoke } from "@tauri-apps/api/core";
import i18n from "@/i18n";
import { admissionErrorMessage } from "@/lib/admission-error";
import { showToast } from "@/lib/toast-emitter";

export async function replaceSessionMessage(
  sessionId: string,
  messageId: string,
  newContent: string,
): Promise<boolean> {
  try {
    await invoke("truncate_and_replace_at", {
      sessionId,
      input: { message_id: messageId, new_content: newContent },
    });
    return true;
  } catch (error) {
    showToast(admissionErrorMessage(error, i18n.t, "errors.sessionSaveFailed"), "error");
    return false;
  }
}
