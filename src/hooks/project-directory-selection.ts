import type { Project } from "@/types/agent";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";

type AccessRequest = (
  path: string,
  onAllowed: () => void | Promise<void>,
) => Promise<void>;

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
