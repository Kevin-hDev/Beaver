import {
  Link,
  PuzzlePiece,
} from "@/components/ui/icons";
import type { ExtensionRecord } from "@/types/extensions";

interface ExtensionIconProps {
  extension: ExtensionRecord;
}

export function ExtensionIcon({ extension }: ExtensionIconProps) {
  const size = "var(--icon-lg)";
  if (extension.kind === "external") {
    return <Link size={size} weight="regular" />;
  }
  return <PuzzlePiece size={size} weight="fill" />;
}
