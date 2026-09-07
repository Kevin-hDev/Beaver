/* @vitest-environment jsdom */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { createRef } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { ChatInputEditor } from "../chat-input-editor";

const focus = vi.hoisted(() => vi.fn());

vi.mock("@/hooks/use-codemirror-chat", () => ({
  useCodemirrorChat: () => ({ hostRef: createRef<HTMLDivElement>(), focus }),
}));

afterEach(() => {
  cleanup();
  focus.mockClear();
});

describe("ChatInputEditor et l'activité de surface", () => {
  it("ne focalise pas le composeur inactif puis reprend au retour", () => {
    const view = render(
      <AppSurfaceActivityProvider active>
        <ChatInputEditor
          value=""
          placeholder="composer"
          readOnly={false}
          activeSkills={[]}
          onTextChange={vi.fn()}
          onKeyEvent={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );

    fireEvent.keyDown(window, { code: "KeyL", key: "l", ctrlKey: true });
    expect(focus).toHaveBeenCalledTimes(1);

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <ChatInputEditor
          value=""
          placeholder="composer"
          readOnly={false}
          activeSkills={[]}
          onTextChange={vi.fn()}
          onKeyEvent={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.keyDown(window, { code: "KeyL", key: "l", ctrlKey: true });
    expect(focus).toHaveBeenCalledTimes(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <ChatInputEditor
          value=""
          placeholder="composer"
          readOnly={false}
          activeSkills={[]}
          onTextChange={vi.fn()}
          onKeyEvent={vi.fn()}
        />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.keyDown(window, { code: "KeyL", key: "l", ctrlKey: true });
    expect(focus).toHaveBeenCalledTimes(2);
  });
});
