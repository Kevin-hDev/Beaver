import { CaretDown } from "./icons";

export function SelectChevron({ className }: { className?: string }) {
  return <CaretDown size="var(--icon-sm)" weight="bold" className={className} aria-hidden="true" />;
}
