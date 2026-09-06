import type { useUpdateChecker } from "@/hooks/use-update-checker";
import type { RefObject, ReactNode } from "react";
import { SearchDialog } from "./search-dialog";
import { UpdateNotifications } from "./update-notifications";


interface AppLayoutOverlaysProps {
  updatesAnchorRef: RefObject<HTMLElement | null>;
  installationRows?: ReactNode;
  searchOpen: boolean;
  updatesOpen: boolean;
  onCloseSearch: () => void;
  onCloseUpdates: () => void;
  onSearchSelect: (sessionId: string) => void;
  updates: ReturnType<typeof useUpdateChecker>;
}

export function AppLayoutOverlays({
  updatesAnchorRef, installationRows,
  searchOpen,
  updatesOpen,
  onCloseSearch,
  onCloseUpdates,
  onSearchSelect,
  updates,
}: AppLayoutOverlaysProps) {
  return (
    <>
      <SearchDialog
        open={searchOpen}
        onClose={onCloseSearch}
        onSelect={onSearchSelect}
      />
      <UpdateNotifications
        isOpen={updatesOpen}
        onClose={onCloseUpdates}
        appUpdate={updates.visibleAppUpdate}
        ollamaBinaryUpdate={updates.visibleOllamaBinaryUpdate}
        ollamaUpdates={updates.visibleOllamaUpdates}
        forecastDevUpdates={updates.forecastDevUpdates}
        pulling={updates.pulling}
        ollamaBinaryUpdating={updates.ollamaBinaryUpdating}
        ollamaBinaryPercent={updates.ollamaBinaryPercent}
        appDownloading={updates.appDownloading}
        appPercent={updates.appPercent}
        onPullModel={(name) => void updates.pullModel(name)}
        onDownloadApp={(url) => void updates.downloadAppUpdate(url)}
        onUpdateOllamaBinary={() => void updates.updateOllamaBinary()}
        onDismissUpdate={(update) => void updates.dismissUpdate(update)}
        onCancelApp={() => void updates.cancelAppUpdate()}
        onCancelOllamaBinary={() => void updates.cancelOllamaBinary()}
        onCancelModel={() => void updates.cancelModelUpdate()}
        appCancelling={updates.appCancelling}
        ollamaBinaryCancelling={updates.ollamaBinaryCancelling}
        modelCancelling={updates.modelCancelling}
        anchorRef={updatesAnchorRef}
      >{installationRows}</UpdateNotifications>
    </>
  );
}
