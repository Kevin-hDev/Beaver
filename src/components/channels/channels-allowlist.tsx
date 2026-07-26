import { useState } from "react";
import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";

const MAX_ALLOWLIST_USERS = 100;

interface ChannelsAllowlistProps {
  allowlist: string[];
  onChange: (list: string[]) => void;
}

export function ChannelsAllowlist({ allowlist, onChange }: ChannelsAllowlistProps) {
  const { t } = useTranslation();
  const [input, setInput] = useState("");

  const handleAdd = () => {
    const id = input.trim();
    if (!id || allowlist.includes(id) || allowlist.length >= MAX_ALLOWLIST_USERS) return;
    onChange([...allowlist, id]);
    setInput("");
  };

  const handleRemove = (id: string) => {
    onChange(allowlist.filter((a) => a !== id));
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleAdd();
    }
  };

  return (
    <div className="ch-allowlist">
      <div className="ch-allowlist-input-row">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={t("channels.detail.allowlistPlaceholder")}
          className="wk-input ch-allowlist-input"
        />
        <button
          type="button"
          className="wk-btn-secondary"
          onClick={handleAdd}
          disabled={!input.trim() || allowlist.length >= MAX_ALLOWLIST_USERS}
        >
          {t("channels.detail.allowlistAdd")}
        </button>
      </div>
      {allowlist.length > 0 && (
        <div className="ch-allowlist-tags">
          {allowlist.map((id) => (
            <span key={id} className="ch-allowlist-tag">
              {id}
              <Tooltip label={t("channels.detail.removeUser")}>
                <button type="button" className="ch-allowlist-tag-remove" onClick={() => handleRemove(id)}>
                  <X size="var(--icon-2xs)" />
                </button>
              </Tooltip>
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
