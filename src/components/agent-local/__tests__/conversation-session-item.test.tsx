import { describe, expect, it, vi } from "vitest";
import { render } from "@testing-library/react";
import { ConversationSessionItem } from "../conversation-session-item";
import type { AgentSessionMeta } from "@/types/agent";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/components/ui/icons", () => ({
  DotsThreeVertical: () => <span data-testid="dots" />,
  ChatsCircle: (props: { weight?: string; className?: string }) => (
    <span data-testid="chat-icon" data-weight={props.weight} className={props.className} />
  ),
}));

vi.mock("@/components/channels/channel-icon", () => ({
  ChannelIcon: () => <span data-testid="channel-icon" />,
}));

vi.mock("../conversation-session-item.css", () => ({}));

function session(overrides: Partial<AgentSessionMeta> = {}): AgentSessionMeta {
  return {
    id: "s1",
    name: "Test",
    model: "llama3",
    provider: "ollama",
    message_count: 1,
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function renderItem(overrides: Partial<Parameters<typeof ConversationSessionItem>[0]> = {}) {
  return render(
    <ConversationSessionItem
      session={session()}
      active={false}
      isRunning={false}
      hasUnread={false}
      renaming={false}
      inputRef={{ current: null }}
      onSelect={vi.fn()}
      onRenameSubmit={vi.fn()}
      onCancelRename={vi.fn()}
      onMenu={vi.fn()}
      nowMs={Date.UTC(2026, 0, 1, 0, 5, 0)}
      {...overrides}
    />,
  );
}

describe("ConversationSessionItem", () => {
  it("anime l'icône et le nom quand la session est en cours", () => {
    const { container } = renderItem({ isRunning: true });
    const item = container.querySelector(".conv-session-indented");

    expect(item?.classList.contains("is-running")).toBe(true);
    expect(item?.querySelector(".conv-name.thinking-active")).not.toBeNull();
    expect(item?.querySelector(".conv-running-icon")).not.toBeNull();
  });

  it("affiche le point terminé pour une session non active", () => {
    const { container } = renderItem({ hasUnread: true });

    expect(container.querySelector(".conv-session-indented.has-unread")).not.toBeNull();
    expect(container.querySelector(".conv-unread-dot")).not.toBeNull();
  });

  it("masque le point terminé pour la session active", () => {
    const { container } = renderItem({ active: true, hasUnread: true });

    expect(container.querySelector(".conv-unread-dot")).toBeNull();
  });
});
