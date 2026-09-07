import { AgentLocalTab } from "@/components/agent-local/agent-local-tab";
import { HeartbeatTab } from "@/components/heartbeat/heartbeat-tab";
import { PersonalityTab } from "@/components/personality/personality-tab";
import { SettingsTab } from "@/components/settings/settings-tab";
import { useSlotOccupantByTarget } from "@/features/extension-ui/slot-contexts";
import { SlotRenderer } from "@/features/extension-ui/slot-renderer";
import type { MainTabId, SlotOccupant } from "@/features/extension-ui/slot-types";
import {
  StandardTabContent,
  useStandardEntry,
} from "@/features/extension-ui/standard/standard-contributions";
import { AppSurfaceActivityProvider } from "./app-surface-activity";
import type { NavPanel } from "@/hooks/use-panel-focus";
import type { ThemeChoice } from "@/hooks/use-theme";
import type {
  AgentLocalNavState,
  AgentLocalWorkspaceState,
  AppNavState,
  DeepPartial,
  SettingsNavState,
} from "@/types/navigation";

interface CoreMainTabContentProps {
  activeTab: MainTabId;
  nav: AppNavState;
  agentNavState: AgentLocalNavState;
  themeChoice: ThemeChoice;
  focusedPanel: NavPanel;
  onWakeupChange: (id: string | null) => void;
  onPathChange: (path: string | null) => void;
  onSessionChange: (id: string | null) => void;
  onAgentNavChange: (partial: Partial<AgentLocalWorkspaceState>) => void;
  onWorkspaceClear: (sessionId: string) => void;
  onThemeChange: (theme: ThemeChoice) => void;
  onSettingsNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onSettingsNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function CoreMainTabContent(props: CoreMainTabContentProps) {
  const occupant = useSlotOccupantByTarget(props.activeTab, "tab");
  const agentLocal = useSlotOccupantByTarget("agent-local", "tab");
  const agentLocalActive = occupant?.target === "agent-local";
  const entry = useStandardEntry(occupant);
  /* Agent Local garde ses PTY et flux vivants ; seule son activité change. */
  const persistentAgentLocal = agentLocal ? (
    <AppSurfaceActivityProvider active={agentLocalActive}>
      <SlotRenderer
        placement={agentLocal.placement}
        occupantId={agentLocal.id}
        context={props}
        render={renderAgentLocal}
      />
    </AppSurfaceActivityProvider>
  ) : null;

  if (!occupant || agentLocalActive) return persistentAgentLocal;
  if (entry) return <>{persistentAgentLocal}<StandardTabContent entry={entry} /></>;
  return (
    <>{persistentAgentLocal}
      <SlotRenderer
        placement={occupant.placement}
        occupantId={occupant.id}
        context={props}
        render={renderCoreMainTab}
      />
    </>
  );
}

function renderCoreMainTab(
  occupant: SlotOccupant,
  context: CoreMainTabContentProps,
) {
  const listFocused = context.focusedPanel === "list" && context.activeTab === occupant.target;
  if (occupant.target === "heartbeat") {
    return (
      <HeartbeatTab
        activeWakeupId={context.nav.heartbeat.wakeupId}
        onWakeupChange={context.onWakeupChange}
        listFocused={listFocused}
      />
    );
  }
  if (occupant.target === "personality") {
    return (
      <PersonalityTab
        activePath={context.nav.personality.path}
        onPathChange={context.onPathChange}
        listFocused={listFocused}
      />
    );
  }
  if (occupant.target === "settings") {
    return (
      <SettingsTab
        themeChoice={context.themeChoice}
        onThemeChange={context.onThemeChange}
        navState={context.nav.settings}
        onNavChange={context.onSettingsNavChange}
        onNavReplace={context.onSettingsNavReplace}
        listFocused={listFocused}
        activeSessionId={context.nav.agentLocal.sessionId}
      />
    );
  }
  return null;
}

function renderAgentLocal(
  occupant: SlotOccupant,
  context: CoreMainTabContentProps,
) {
  if (occupant.target !== "agent-local") return null;
  return (
    <AgentLocalTab
      navState={context.agentNavState}
      onSessionChange={context.onSessionChange}
      onNavChange={context.onAgentNavChange}
      onWorkspaceClear={context.onWorkspaceClear}
      listFocused={context.focusedPanel === "list" && context.activeTab === occupant.target}
    />
  );
}
