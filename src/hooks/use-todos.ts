import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import type { AgentSession, AgentTodoItem } from "@/types/agent";

export function useTodos(sessionId: string | undefined) {
  const [todos, setTodos] = useState<AgentTodoItem[]>([]);

  useEffect(() => {
    let cancelled = false;
    let liveTodosSeen = false;
    queueMicrotask(() => {
      if (!cancelled && !liveTodosSeen) setTodos([]);
    });
    if (!sessionId) return () => { cancelled = true; };

    const applySnapshot = (snapshot: StreamSnapshot | null) => {
      if (cancelled || !snapshot || snapshot.projection.todos === null) return;
      liveTodosSeen = true;
      setTodos(snapshot.projection.todos);
    };
    const unsubscribe = agentStreamManager.subscribe(sessionId, applySnapshot);
    applySnapshot(agentStreamManager.getSnapshot(sessionId));

    void invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((session) => {
        if (!cancelled && !liveTodosSeen) setTodos(session.todos ?? []);
      })
      .catch(() => { /* conserve la dernière liste confirmée */ });

    return () => {
      cancelled = true;
      unsubscribe();
    };
  }, [sessionId]);

  return todos;
}
