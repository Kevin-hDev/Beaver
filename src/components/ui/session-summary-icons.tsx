import { svgSizeProps } from "@/components/ui/icon-size";

interface SessionSummaryIconProps {
  size?: number | string;
  className?: string;
}

export function CommitIcon({ size = "var(--icon-md)", className }: SessionSummaryIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 24 24" aria-hidden="true">
      <path d="M0 0h24v24H0z" fill="none" />
      <path fill="currentColor" d="M8.813 15.863Q7.45 14.725 7.1 13H3q-.425 0-.712-.288T2 12t.288-.712T3 11h4.1q.35-1.725 1.713-2.863T12 7t3.188 1.138T16.9 11H21q.425 0 .713.288T22 12t-.288.713T21 13h-4.1q-.35 1.725-1.712 2.863T12 17t-3.187-1.137M12 15q1.25 0 2.125-.875T15 12t-.875-2.125T12 9t-2.125.875T9 12t.875 2.125T12 15" />
    </svg>
  );
}

export function ModificationIcon({ size = "var(--icon-md)", className }: SessionSummaryIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 24 24" aria-hidden="true">
      <path d="M0 0h24v24H0z" fill="none" />
      <path fill="currentColor" d="M10 20H6V4h7v5h5v3.1l2-2V8l-6-6H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h4zm10.2-7c.1 0 .3.1.4.2l1.3 1.3c.2.2.2.6 0 .8l-1 1l-2.1-2.1l1-1c.1-.1.2-.2.4-.2m0 3.9L14.1 23H12v-2.1l6.1-6.1z" />
    </svg>
  );
}

export function PlanIcon({ size = "var(--icon-md)", className }: SessionSummaryIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 48 48" fill="none" stroke="currentColor" strokeWidth="4" aria-hidden="true">
      <path d="M0 0h48v48H0z" fill="none" stroke="none" />
      <path strokeLinejoin="round" d="M5 19h38v22a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2zm0-9a2 2 0 0 1 2-2h34a2 2 0 0 1 2 2v9H5z" />
      <path strokeLinecap="round" strokeLinejoin="round" d="m16 31 6 6 12-12" />
      <path strokeLinecap="round" d="M16 5v8m16-8v8" />
    </svg>
  );
}

export function SubagentSummaryIcon({ size = "var(--icon-md)", className }: SessionSummaryIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 32 32" aria-hidden="true">
      <path d="M0 0h32v32H0z" fill="none" />
      <path fill="currentColor" d="M27.2 16c0-6.19-5.01-11.2-11.2-11.2S4.8 9.81 4.8 16S9.81 27.2 16 27.2S27.2 22.19 27.2 16m-5.6 2.1a1.4 1.4 0 0 1 0 2.8h-4.2a1.4 1.4 0 0 1 0-2.8zm-11.2-6.8a1.397 1.397 0 0 1 1.84.361l.08.119l2.1 3.5l.087.171a1.4 1.4 0 0 1 0 1.1l-.088.171l-2.1 3.5a1.4 1.4 0 0 1-2.4-1.44l1.67-2.78-1.67-2.78-.067-.127a1.394 1.394 0 0 1 .547-1.79zM30 16c0 7.73-6.27 14-14 14S2 23.73 2 16S8.27 2 16 2s14 6.27 14 14" />
    </svg>
  );
}

export function TodoListIcon({ size = "var(--icon-md)", className }: SessionSummaryIconProps) {
  return (
    <svg {...svgSizeProps(size)} className={className} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" aria-hidden="true">
      <path d="M0 0h24v24H0z" fill="none" stroke="none" />
      <path d="M13 5h8m-8 7h8m-8 7h8M3 17l2 2 4-4" />
      <rect width="6" height="6" x="3" y="4" rx="1" />
    </svg>
  );
}
