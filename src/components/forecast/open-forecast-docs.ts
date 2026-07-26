import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { cleanupTauriListener } from "@/lib/tauri-listen";

const DOCS_WINDOW_LABEL = "forecast-docs";

export async function openForecastDocsWindow(title: string) {
  const existing = await WebviewWindow.getByLabel(DOCS_WINDOW_LABEL);
  if (existing) {
    await existing.setFocus();
    return;
  }

  const docsWindow = new WebviewWindow(DOCS_WINDOW_LABEL, {
    url: "/#/forecast-docs",
    title,
    width: 1180,
    height: 820,
    minWidth: 960,
    minHeight: 640,
    decorations: true,
    transparent: false,
  });

  await new Promise<void>((resolve, reject) => {
    const removeCreated = docsWindow.once("tauri://created", () => {
      cleanupTauriListener(removeError);
      resolve();
    });
    const removeError = docsWindow.once<string>("tauri://error", (event) => {
      cleanupTauriListener(removeCreated);
      reject(new Error(event.payload));
    });
  });
}
