import { createElement, type ComponentType } from "react";
import type { InlineIconProps } from "@/components/ui/inline-icon";
import {
  Activity, Archive, Bell, BookOpen, Brain, Check, ChevronDown, Circle,
  Gear, House, Info, Link, Moon, Plus, PuzzlePiece, Sparkle, Sun, Terminal,
  Warning, X,
} from "@/components/ui/icons";
import type { ExtensionUiIcon } from "@/types/extension-ui-contract.generated";

const ICONS: Readonly<Record<ExtensionUiIcon, ComponentType<InlineIconProps>>> = {
  activity: Activity,
  archive: Archive,
  bell: Bell,
  "book-open": BookOpen,
  brain: Brain,
  check: Check,
  "chevron-down": ChevronDown,
  circle: Circle,
  gear: Gear,
  house: House,
  info: Info,
  link: Link,
  moon: Moon,
  plus: Plus,
  "puzzle-piece": PuzzlePiece,
  sparkle: Sparkle,
  sun: Sun,
  terminal: Terminal,
  warning: Warning,
  x: X,
};

export function standardIcon(name?: ExtensionUiIcon): ComponentType<InlineIconProps> {
  return name ? ICONS[name] : PuzzlePiece;
}

export function StandardIcon({ name, ...props }: InlineIconProps & { name?: ExtensionUiIcon }) {
  return createElement(standardIcon(name), props);
}
