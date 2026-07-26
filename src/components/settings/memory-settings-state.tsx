export function MemoryState({
  text,
  retryLabel,
  onRetry,
}: {
  text: string;
  retryLabel?: string;
  onRetry?: () => void;
}) {
  return (
    <div className="mems-page">
      <div className="mems-state">
        <span>{text}</span>
        {onRetry && <RetryButton label={retryLabel ?? text} onRetry={onRetry} />}
      </div>
    </div>
  );
}

export function MemoryError({
  text,
  retryLabel,
  onRetry,
}: {
  text: string;
  retryLabel: string;
  onRetry: () => void;
}) {
  return (
    <div className="mems-error" role="alert">
      <span>{text}</span>
      <RetryButton label={retryLabel} onRetry={onRetry} />
    </div>
  );
}

function RetryButton({ label, onRetry }: { label: string; onRetry: () => void }) {
  return <button type="button" onClick={onRetry}>{label}</button>;
}
