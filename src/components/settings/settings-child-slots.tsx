import { memo } from "react";
import { createPortal } from "react-dom";
import { useOllamaTabSlots } from "@/components/ollama/ollama-tab";
import { useApiKeysTabSlots } from "@/components/api-keys/api-keys-tab";
import { useOAuthProviderSlots } from "@/components/providers/oauth-providers";
import { ProvidersShell } from "@/components/providers/providers-shell";
import { useConnectorsTabSlots } from "@/components/connectors/connectors-tab";
import { useChannelsTabSlots } from "@/components/channels/channels-tab";
import type { PanelContentSlots } from "@/components/layout/panel-slots";
import type { DeepPartial, SettingsNavState, SettingsSubTab } from "@/types/navigation";

interface SlotPortalProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
  listTarget: HTMLElement;
  detailTarget: HTMLElement;
}

interface SettingsChildSlotsProps extends Omit<SlotPortalProps, "listTarget" | "detailTarget"> {
  subTab: SettingsSubTab;
  listTarget: HTMLElement | null;
  detailTarget: HTMLElement | null;
}

type ChildSlotProps = Omit<SlotPortalProps, "listTarget" | "detailTarget">;

export function usesSettingsChildSlots(subTab: SettingsSubTab): boolean {
  return subTab === "ollama"
    || subTab === "connectors"
    || subTab === "channels"
    || subTab === "providers";
}

export function SettingsChildSlots({
  subTab,
  listTarget,
  detailTarget,
  ...props
}: SettingsChildSlotsProps) {
  if (!listTarget || !detailTarget) return null;
  const portalProps = { ...props, listTarget, detailTarget };
  if (subTab === "ollama") return <OllamaSlotPortal {...portalProps} />;
  if (subTab === "connectors") return <ConnectorsSlotPortal {...portalProps} />;
  if (subTab === "channels") return <ChannelsSlotPortal {...portalProps} />;
  if (subTab === "providers") return <ProvidersSlotPortal {...portalProps} />;
  return null;
}

const OllamaSlotPortal = memo(function OllamaSlotPortal(props: SlotPortalProps) {
  const slots = useOllamaTabSlots(childProps(props));
  return <SlotPortals slots={slots} listTarget={props.listTarget} detailTarget={props.detailTarget} />;
});

const ConnectorsSlotPortal = memo(function ConnectorsSlotPortal(props: SlotPortalProps) {
  const slots = useConnectorsTabSlots(childProps(props));
  return <SlotPortals slots={slots} listTarget={props.listTarget} detailTarget={props.detailTarget} />;
});

const ChannelsSlotPortal = memo(function ChannelsSlotPortal(props: SlotPortalProps) {
  const slots = useChannelsTabSlots(childProps(props));
  return <SlotPortals slots={slots} listTarget={props.listTarget} detailTarget={props.detailTarget} />;
});

const ProvidersSlotPortal = memo(function ProvidersSlotPortal(props: SlotPortalProps) {
  return props.navState.providersSubTab === "oauth"
    ? <OAuthSlotPortal {...props} />
    : <ApiKeysSlotPortal {...props} />;
});

const ApiKeysSlotPortal = memo(function ApiKeysSlotPortal(props: SlotPortalProps) {
  const slots = useApiKeysTabSlots(childProps(props));
  return <ProviderSlotPortals slots={slots} active="api" props={props} />;
});

const OAuthSlotPortal = memo(function OAuthSlotPortal(props: SlotPortalProps) {
  const slots = useOAuthProviderSlots(childProps(props));
  return <ProviderSlotPortals slots={slots} active="oauth" props={props} />;
});

function ProviderSlotPortals({ slots, active, props }: {
  slots: PanelContentSlots;
  active: "api" | "oauth";
  props: SlotPortalProps;
}) {
  const detail = (
    <ProvidersShell active={active} onChange={(providersSubTab) => props.onNavChange({ providersSubTab })}>
      {slots.detail}
    </ProvidersShell>
  );
  return <SlotPortals slots={{ list: slots.list, detail }} listTarget={props.listTarget} detailTarget={props.detailTarget} />;
}

function childProps({ navState, onNavChange, onNavReplace }: SlotPortalProps): ChildSlotProps {
  return { navState, onNavChange, onNavReplace };
}

function SlotPortals({
  slots,
  listTarget,
  detailTarget,
}: {
  slots: PanelContentSlots;
  listTarget: HTMLElement;
  detailTarget: HTMLElement;
}) {
  return (
    <>
      {createPortal(slots.list, listTarget)}
      {createPortal(slots.detail, detailTarget)}
    </>
  );
}
