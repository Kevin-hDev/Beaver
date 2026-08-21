import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import type { Project } from "@/types/agent";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";

type AccessRequest = (
  path: string,
  onAllowed: () => void | Promise<void>,
) => Promise<void>;

/* Choisir un dossier, en demander l'accès, puis l'enregistrer comme projet :
   autorité unique de cette séquence pour l'écran d'accueil, la conversation et
   la barre latérale. Les trois l'écrivaient séparément. */
export async function addProjectDirectory(
  requestAccess: AccessRequest,
  addProject: (path: string) => Promise<Project>,
  onAdded?: (project: Project) => void,
) {
  let result: string | string[] | null;
  try {
    result = await openFileDialog({ directory: true });
  } catch {
    showToast(i18n.t("errors.operationFailed"), "error");
    return;
  }
  if (!result) return;
  /* Un seul dossier est demandé, donc le sélecteur rend une chaîne. La branche
     tableau ne sert qu'à ne pas dépendre de cette promesse : elle prend le
     premier, là où un String() collerait tous les chemins bout à bout. */
  const path: unknown = Array.isArray(result) ? result[0] : result;
  if (typeof path !== "string" || !path) return;
  /* requestAccess attrape déjà l'échec de l'enregistrement et l'affiche. */
  await requestAccess(path, async () => {
    const project = await addProject(path);
    onAdded?.(project);
  });
}

export function selectProjectDirectory(
  id: string | null,
  projects: Project[],
  requestAccess: AccessRequest,
  select: (id: string | null) => void,
) {
  if (!id) {
    select(null);
    return;
  }
  const project = projects.find((candidate) => candidate.id === id);
  if (!project) {
    showToast(i18n.t("errors.operationFailed"), "error");
    return;
  }
  void requestAccess(project.path, () => select(id));
}
