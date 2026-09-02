import { PuzzlePiece } from "@/components/ui/icons";
import documentsIcon from "@/assets/extensions/office/documents.svg";
import pdfIcon from "@/assets/extensions/office/pdf.svg";
import presentationsIcon from "@/assets/extensions/office/presentations.svg";
import spreadsheetsIcon from "@/assets/extensions/office/spreadsheets.svg";
import type { ExtensionRecord } from "@/types/extensions";
import "./extension-icon.css";

interface ExtensionIconProps {
  extension: ExtensionRecord;
}

export function ExtensionIcon({ extension }: ExtensionIconProps) {
  const officialIcon = OFFICIAL_PLUGIN_ICONS[extension.manifest.id];
  if (officialIcon) {
    return <img className="exti-artwork" src={officialIcon} alt="" aria-hidden="true" />;
  }
  const size = "var(--icon-lg)";
  return <PuzzlePiece size={size} weight="fill" />;
}

const OFFICIAL_PLUGIN_ICONS: Readonly<Record<string, string>> = {
  "beaver.office.documents": documentsIcon,
  "beaver.office.pdf": pdfIcon,
  "beaver.office.spreadsheets": spreadsheetsIcon,
  "beaver.office.presentations": presentationsIcon,
};
