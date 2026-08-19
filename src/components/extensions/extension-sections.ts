import type { ComponentType } from "react";
import {
  CustomExtensionsIcon,
  ExtensionHostIcon,
  ExternalAppsIcon,
  PluginsIcon,
} from "@/components/ui/extension-section-icons";
import type { InlineIconProps } from "@/components/ui/inline-icon";
import type { ExtensionsSettingsSection } from "@/types/navigation";

interface ExtensionSectionDef {
  id: ExtensionsSettingsSection;
  key: string;
  icon: ComponentType<InlineIconProps>;
}

export const EXTENSION_SECTIONS: readonly ExtensionSectionDef[] = [
  { id: "plugins", key: "extensions.sections.plugins", icon: PluginsIcon },
  { id: "custom", key: "extensions.sections.custom", icon: CustomExtensionsIcon },
  { id: "external", key: "extensions.sections.external", icon: ExternalAppsIcon },
  { id: "host", key: "extensions.sections.host", icon: ExtensionHostIcon },
];
