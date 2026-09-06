/* @vitest-environment jsdom */
import { readFileSync } from "node:fs";
import { cleanup, render, screen } from "@testing-library/react";
import { useEffect } from "react";
import { afterEach, describe, expect, it } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import { AgentLocalPanelSurface } from "../agent-local-panel-surface";

const CSS = readFileSync("src/components/agent-local/agent-local-tab.css", "utf8");

afterEach(cleanup);

function MountedChild({ label, onMount }: { label: string; onMount: () => void }) {
  useEffect(() => {
    onMount();
  }, [onMount]);
  return <output data-testid={label}>{label}</output>;
}

describe("frontière du panneau Agent Local", () => {
  it("conserve chaque enfant monté pendant une navigation de surface", () => {
    let listMounts = 0;
    let detailMounts = 0;
    const onListMount = () => { listMounts += 1; };
    const onDetailMount = () => { detailMounts += 1; };
    const view = render(
      <AppSurfaceActivityProvider active>
        <AgentLocalPanelSurface>
          <MountedChild label="list" onMount={onListMount} />
          <MountedChild label="detail" onMount={onDetailMount} />
        </AgentLocalPanelSurface>
      </AppSurfaceActivityProvider>,
    );

    expect(screen.getByTestId("list")).toBeInTheDocument();
    expect(screen.getByTestId("detail")).toBeInTheDocument();
    expect(listMounts).toBe(1);
    expect(detailMounts).toBe(1);

    view.rerender(
      <AppSurfaceActivityProvider active={false}>
        <AgentLocalPanelSurface>
          <MountedChild label="list" onMount={onListMount} />
          <MountedChild label="detail" onMount={onDetailMount} />
        </AgentLocalPanelSurface>
      </AppSurfaceActivityProvider>,
    );
    expect(screen.getByTestId("list")).toBeInTheDocument();
    expect(screen.getByTestId("detail")).toBeInTheDocument();

    view.rerender(
      <AppSurfaceActivityProvider active>
        <AgentLocalPanelSurface>
          <MountedChild label="list" onMount={onListMount} />
          <MountedChild label="detail" onMount={onDetailMount} />
        </AgentLocalPanelSurface>
      </AppSurfaceActivityProvider>,
    );

    expect(listMounts).toBe(1);
    expect(detailMounts).toBe(1);
  });

  it("fait primer la règle hidden sur le display flex du panneau", () => {
    const panelRule = CSS.match(/\.al-panel-surface\s*\{([\s\S]*?)\}/)?.[1] ?? "";
    const hiddenRule = CSS.match(/\.al-panel-surface\[hidden\]\s*\{([\s\S]*?)\}/)?.[1] ?? "";

    expect(panelRule).toContain("display: flex");
    expect(hiddenRule).toContain("display: none");
  });
});
