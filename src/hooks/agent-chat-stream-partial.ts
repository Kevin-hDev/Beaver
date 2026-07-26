import type { ManagedStreamState } from "./agent-chat-stream-types";

export function markUnconfirmedContentAsWork(state: ManagedStreamState): ManagedStreamState {
  const hasWorkEvidence = state.completedSegments.some((seg) => (
    seg.phase === "work" || !!seg.thinking || seg.tools.length > 0
  )) || !!state.currentThinking || state.currentTools.length > 0;
  if (!hasWorkEvidence) return state;

  const completedSegments = state.completedSegments.map((seg) => (
    seg.phase ? seg : { ...seg, phase: "work" as const }
  ));
  const hasCurrent = !!state.currentContent || !!state.currentThinking || state.currentTools.length > 0;
  return {
    ...state,
    completedSegments,
    currentContentPhase: hasCurrent && !state.currentContentPhase ? "work" : state.currentContentPhase,
  };
}
