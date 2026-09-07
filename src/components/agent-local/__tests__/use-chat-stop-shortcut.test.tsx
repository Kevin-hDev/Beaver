/* @vitest-environment jsdom */
import { cleanup, fireEvent, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { useChatStopShortcut } from "../use-chat-stop-shortcut";

afterEach(cleanup);

function Probe({ enabled, onStop }: { enabled: boolean; onStop: () => void }) {
  useChatStopShortcut({ enabled, onStop });
  return <div>stop</div>;
}

describe("useChatStopShortcut", () => {
  it("ignore Escape inactif et reprend quand la surface revient", () => {
    const onStop = vi.fn();
    const view = render(
      <AppSurfaceActivityProvider active>
        <Probe enabled onStop={onStop} />
      </AppSurfaceActivityProvider>,
    );

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onStop).toHaveBeenCalledTimes(1);

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <Probe enabled onStop={onStop} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onStop).toHaveBeenCalledTimes(1);

    view.rerender(
      <AppSurfaceActivityProvider active>
        <Probe enabled onStop={onStop} />
      </AppSurfaceActivityProvider>,
    );
    fireEvent.keyDown(window, { key: "Escape" });
    expect(onStop).toHaveBeenCalledTimes(2);
  });
});
