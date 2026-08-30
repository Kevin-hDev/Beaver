import type { EditableRowController } from "@/components/ui/editable-row-actions";

interface CompressionProfileActionsProps {
  controller: EditableRowController;
  disabled: boolean;
  labels: {
    rename: string;
    remove: string;
    confirm: string;
    cancel: string;
    question: string;
  };
}

export function CompressionProfileActions({
  controller,
  disabled,
  labels,
}: CompressionProfileActionsProps) {
  return (
    <div className="cpra-actions">
      <button
        type="button"
        className="btn btn-sm btn-secondary"
        disabled={disabled || controller.editing}
        onClick={(event) => { event.stopPropagation(); controller.startRename(); }}
      >
        {labels.rename}
      </button>
      <div className="cpra-confirm-anchor">
        <button
          type="button"
          className="btn btn-sm btn-secondary"
          disabled={disabled || controller.editing}
          aria-expanded={controller.confirmingDelete}
          onClick={(event) => { event.stopPropagation(); controller.startDelete(); }}
        >
          {labels.remove}
        </button>
        {controller.confirmingDelete && (
          <div className="cpra-confirm relief" role="dialog" aria-label={labels.remove}>
            <p>{labels.question}</p>
            <div className="cpra-confirm-buttons">
              <button type="button" className="btn btn-sm btn-secondary" onClick={() => controller.cancel()}>
                {labels.cancel}
              </button>
              <button type="button" className="btn btn-sm btn-danger" onClick={() => { void controller.confirmDelete(); }}>
                {labels.confirm}
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
