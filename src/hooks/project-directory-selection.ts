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
  const path = typeof result === "string" ? result : String(result);
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
