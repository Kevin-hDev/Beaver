import { memo } from "react";
import { createPortal } from "react-dom";
import { useOllamaTabContent } from "@/components/ollama/ollama-tab";
import { useApiKeysTabContent } from "@/components/api-keys/api-keys-tab";
import { useOAuthProviderContent } from "@/components/providers/oauth-providers";
import { useConnectorsTabContent } from "@/components/connectors/connectors-tab";
import { useChannelsTabContent } from "@/components/channels/channels-tab";
import { useExtensionsTabContent } from "@/components/extensions/extensions-tab";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";

interface ChildContentProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

interface ChildPortalProps extends ChildContentProps {
  target: HTMLElement;
}

interface SettingsChildSlotsProps extends ChildContentProps {
  subTab: SettingsSubTab;
  target: HTMLElement | null;
}

export function usesSettingsChildSlots(subTab: SettingsSubTab): boolean {
  return subTab === "ollama"
    || subTab === "connectors"
    || subTab === "extensions"
    || subTab === "channels"
    || subTab === "providers";
}

export function SettingsChildSlots({ subTab, target, ...props }: SettingsChildSlotsProps) {
  if (!target) return null;
  const portalProps = { ...props, target };
  if (subTab === "ollama") return <OllamaPortal {...portalProps} />;
  if (subTab === "connectors") return <ConnectorsPortal {...portalProps} />;
  if (subTab === "extensions") return <ExtensionsPortal {...portalProps} />;
  if (subTab === "channels") return <ChannelsPortal {...portalProps} />;
  if (subTab === "providers") return <ProvidersPortal {...portalProps} />;
  return null;
}

const OllamaPortal = memo(function OllamaPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useOllamaTabContent(props), target);
});

const ConnectorsPortal = memo(function ConnectorsPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useConnectorsTabContent(props), target);
});

const ExtensionsPortal = memo(function ExtensionsPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useExtensionsTabContent(props), target);
});

const ChannelsPortal = memo(function ChannelsPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useChannelsTabContent(props), target);
});

const ProvidersPortal = memo(function ProvidersPortal(props: ChildPortalProps) {
  return props.navState.providersSubTab === "oauth"
    ? <OAuthPortal {...props} />
    : <ApiKeysPortal {...props} />;
});

const ApiKeysPortal = memo(function ApiKeysPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useApiKeysTabContent(props), target);
});

const OAuthPortal = memo(function OAuthPortal({ target, ...props }: ChildPortalProps) {
  return createPortal(useOAuthProviderContent(props), target);
});
