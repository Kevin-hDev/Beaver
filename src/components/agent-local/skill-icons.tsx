import { MAGIC_WAND_PATH } from "./skill-chip-icons";

interface SkillIconProps {
  className?: string;
}

export function MagicWandIcon({ className }: SkillIconProps) {
  return (
    <svg
      className={className}
      viewBox="0 0 256 256"
      fill="currentColor"
      aria-hidden="true"
    >
      <path d={MAGIC_WAND_PATH} />
    </svg>
  );
}
