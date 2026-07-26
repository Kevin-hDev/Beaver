import type { ComponentProps } from "react";
import {
  Compass,
  PencilSimple,
  TerminalWindow,
  Globe,
  GitBranch,
  Wrench,
  BookOpenText,
  FileText,
  Image as ImageIcon,
  MagnifyingGlass,
  FolderOpen,
  FilePlus,
  Pencil,
  Link,
  Sparkle,
  Users,
  ChartLineUp,
  Plugs,
} from "@/components/ui/icons";

const ICONS = {
  Compass,
  PencilSimple,
  TerminalWindow,
  Globe,
  GitBranch,
  Wrench,
  BookOpenText,
  FileText,
  Image: ImageIcon,
  MagnifyingGlass,
  FolderOpen,
  FilePlus,
  Pencil,
  Link,
  Sparkle,
  Users,
  ChartLineUp,
  Plugs,
} as const;

type ToolIconName = keyof typeof ICONS;

type IconComponentProps = ComponentProps<(typeof ICONS)[ToolIconName]>;

export function ToolIcon({ name, ...props }: { name: string } & IconComponentProps) {
  const Cmp = ICONS[name as ToolIconName] ?? Wrench;
  return <Cmp {...props} />;
}
