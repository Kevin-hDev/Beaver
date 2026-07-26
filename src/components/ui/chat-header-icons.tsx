import { svgSizeProps } from "@/components/ui/icon-size";

interface ChatHeaderIconProps {
  size?: number | string;
  className?: string;
}

export function SessionSummaryIcon({ size = 16, className }: ChatHeaderIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 16 16" aria-hidden="true">
      <path d="M0 0h16v16H0z" fill="none" />
      <path fill="currentColor" d="M5.5 5A.75.75 0 1 1 4 5a.75.75 0 0 1 1.5 0m0 3A.75.75 0 1 1 4 8a.75.75 0 0 1 1.5 0m-.75 3.75a.75.75 0 1 0 0-1.5a.75.75 0 0 0 0 1.5M6.5 5a.5.5 0 0 1 .5-.5h4.5a.5.5 0 0 1 0 1H7a.5.5 0 0 1-.5-.5M7 7.5a.5.5 0 0 0 0 1h4.5a.5.5 0 0 0 0-1zM6.5 11a.5.5 0 0 1 .5-.5h4.5a.5.5 0 0 1 0 1H7a.5.5 0 0 1-.5-.5m-2-9A2.5 2.5 0 0 0 2 4.5v7A2.5 2.5 0 0 0 4.5 14h7a2.5 2.5 0 0 0 2.5-2.5v-7A2.5 2.5 0 0 0 11.5 2zM3 4.5A1.5 1.5 0 0 1 4.5 3h7A1.5 1.5 0 0 1 13 4.5v7a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 3 11.5z" />
    </svg>
  );
}

export function PanelToggleIcon({ size = 24, className }: ChatHeaderIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" aria-hidden="true">
      <path d="M0 0h24v24H0z" fill="none" stroke="none" />
      <path d="M15 3.5v17M3 9.4c0-2.24 0-3.36.436-4.216a4 4 0 0 1 1.748-1.748C6.04 3 7.16 3 9.4 3h5.2c2.24 0 3.36 0 4.216.436a4 4 0 0 1 1.748 1.748C21 6.04 21 7.16 21 9.4v5.2c0 2.24 0 3.36-.436 4.216a4 4 0 0 1-1.748 1.748C17.96 21 16.84 21 14.6 21H9.4c-2.24 0-3.36 0-4.216-.436a4 4 0 0 1-1.748-1.748C3 17.96 3 16.84 3 14.6z" />
    </svg>
  );
}

export function TerminalIcon({ size = 24, className }: ChatHeaderIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="1.5" aria-hidden="true">
      <path d="M0 0h24v24H0z" fill="none" stroke="none" />
      <rect width="18.5" height="15.5" x="2.75" y="4.25" rx="3.5" />
      <path d="m7.25 9 3 3-3 3m5.5 0h4" />
    </svg>
  );
}
