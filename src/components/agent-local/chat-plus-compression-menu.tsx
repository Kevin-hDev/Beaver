import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import type { CompressionProfileView } from "@/types/compression-profile.generated";
import "./chat-plus-compression-menu.css";

interface ChatPlusCompressionMenuProps {
  profiles: CompressionProfileView[];
  status: "loading" | "ready" | "error";
  selectedId?: string;
  onSelect: (profileId: string) => Promise<boolean>;
  onConfirmed: () => void;
}

export function ChatPlusCompressionMenu({
  profiles,
  status,
  selectedId,
  onSelect,
  onConfirmed,
}: ChatPlusCompressionMenuProps) {
  const { t } = useTranslation();
  const [pendingId, setPendingId] = useState<string | null>(null);
  if (status === "loading") {
    return <div className="cpm-sub-empty" aria-busy="true">{t("chatMenu.compressionLoading")}</div>;
  }
  if (status === "error") {
    return <div className="cpm-sub-empty">{t("chatMenu.compressionUnavailable")}</div>;
  }
  if (profiles.length === 0) {
    return <div className="cpm-sub-empty">{t("chatMenu.compressionNoProfiles")}</div>;
  }

  return (
    <div className="cpcm-list">
      {profiles.slice(0, 20).map((profile) => (
        <button
          key={profile.id}
          type="button"
          className="cpm-sub-item cpcm-item"
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
          <span className="cpcm-name">{profile.name}</span>
          {selectedId === profile.id && <Check size="var(--icon-sm)" aria-hidden="true" />}
        </button>
      ))}
    </div>
  );
}
