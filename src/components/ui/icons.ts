import { createElement, forwardRef } from "react";
import type { Icon, IconProps } from "@phosphor-icons/react";
import {
  Archive as PhArchive,
  ArrowClockwise as PhArrowClockwise,
  ArrowLeft as PhArrowLeft,
  ArrowRight as PhArrowRight,
  ArrowUpRight as PhArrowUpRight,
  BookOpen as PhBookOpen,
  Pulse as PhPulse,
  ClipboardText as PhClipboardText,
  UserCircle as PhUserCircle,
  Sliders as PhSliders,
  Moon as PhMoon,
  Sun as PhSun,
  Pencil as PhPencil,
  Trash as PhTrash,
  Plus as PhPlus,
  Copy as PhCopy,
  Check as PhCheck,
  CheckCircle as PhCheckCircle,
  Circle as PhCircle,
  CornersIn as PhCornersIn,
  CornersOut as PhCornersOut,
  ShieldWarning as PhShieldWarning,
  ArrowsClockwise as PhArrowsClockwise,
  Gear as PhGear,
  X as PhX,
  CaretDown as PhCaretDown,
  CaretUp as PhCaretUp,
  CaretRight as PhCaretRight,
  DownloadSimple as PhDownloadSimple,
  FileText as PhFileText,
  Clock as PhClock,
  CaretLeft as PhCaretLeft,
  Key as PhKey,
  MagnifyingGlass as PhMagnifyingGlass,
  ArrowSquareOut as PhArrowSquareOut,
  Star as PhStar,
  FolderSimple as PhFolderSimple,
  FolderSimplePlus as PhFolderSimplePlus,
  FolderOpen as PhFolderOpen,
  DotsThreeVertical as PhDotsThreeVertical,
  Hand as PhHand,
  PencilSimple as PhPencilSimple,
  ChatCircleDots as PhChatCircleDots,
  ChatsCircle as PhChatsCircle,
  GearSix as PhGearSix,
  Info as PhInfo,
  Keyboard as PhKeyboard,
  BookOpenText as PhBookOpenText,
  Plugs as PhPlugs,
  PuzzlePiece as PhPuzzlePiece,
  Image as PhImage,
  GitBranch as PhGitBranch,
  GitFork as PhGitFork,
  Broadcast as PhBroadcast,
  Hash as PhHash,
  ChartLineUp as PhChartLineUp,
  Brain as PhBrain,
  Compass as PhCompass,
  TerminalWindow as PhTerminalWindow,
  Globe as PhGlobe,
  Wrench as PhWrench,
  FilePlus as PhFilePlus,
  Link as PhLink,
  List as PhList,
  ListChecks as PhListChecks,
  PauseCircle as PhPauseCircle,
  Question as PhQuestion,
  Sidebar as PhSidebar,
  SidebarSimple as PhSidebarSimple,
  Sparkle as PhSparkle,
  Square as PhSquare,
  TreeStructure as PhTreeStructure,
  Users as PhUsers,
  Spinner as PhSpinner,
  PawPrint as PhPawPrint,
} from "@phosphor-icons/react";

function withSafeCssSize(Component: Icon): Icon {
  const Wrapped = forwardRef<SVGSVGElement, IconProps>(({ size, style, ...props }, ref) => {
    if (typeof size !== "string") {
      return createElement(Component, { ...props, ref, size, style });
    }
    return createElement(Component, {
      ...props,
      ref,
      style: { width: size, height: size, ...style },
    });
  });
  Wrapped.displayName = Component.displayName ?? Component.name ?? "Icon";
  return Wrapped;
}

export const Pulse = withSafeCssSize(PhPulse);
export const ClipboardText = withSafeCssSize(PhClipboardText);
export const UserCircle = withSafeCssSize(PhUserCircle);
export const Sliders = withSafeCssSize(PhSliders);
export const Moon = withSafeCssSize(PhMoon);
export const Sun = withSafeCssSize(PhSun);
export const Pencil = withSafeCssSize(PhPencil);
export const Trash = withSafeCssSize(PhTrash);
export const Plus = withSafeCssSize(PhPlus);
export const Copy = withSafeCssSize(PhCopy);
export const Check = withSafeCssSize(PhCheck);
export const ShieldWarning = withSafeCssSize(PhShieldWarning);
export const ArrowsClockwise = withSafeCssSize(PhArrowsClockwise);
export const Gear = withSafeCssSize(PhGear);
export const X = withSafeCssSize(PhX);
export const CaretDown = withSafeCssSize(PhCaretDown);
export const CaretUp = withSafeCssSize(PhCaretUp);
export const CaretRight = withSafeCssSize(PhCaretRight);
export const DownloadSimple = withSafeCssSize(PhDownloadSimple);
export const FileText = withSafeCssSize(PhFileText);
export const Clock = withSafeCssSize(PhClock);
export const CaretLeft = withSafeCssSize(PhCaretLeft);
export const Key = withSafeCssSize(PhKey);
export const MagnifyingGlass = withSafeCssSize(PhMagnifyingGlass);
export const ArrowSquareOut = withSafeCssSize(PhArrowSquareOut);
export const Star = withSafeCssSize(PhStar);
export const FolderSimple = withSafeCssSize(PhFolderSimple);
export const FolderSimplePlus = withSafeCssSize(PhFolderSimplePlus);
export const FolderOpen = withSafeCssSize(PhFolderOpen);
export const DotsThreeVertical = withSafeCssSize(PhDotsThreeVertical);
export const Hand = withSafeCssSize(PhHand);
export const PencilSimple = withSafeCssSize(PhPencilSimple);
export const ChatCircleDots = withSafeCssSize(PhChatCircleDots);
export const ChatsCircle = withSafeCssSize(PhChatsCircle);
export const GearSix = withSafeCssSize(PhGearSix);
export const Info = withSafeCssSize(PhInfo);
export const Keyboard = withSafeCssSize(PhKeyboard);
export const BookOpenText = withSafeCssSize(PhBookOpenText);
export const Plugs = withSafeCssSize(PhPlugs);
export const PuzzlePiece = withSafeCssSize(PhPuzzlePiece);
export const Image = withSafeCssSize(PhImage);
export const GitBranch = withSafeCssSize(PhGitBranch);
export const GitFork = withSafeCssSize(PhGitFork);
export const Broadcast = withSafeCssSize(PhBroadcast);
export const Hash = withSafeCssSize(PhHash);
export const ChartLineUp = withSafeCssSize(PhChartLineUp);
export const Brain = withSafeCssSize(PhBrain);
export const Compass = withSafeCssSize(PhCompass);
export const TerminalWindow = withSafeCssSize(PhTerminalWindow);
export const Globe = withSafeCssSize(PhGlobe);
export const Wrench = withSafeCssSize(PhWrench);
export const FilePlus = withSafeCssSize(PhFilePlus);
export const Link = withSafeCssSize(PhLink);
export const Sparkle = withSafeCssSize(PhSparkle);
export const Users = withSafeCssSize(PhUsers);
export const Spinner = withSafeCssSize(PhSpinner);
export const PawPrint = withSafeCssSize(PhPawPrint);

export const Archive = withSafeCssSize(PhArchive);
export const ArrowLeft = withSafeCssSize(PhArrowLeft);
export const ArrowRight = withSafeCssSize(PhArrowRight);
export const ArrowUpRight = withSafeCssSize(PhArrowUpRight);
export const BookOpen = withSafeCssSize(PhBookOpen);
export const CheckCircle2 = withSafeCssSize(PhCheckCircle);
export const ChevronDown = withSafeCssSize(PhCaretDown);
export const ChevronRight = withSafeCssSize(PhCaretRight);
export const Circle = withSafeCssSize(PhCircle);
export const Clock3 = withSafeCssSize(PhClock);
export const FolderTree = withSafeCssSize(PhTreeStructure);
export const Globe2 = withSafeCssSize(PhGlobe);
export const HelpCircle = withSafeCssSize(PhQuestion);
export const List = withSafeCssSize(PhList);
export const ListChecks = withSafeCssSize(PhListChecks);
export const Maximize2 = withSafeCssSize(PhCornersOut);
export const Minimize2 = withSafeCssSize(PhCornersIn);
export const PanelRightClose = withSafeCssSize(PhSidebarSimple);
export const PanelRightOpen = withSafeCssSize(PhSidebar);
export const PauseCircle = withSafeCssSize(PhPauseCircle);
export const RotateCw = withSafeCssSize(PhArrowClockwise);
export const Search = withSafeCssSize(PhMagnifyingGlass);
export const Settings = withSafeCssSize(PhGear);
export const Square = withSafeCssSize(PhSquare);
export const TerminalSquare = withSafeCssSize(PhTerminalWindow);
