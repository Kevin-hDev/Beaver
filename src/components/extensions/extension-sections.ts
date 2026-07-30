import type { Icon } from "@phosphor-icons/react";
import { Gear, Link, PuzzlePiece, Wrench } from "@/components/ui/icons";
import type { ExtensionsSettingsSection } from "@/types/navigation";

interface ExtensionSectionDef {
  id: ExtensionsSettingsSection;
  key: string;
  icon: Icon;
}

export const EXTENSION_SECTIONS: readonly ExtensionSectionDef[] = [
  { id: "plugins", key: "extensions.sections.plugins", icon: PuzzlePiece },
  { id: "custom", key: "extensions.sections.custom", icon: Wrench },
  { id: "external", key: "extensions.sections.external", icon: Link },
  { id: "host", key: "extensions.sections.host", icon: Gear },
];
