import { svgSizeProps } from "./icon-size";

interface FolderStateIconProps {
  open: boolean;
  size?: number | string;
  className?: string;
}

const CLOSED_PATH = "M216,72H131.31L104,44.69A15.86,15.86,0,0,0,92.69,40H40A16,16,0,0,0,24,56V200.62A15.4,15.4,0,0,0,39.38,216H216.89A15.13,15.13,0,0,0,232,200.89V88A16,16,0,0,0,216,72ZM40,56H92.69l16,16H40ZM216,200H40V88H216Z";
const OPEN_PATH = "M245,110.64A16,16,0,0,0,232,104H216V88a16,16,0,0,0-16-16H130.67L102.94,51.2a16.14,16.14,0,0,0-9.6-3.2H40A16,16,0,0,0,24,64V208h0a8,8,0,0,0,8,8H211.1a8,8,0,0,0,7.59-5.47l28.49-85.47A16.05,16.05,0,0,0,245,110.64ZM93.34,64,123.2,86.4A8,8,0,0,0,128,88h72v16H69.77a16,16,0,0,0-15.18,10.94L40,158.7V64Zm112,136H43.1l26.67-80H232Z";

export function FolderStateIcon({ open, size = 14, className }: FolderStateIconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      {...svgSizeProps(size)}
      fill="currentColor"
      viewBox="0 0 256 256"
      className={className}
      aria-hidden="true"
      focusable="false"
    >
      <path d={open ? OPEN_PATH : CLOSED_PATH} />
    </svg>
  );
}
