import { useEffect, useState } from "react";
import { Trash } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";

interface GitDeleteButtonProps {
  label: string;
  confirmLabel: string;
  onInspect: () => Promise<boolean>;
  onConfirm: () => Promise<void>;
}

export function GitDeleteButton({
  label,
  confirmLabel,
  onInspect,
  onConfirm,
}: GitDeleteButtonProps) {
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (!confirming) return;
    const timer = window.setTimeout(() => setConfirming(false), 5000);
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") setConfirming(false);
    };
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.clearTimeout(timer);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [confirming]);

  const handleClick = async (event: React.MouseEvent<HTMLButtonElement>) => {
    event.preventDefault();
    event.stopPropagation();
    if (busy) return;
    setBusy(true);
    try {
      if (confirming) {
        await onConfirm();
        setConfirming(false);
      } else if (await onInspect()) {
        setConfirming(true);
      }
    } finally {
      setBusy(false);
    }
  };

  const button = (
    <button
      type="button"
      className="bs-delete-btn"
      data-confirming={confirming ? "true" : undefined}
      aria-label={confirming ? confirmLabel : label}
      disabled={busy}
      onClick={(event) => void handleClick(event)}
    >
      {confirming ? confirmLabel : <Trash size="var(--icon-sm)" />}
    </button>
  );

  return confirming ? button : <Tooltip label={label} align="right">{button}</Tooltip>;
}
