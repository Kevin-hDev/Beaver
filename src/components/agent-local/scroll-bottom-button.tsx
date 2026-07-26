interface ScrollBottomButtonProps {
  onClick: () => void;
  variant?: "floating" | "inline";
}

export function ScrollBottomButton({ onClick, variant = "floating" }: ScrollBottomButtonProps) {
  const className = variant === "inline"
    ? "icon-btn scroll-bottom-btn scroll-bottom-btn-inline"
    : "icon-btn scroll-bottom-btn";

  return (
    <button type="button" className={className} onClick={onClick}>
      <svg className="scroll-bottom-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
        <path d="M12 17V3" />
        <path d="m6 11 6 6 6-6" />
        <path d="M19 21H5" />
      </svg>
    </button>
  );
}
