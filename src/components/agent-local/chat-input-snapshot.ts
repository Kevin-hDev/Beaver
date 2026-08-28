import type { DroppedFile } from "@/hooks/use-file-drop";

export function sameChatFiles(current?: DroppedFile[], sent?: DroppedFile[]) {
  if (!current && !sent) return true;
  if (!current || !sent || current.length !== sent.length) return false;
  return current.every((file, index) => {
    const other = sent[index];
    return other?.name === file.name && other.path === file.path
      && other.type === file.type && other.size === file.size
      && other.preview === file.preview && other.accessGrant === file.accessGrant;
  });
}
