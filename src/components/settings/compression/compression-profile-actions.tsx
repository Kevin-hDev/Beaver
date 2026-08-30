import { EditableRowActions, type EditableRowController } from "@/components/ui/editable-row-actions";

interface CompressionProfileActionsProps {
  controller: EditableRowController;
  disabled: boolean;
  labels: {
    rename: string;
    remove: string;
    confirm: string;
    cancel: string;
  };
}

export function CompressionProfileActions({
  controller,
  disabled,
  labels,
}: CompressionProfileActionsProps) {
  return (
    <EditableRowActions
      controller={controller}
      disabled={disabled}
      renameLabel={labels.rename}
      deleteLabel={labels.remove}
      confirmLabel={labels.confirm}
      cancelLabel={labels.cancel}
    />
  );
}
