import { describe, expect, it, vi } from "vitest";
import { fireEvent, render } from "@testing-library/react";
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
      dragProps={{ "data-drag-id": "s1", "data-drag-group": "essai", "data-dragging": undefined, style: {} }}
      dragHandleProps={{ onPointerDown: vi.fn() }}
      didDrag={() => false}
      onStartRename={vi.fn()}
      onRenameSubmit={vi.fn()}
      onCancelRename={vi.fn()}
      onMenu={vi.fn()}
      nowMs={Date.UTC(2026, 0, 1, 0, 5, 0)}
      {...overrides}
    />,
  );
}

describe("ConversationSessionItem", () => {
  /* Un seul signal dit qu'une session tourne : le repère posé dans la gouttière
     du retrait. Le nom scintillait aussi, deux animations pour une même chose. */
  it("pose le repère d'activité quand la session est en cours", () => {
    const { container } = renderItem({ isRunning: true });
    const item = container.querySelector(".conv-session-indented");

    expect(item?.classList.contains("is-running")).toBe(true);
    expect(item?.querySelector(".conv-running-icon")).not.toBeNull();
  });

  it("ne pose aucun repère d'activité quand la session est au repos", () => {
    const { container } = renderItem({ isRunning: false });

    expect(container.querySelector(".conv-running-icon")).toBeNull();
  });

  it("affiche le point terminé pour une session non active", () => {
    const { container } = renderItem({ hasUnread: true });

    expect(container.querySelector(".conv-session-indented.has-unread")).not.toBeNull();
    expect(container.querySelector(".conv-unread-dot")).not.toBeNull();
  });

  it("passe en renommage au double-clic sur le nom", () => {
    const onStartRename = vi.fn();
    const { container } = renderItem({ onStartRename });

    fireEvent.doubleClick(container.querySelector(".conv-session-main") as HTMLElement);

    expect(onStartRename).toHaveBeenCalledWith("s1");
  });

  /* Le bouton de menu et l'âge vivent hors de la zone du nom : les
     double-cliquer en visant leur action ne doit pas ouvrir un renommage. */
  it("ignore le double-clic sur le bouton de menu", () => {
    const onStartRename = vi.fn();
    const { getByTestId } = renderItem({ onStartRename });

    fireEvent.doubleClick(getByTestId("dots"));

    expect(onStartRename).not.toHaveBeenCalled();
  });

  it("masque le point terminé pour la session active", () => {
    const { container } = renderItem({ active: true, hasUnread: true });

    expect(container.querySelector(".conv-unread-dot")).toBeNull();
  });
});
