import { Channel, invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

import type { InstallerEvent, InstallerSnapshot } from "./installer-contract.generated";

export interface InstallerApi {
  snapshot: () => Promise<InstallerSnapshot>;
  chooseDirectory: () => Promise<string | null>;
  start: (onEvent: (event: InstallerEvent) => void) => Promise<void>;
  cancel: () => Promise<InstallerSnapshot>;
  launch: () => Promise<void>;
  close: () => Promise<void>;
}

export const installerApi: InstallerApi = {
  snapshot: () => invoke("installer_snapshot"),
  chooseDirectory: () => invoke("choose_install_directory"),
  start: async (onEvent) => {
    const channel = new Channel<InstallerEvent>();
    channel.onmessage = onEvent;
    await invoke("start_install", { onEvent: channel });
  },
  cancel: () => invoke("cancel_install"),
  launch: () => invoke("launch_beaver"),
  close: () => getCurrentWindow().close(),
};
