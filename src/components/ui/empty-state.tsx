import "./empty-state.css";

interface EmptyStateProps {
  message: string;
  action?: string;
  onAction?: () => void;
}

export function EmptyState({ message, action, onAction }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <div className="empty-msg">{message}</div>
      {action && onAction && (
        <button className="empty-action" onClick={onAction}>
          {action}
        </button>
      )}
    </div>
  );
}
