import type { useUpdateChecker } from "@/hooks/use-update-checker";
import { IS_MAC } from "@/lib/platform";
import { SearchDialog } from "./search-dialog";
import { UpdateNotifications } from "./update-notifications";

const UPDATES_ANCHOR_MAC = 197;
const UPDATES_ANCHOR_OTHER = 122;

interface AppLayoutOverlaysProps {
  searchOpen: boolean;
  updatesOpen: boolean;
  onCloseSearch: () => void;
  onCloseUpdates: () => void;
  onSearchSelect: (sessionId: string) => void;
  updates: ReturnType<typeof useUpdateChecker>;
}

export function AppLayoutOverlays({
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
        appUpdate={updates.appUpdate}
        ollamaBinaryUpdate={updates.ollamaBinaryUpdate}
        ollamaUpdates={updates.ollamaUpdates}
        forecastDevUpdates={updates.forecastDevUpdates}
        pulling={updates.pulling}
        ollamaBinaryUpdating={updates.ollamaBinaryUpdating}
        ollamaBinaryPercent={updates.ollamaBinaryPercent}
        appDownloading={updates.appDownloading}
        appPercent={updates.appPercent}
        onPullModel={(name) => void updates.pullModel(name)}
        onDownloadApp={(url) => void updates.downloadAppUpdate(url)}
        onUpdateOllamaBinary={() => void updates.updateOllamaBinary()}
        anchorLeft={IS_MAC ? UPDATES_ANCHOR_MAC : UPDATES_ANCHOR_OTHER}
      />
    </>
  );
}
