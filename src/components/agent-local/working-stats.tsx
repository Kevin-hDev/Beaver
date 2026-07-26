import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { InlineActivityIndicator } from "./inline-activity-indicator";
import { formatCompactDuration } from "@/lib/duration-format";

function WorkingStats({ startedAt, liveTokenCount }: {
  startedAt: number; liveTokenCount: number;
}) {
  const { t } = useTranslation();
  const [now, setNow] = useState(startedAt);

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, []);

  const elapsed = formatCompactDuration(now - startedAt);
  const hasTokens = liveTokenCount > 0;

  return (
    <span className="working-stats thinking-active">
      <span>
        {t("agentLocal.working", { duration: elapsed })}
        {hasTokens ? ` · ↑ ${liveTokenCount} ${t("agentLocal.tokens")}` : ""}
      </span>
    </span>
  );
}

export function LoadingIndicator({ startedAt, liveTokenCount }: {
  startedAt: number; liveTokenCount: number;
}) {
  return (
    <div className="chat-loading">
      <InlineActivityIndicator className="chat-loading-indicator">
        <WorkingStats startedAt={startedAt} liveTokenCount={liveTokenCount} />
      </InlineActivityIndicator>
    </div>
  );
}
