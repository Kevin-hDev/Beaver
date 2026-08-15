import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { RenameIcon } from "@/components/ui/rename-icon";
import { ArchiveBoxIcon } from "@/components/ui/archive-box-icon";
import { CopyIcon } from "@/components/ui/copy-icon";
import { ValidateIcon } from "@/components/ui/validate-icon";
import type { ContextMenuItem } from "@/components/ui/context-menu";
import { useCopyToClipboard } from "@/hooks/use-copy-to-clipboard";

interface SessionMenuActions {
  sessionId: string | null;
  onRename: (id: string) => void;
  onArchive: (id: string) => void;
}

/* Les trois commandes du menu d'une conversation. Elles vivent hors de la liste
   parce que celle de copie porte un état — la confirmation affichée sur sa
   propre ligne — et que cet état n'a rien à faire dans le composant de liste. */
export function useSessionMenuItems({ sessionId, onRename, onArchive }: SessionMenuActions): ContextMenuItem[] {
  const { t } = useTranslation();
  const { state, copy, reset } = useCopyToClipboard();

  /* Le menu rouvert sur une autre conversation repart de la commande, jamais
     d'une confirmation qui concernait la précédente. */
  useEffect(() => { reset(); }, [sessionId, reset]);

  if (!sessionId) return [];

  return [
    {
      id: "copy-id",
      label: t(`history.${copyLabelKey(state)}`),
      icon: state === "copied" ? <ValidateIcon /> : <CopyIcon />,
      keepOpen: true,
      danger: state === "error",
      onClick: () => { void copy(sessionId); },
    },
    { id: "rename", label: t("history.rename"), icon: <RenameIcon />, onClick: () => onRename(sessionId) },
    { id: "archive", label: t("history.archive"), icon: <ArchiveBoxIcon />, onClick: () => onArchive(sessionId) },
  ];
}

function copyLabelKey(state: string): string {
  if (state === "copied") return "idCopied";
  if (state === "error") return "copyIdFailed";
  return "copyId";
}
