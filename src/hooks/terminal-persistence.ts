import { homeDir, join } from "@tauri-apps/api/path";
import { readTextFile, remove, rename, writeTextFile } from "@tauri-apps/plugin-fs";
import { DEFAULT_GROUP_KEY, type TerminalGroup } from "./terminal-types";

interface SavedGroups {
  [groupKey: string]: { label: string; cwd: string }[];
}

async function getTabsPath(): Promise<string> {
  const home = await homeDir();
  return join(home, ".local", "share", "cl-go-dash", "terminal-tabs.json");
}

export async function loadSavedGroups(): Promise<SavedGroups> {
  try {
    const path = await getTabsPath();
    const text = await readTextFile(path);
    const parsed = JSON.parse(text) as unknown;
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return parsed as SavedGroups;
    }
    if (Array.isArray(parsed) && parsed.length > 0) {
      return { [DEFAULT_GROUP_KEY]: parsed as SavedGroups[string] };
    }
    return {};
  } catch {
    return {};
  }
}

export async function saveGroups(groups: Map<string, TerminalGroup>): Promise<void> {
  try {
    const path = await getTabsPath();
    const data: SavedGroups = {};
    for (const [key, group] of groups) {
      if (group.tabs.length > 0) {
        data[key] = group.tabs.map(({ label, cwd }) => ({ label, cwd }));
      }
    }
    /* Écriture directe = JSON tronqué si l'app meurt au milieu : on écrit un
       fichier temporaire puis on le renomme, opération atomique sur le même
       volume. Windows refuse de renommer par-dessus un fichier existant :
       on le retire d'abord, la fenêtre de risque y est réduite mais pas nulle. */
    const tmpPath = `${path}.tmp`;
    await writeTextFile(tmpPath, JSON.stringify(data));
    try {
      await rename(tmpPath, path);
    } catch {
      await remove(path).catch(() => {});
      await rename(tmpPath, path);
    }
  } catch {
    console.warn("[terminal-tabs] failed to save");
  }
}
