import { svgSizeProps } from "./icon-size";

interface ComposeIconProps {
  size?: number | string;
  className?: string;
}

export function ComposeIcon({ size = 16, className }: ComposeIconProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      {...svgSizeProps(size)}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.8}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="M12 3H7a4 4 0 0 0-4 4v10a4 4 0 0 0 4 4h10a4 4 0 0 0 4-4v-5" />
      <path d="M18.375 2.625a2.121 2.121 0 1 1 3 3L12 15l-4 1 1-4Z" />
    </svg>
  );
}
