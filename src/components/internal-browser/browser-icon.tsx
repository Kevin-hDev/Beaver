import browserIcon from "@/assets/streamline-plump-color-web.svg";

interface BrowserIconProps {
  className: string;
}

export function BrowserIcon({ className }: BrowserIconProps) {
  return (
    <img
      className={className}
      src={browserIcon}
      alt=""
      aria-hidden="true"
      draggable={false}
    />
  );
}
