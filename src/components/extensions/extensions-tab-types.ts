import type { DeepPartial, SettingsNavState } from "@/types/navigation";

export interface ExtensionsTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}
