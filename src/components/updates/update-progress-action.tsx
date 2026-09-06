import { OperationProgressAction, type OperationProgressActionProps } from "@/components/ui/operation-progress-action";

type UpdateProgressActionProps = Omit<OperationProgressActionProps, "percent" | "canCancel" | "phaseLabel"> & {
  percent: number;
};

export function UpdateProgressAction(props: UpdateProgressActionProps) {
  return <OperationProgressAction {...props} canCancel />;
}
