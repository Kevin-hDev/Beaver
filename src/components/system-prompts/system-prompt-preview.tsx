import type { SystemPromptView } from "@/types/system-prompts";

interface SystemPromptPreviewProps {
  view: SystemPromptView | null;
  emptyLabel: string;
}

export function SystemPromptPreview({ view, emptyLabel }: SystemPromptPreviewProps) {
  if (!view) return <div className="spp-preview spp-preview-empty">…</div>;
  if (!view.content) return <div className="spp-preview spp-preview-empty">{emptyLabel}</div>;
  /* Contenu entier : la zone défile. Couper à un nombre de lignes fixe donnait
     un texte tronqué sans moyen de lire la suite. */
  return <div className="spp-preview">{view.content}</div>;
}
