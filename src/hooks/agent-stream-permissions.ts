import { records } from "./agent-stream-records";

export function clearStreamPermission(permissionId: string): void {
  for (const record of records.values()) {
    const nextPending = record.state.pendingPermissions.filter((item) => item.id !== permissionId);
    if (nextPending.length === record.state.pendingPermissions.length) continue;
    record.state = { ...record.state, pendingPermissions: nextPending };
  }
}
