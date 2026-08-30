import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import type { CompressionProfileView } from "@/types/compression-profile.generated";
import "./chat-plus-compression-menu.css";

interface ChatPlusCompressionMenuProps {
  profiles: CompressionProfileView[];
  selectedId?: string;
  onSelect: (profileId: string) => Promise<boolean>;
  onConfirmed: () => void;
}

export function ChatPlusCompressionMenu({
  profiles,
  selectedId,
  onSelect,
  onConfirmed,
}: ChatPlusCompressionMenuProps) {
  const { t } = useTranslation();
  const [pendingId, setPendingId] = useState<string | null>(null);
  if (profiles.length === 0) {
    return <div className="cpm-sub-empty">{t("chatMenu.compressionNoProfiles")}</div>;
  }

  return (
    <div className="cpc-list">
      {profiles.slice(0, 20).map((profile) => (
        <button
          key={profile.id}
          type="button"
          className="cpm-sub-item cpc-item"
          title={profile.name}
          disabled={pendingId !== null}
          aria-current={selectedId === profile.id ? "true" : undefined}
          onClick={() => {
            setPendingId(profile.id);
            void onSelect(profile.id).then((saved) => {
              setPendingId(null);
              if (saved) onConfirmed();
            });
          }}
        >
          <span className="cpc-name">{profile.name}</span>
          {selectedId === profile.id && <Check size="var(--icon-sm)" aria-hidden="true" />}
        </button>
      ))}
    </div>
  );
}
