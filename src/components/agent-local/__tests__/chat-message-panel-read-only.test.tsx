import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ChatMessagePanel } from "../chat-message-panel";

const messageListProps = vi.hoisted(() => ({ current: undefined as Record<string, unknown> | undefined }));

vi.mock("../message-list", () => ({
  MessageList: (props: Record<string, unknown>) => {
    messageListProps.current = props;
    return <div data-testid="message-list" />;
  },
}));

describe("ChatMessagePanel child read-only mode", () => {
  it("keeps file preview while removing reload, edit, and clone actions", () => {
    render(
      <ChatMessagePanel
        chat={{} as never}
        runtime={{ handleReload: vi.fn(), handleEdit: vi.fn(), handleFileClick: vi.fn() } as never}
        knownSubagents={[]}
        cloneEnabled
        requestClone={vi.fn()}
        onFilePreviewPath={vi.fn()}
        readOnly
      />,
    );

    expect(messageListProps.current?.onReload).toBeUndefined();
    expect(messageListProps.current?.onEdit).toBeUndefined();
    expect(messageListProps.current?.onCloneMessage).toBeUndefined();
    expect(messageListProps.current?.onFilePreview).toBeTypeOf("function");
  });
});
