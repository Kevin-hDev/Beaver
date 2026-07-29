import { useMemo, useState } from "react";
import { useExtensions } from "@/hooks/use-extensions";
import { ExtensionAddDialog } from "./extension-add-dialog";
import { ExtensionsErrorBoundary } from "./extensions-error-boundary";
import { ExtensionsPage } from "./extensions-page";
import { ExtensionsSidebar } from "./extensions-sidebar";
import type { ExtensionsTabProps } from "./extensions-tab-types";

export function useExtensionsTabSlots({
  navState,
  onNavChange,
  onNavReplace,
}: ExtensionsTabProps): { list: React.ReactNode; detail: React.ReactNode } {
  const registry = useExtensions();
  const [adding, setAdding] = useState(false);
  const selected = registry.extensions.find(
    (extension) => extension.manifest.id === navState.extensionId,
  ) ?? null;

  const list = useMemo(() => (
    <ExtensionsSidebar
      section={navState.extensionsSection}
      onSelect={(extensionsSection) =>
        onNavReplace({ extensionsSection, extensionId: null })}
    />
  ), [navState.extensionsSection, onNavReplace]);

  const detailResetKey = `${navState.extensionsSection}:${navState.extensionId ?? "list"}`;
  const detail = (
    <>
      <ExtensionsErrorBoundary
        resetKey={detailResetKey}
        onReset={() => onNavReplace({ extensionId: null })}
      >
        <ExtensionsPage
          section={navState.extensionsSection}
          selected={selected}
          records={registry.extensions}
          host={registry.host}
          loading={registry.loading}
          loadError={registry.loadError}
          operationError={registry.operationError}
          busyIds={registry.busyIds}
          protectedPluginIds={registry.protectedPluginIds}
          priorityBusy={registry.priorityBusy}
          onSelect={(extensionId) => onNavChange({ extensionId })}
          onAdd={() => setAdding(true)}
          onEnabled={(id, enabled) => void registry.setEnabled(id, enabled)}
          onShowInChat={(id, show) => void registry.setShowInChat(id, show)}
          onOpenSource={(id) => void registry.openSource(id)}
          onUpdate={(id) => void registry.update(id)}
          onRemove={(id) => {
            onNavReplace({ extensionId: null });
            void registry.remove(id);
          }}
          onReload={() => void registry.reload()}
          onRecover={() => void registry.recover()}
          onPrioritySave={registry.setPriorityPlugins}
        />
      </ExtensionsErrorBoundary>
      {adding && (
        <ExtensionAddDialog
          onClose={() => setAdding(false)}
          onAdd={async (path) => {
            const outcome = await registry.addLocal(path);
            if (!outcome.record) return outcome.errorKey;
            onNavChange({ extensionId: outcome.record.manifest.id });
            return null;
          }}
          onInstall={async (source, locator) => {
            const outcome = source === "git"
              ? await registry.installGit(locator)
              : await registry.installNpm(locator);
            if (!outcome.record) return outcome.errorKey;
            onNavChange({ extensionId: outcome.record.manifest.id });
            return null;
          }}
        />
      )}
    </>
  );

  return { list, detail };
}
