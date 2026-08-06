import type { SystemPromptView } from "@/types/system-prompts";

interface SystemPromptPreviewProps {
  view: SystemPromptView | null;
  emptyLabel: string;
}

export function SystemPromptPreview({ view, emptyLabel }: SystemPromptPreviewProps) {
  if (!view) return <div className="spp-preview spp-preview-empty">…</div>;
  if (!view.content) return <div className="spp-preview spp-preview-empty">{emptyLabel}</div>;
  const lines = view.content.split("\n");
  const preview = lines.slice(0, 22).join("\n");
  return <div className="spp-preview">{preview}{lines.length > 22 ? "\n…" : ""}</div>;
}
