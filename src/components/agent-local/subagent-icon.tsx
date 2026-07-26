import { subagentColorKey } from "@/lib/subagent-display";
import type { SubagentInfo } from "@/types/agent";
import "./subagent-icon.css";

type SubagentIconAgent = Pick<SubagentInfo, "type" | "status" | "colorKey">;

interface SubagentIconProps {
  agent: SubagentIconAgent;
  className?: string;
  size?: string | number;
}

type IconNode = {
  cx: number;
  cy: number;
  r: number;
  motion: "a" | "b" | "c" | "d";
};

const CLAUDIATOR_NODES: IconNode[] = [
  { cx: 19, cy: 19, r: 2, motion: "a" },
  { cx: 5, cy: 19, r: 2, motion: "b" },
  { cx: 12, cy: 5, r: 2, motion: "c" },
  { cx: 5, cy: 5, r: 1, motion: "d" },
  { cx: 19, cy: 5, r: 1, motion: "b" },
  { cx: 5, cy: 12, r: 1, motion: "c" },
  { cx: 12, cy: 12, r: 1, motion: "a" },
  { cx: 19, cy: 12, r: 1, motion: "d" },
  { cx: 12, cy: 19, r: 1, motion: "c" },
];

const GEMINITOR_NODES: IconNode[] = [
  { cx: 12, cy: 14, r: 1, motion: "a" },
  { cx: 7, cy: 14, r: 1, motion: "b" },
  { cx: 17, cy: 14, r: 1, motion: "c" },
  { cx: 12, cy: 5, r: 1, motion: "d" },
  { cx: 14.5, cy: 9.5, r: 1, motion: "b" },
  { cx: 9.5, cy: 9.5, r: 1, motion: "c" },
  { cx: 14.5, cy: 18.5, r: 1, motion: "a" },
  { cx: 9.5, cy: 18.5, r: 1, motion: "d" },
  { cx: 4.5, cy: 18.5, r: 1, motion: "c" },
  { cx: 19.5, cy: 18.5, r: 1, motion: "b" },
];

export function SubagentIcon({ agent, className = "", size = "var(--icon-sm)" }: SubagentIconProps) {
  const isClaudiator = subagentColorKey(agent) === "claudiator";
  const nodes = isClaudiator ? CLAUDIATOR_NODES : GEMINITOR_NODES;
  const classes = [
    "sai-icon",
    `sai-${isClaudiator ? "claudiator" : "geminitor"}`,
    agent.status === "running" ? "sai-running" : "",
    className,
  ].filter(Boolean).join(" ");

  return (
    <svg
      aria-hidden="true"
      className={classes}
      fill="none"
      height={size}
      viewBox="0 0 24 24"
      width={size}
    >
      <g className="sai-node-group">
        {nodes.map((node, index) => (
          <circle
            className={`sai-node sai-node-${node.motion}`}
            cx={node.cx}
            cy={node.cy}
            key={`${node.cx}-${node.cy}-${index}`}
            r={node.r}
          />
        ))}
      </g>
    </svg>
  );
}
