import type { CSSProperties, PointerEvent, ReactNode } from "react";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import "./agent-side-panel.css";

interface AgentSidePanelProps {
  open: boolean;
  fullscreen: boolean;
  displayWidth: number;
  fullscreenSwitching: boolean;
  resizing: boolean;
  mode: PanelMode;
  onResizeStart: (event: PointerEvent) => void;
  previewContent: ReactNode;
  forecastContent?: ReactNode;
  browserContent?: ReactNode;
}

export function AgentSidePanel(props: AgentSidePanelProps) {
  return (
    <>
      <div className={`asp-resize-slot ${props.open && !props.fullscreen ? "open" : ""}`}>
        <div className="asp-resize" onPointerDown={props.onResizeStart} />
      </div>
      <aside
        className={`asp-panel ${props.open ? "open" : ""} ${props.fullscreen ? "fullscreen" : ""} ${props.fullscreenSwitching ? "fullscreen-switching" : ""} ${props.resizing ? "resizing" : ""}`}
        data-nav-zone="sharedPanel"
        style={{
          "--asp-width": `${props.displayWidth}px`,
        } as CSSProperties}
        aria-hidden={!props.open}
        inert={!props.open}
      >
        <div className={`asp-slide-wrapper asp-slide-${props.mode}`}>
          <div className="asp-slide-child" aria-hidden={props.mode !== "preview"} inert={props.mode !== "preview"}>
            {props.previewContent}
          </div>
          <div className="asp-slide-child" aria-hidden={props.mode !== "forecast"} inert={props.mode !== "forecast"}>
            {props.forecastContent}
          </div>
          <div className="asp-slide-child" aria-hidden={props.mode !== "browser"} inert={props.mode !== "browser"}>
            {props.browserContent}
          </div>
        </div>
      </aside>
    </>
  );
}
