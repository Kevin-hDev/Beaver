import { invoke } from "@tauri-apps/api/core";
import type { ChannelType } from "@/types/channels";

export interface GatewayAccountCredentials {
  token?: string;
  botToken?: string;
  appToken?: string;
}

export async function configureGatewayAccountTokens(
  channelId: ChannelType,
  accountId: string,
  credentials: GatewayAccountCredentials,
): Promise<void> {
  await invoke("gateway_configure_account_tokens", {
    channelId,
    accountId: accountId.trim(),
    credentials,
  });
}
