import type { ComponentType } from "react";
import {
  Globe,
  Wrench,
  BookOpenText,
  MagnifyingGlass,
  Link,
  Sparkle,
  Users,
  Plugs,
} from "@/components/ui/icons";
import { TerminalIcon } from "@/components/ui/chat-header-icons";
import { FileReadIcon } from "@/components/ui/file-read-icon";
import { FileWriteIcon } from "@/components/ui/file-write-icon";
import { FolderStateIcon } from "@/components/ui/folder-state-icon";
import { ForecastIcon } from "@/components/ui/forecast-icon";
import { GitIcon } from "@/components/ui/git-icon";
import { RenameIcon } from "@/components/ui/rename-icon";
import { SearchGlobeIcon } from "@/components/ui/search-globe-icon";

export interface ToolIconProps {
  size?: number | string;
  className?: string;
  "aria-hidden"?: boolean | "true" | "false";
}

/* Le dossier d'un listage est celui que la barre latérale montre déplié : la
   ligne annonce qu'on regarde à l'intérieur. Le dessin attend un état, pas ces
   trois props — d'où l'enveloppe. */
function ListIcon({ size, className }: ToolIconProps) {
  return <FolderStateIcon open size={size} className={className} />;
}

const ICONS: Record<string, ComponentType<ToolIconProps>> = {
  Explore: SearchGlobeIcon,
  Terminal: TerminalIcon,
  Globe,
  Git: GitIcon,
  Wrench,
  BookOpenText,
  MagnifyingGlass,
  List: ListIcon,
  FileRead: FileReadIcon,
  FileWrite: FileWriteIcon,
  Edit: RenameIcon,
  Link,
  Sparkle,
  Users,
  Forecast: ForecastIcon,
  Plugs,
};

export function ToolIcon({ name, ...props }: { name: string } & ToolIconProps) {
  const Cmp = ICONS[name] ?? Wrench;
  return <Cmp {...props} />;
}
