export const AGENT_SESSIONS_CHANGED = "clgo-agent-sessions-changed";

export function notifyAgentSessionsChanged() {
  window.dispatchEvent(new Event(AGENT_SESSIONS_CHANGED));
}
