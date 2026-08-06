import { useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { vi } from "vitest";
import { SettingsTab } from "../settings-tab";
import { PanelSlotProvider, PanelSlotTarget } from "@/components/layout/panel-slots";
import { DEFAULT_APP_NAV, type DeepPartial, type SettingsNavState } from "@/types/navigation";
import { resetMascotSettingsMock } from "./settings-tab-test-mascot";

export const CHILD_COMMANDS = new Set([
  "list_ollama_models",
  "list_mcp_connectors",
  "gateway_get_config",
  "gateway_status",
  "list_llm_providers_catalog",
  "list_search_providers_catalog",
  "list_forecast_providers_catalog",
  "list_configured_providers",
  "list_forecast_models",
  "get_memory_overview",
  "get_system_prompt_setting",
]);
const FAILED_COMMANDS = new Set<string>();

vi.mock("@tauri-apps/api/core", async () => {
  const { vi: mockVi } = await import("vitest");
  const data = await import("./settings-tab-test-data");
  const mascot = await import("./settings-tab-test-mascot");

  return {
    invoke: mockVi.fn((cmd: string, args?: Record<string, unknown>) => {
      const mascotResult = mascot.mascotCommandResult(cmd, args);
      if (mascotResult.handled) return Promise.resolve(mascotResult.value);
      if (cmd === "get_advanced_settings") return Promise.resolve({
        autostart: false, start_hidden: false, show_tray: true, default_model: "", keep_alive: "5m",
        allowed_paths: ["/"], hardware_accel: "gpu", multi_model: false, show_gpu_status: false,
        compression_enabled: true, compression_threshold: 85, response_language: "",
        link_preview_enabled: true, ollama_setup_skipped: false,
      });
      if (cmd === "get_agent_settings") return Promise.resolve(data.agentSettings());
      if (cmd === "get_memory_overview") return Promise.resolve(data.memoryOverview());
      if (cmd === "get_system_prompt_setting") return Promise.resolve({
        content: "Beaver instructions",
        source: "beaver",
        selection: "default",
        disabled: false,
      });
      if (FAILED_COMMANDS.has(cmd)) return Promise.reject(new Error("test failure"));
      if (cmd === "get_memory_project_topics") {
        const overview = data.memoryOverview();
        const scope = overview.otherProjects.find((item) => item.id === args?.projectId);
        return Promise.resolve(scope && {
          ...scope,
          topicsLoaded: true,
          topics: [{
            ...overview.global.topics[0],
            id: "029f951b-38a1-7882-bf2f-0784e266c911",
            title: "Mémoire projet",
            path: "/memory/projects/bbbbbbbbbbbbbbbbbbbbbbbb/topics/project.md",
          }],
        });
      }
      if (cmd === "set_memory_mode") {
        return Promise.resolve({ ...data.memoryOverview().settings, mode: args?.mode });
      }
      if (cmd === "set_memory_context_budget") {
        return Promise.resolve({
          ...data.memoryOverview().settings,
          contextBudgetTokens: args?.tokens,
        });
      }
      if (cmd === "archive_memory_topic") {
        const overview = data.memoryOverview();
        return Promise.resolve({
          ...overview,
          global: { ...overview.global, topicCount: 0, totalBytes: 0, topics: [] },
        });
      }
      if (cmd === "read_file_preview") return Promise.resolve("# Interface compacte");
      if (cmd === "list_agent_tool_catalog") return Promise.resolve(data.agentToolCatalog());
      if (cmd === "list_agent_tool_groups") return Promise.resolve(data.agentToolGroups());
      if (cmd === "set_agent_tool_enabled") {
        const enabled = args?.enabled === false ? [] : ["load_skill"];
        return Promise.resolve({ permission_mode: "auto", enabled_optional_tools: enabled });
      }
      if (cmd === "set_agent_tool_group_enabled") {
        const enabled = args?.enabled === false ? [] : ["load_skill"];
        return Promise.resolve({ permission_mode: "auto", enabled_optional_tools: enabled });
      }
      if (cmd === "is_ollama_installed") return Promise.resolve(true);
      if (cmd === "get_modelfile") return Promise.resolve("FROM llama3.2:latest\nPARAMETER temperature 0.7\n");
      if (cmd === "get_selected_forecast_model") return Promise.resolve("chronos-bolt-small");
      if (cmd === "list_configured_providers") return Promise.resolve(["groq", "brave", "nixtla"]);
      if (cmd === "list_oauth_provider_statuses") {
        const xaiConnected = globalThis.localStorage?.getItem("test:xai-connected") === "true";
        return Promise.resolve([
          { id: "openai", display_name: "OpenAI", connected: true, account: "user@example.com", experimental: false },
          { id: "moonshot", display_name: "Moonshot AI", connected: false, account: null, experimental: true },
          { id: "xai", display_name: "xAI", connected: xaiConnected, account: null, experimental: false },
        ]);
      }
      if (cmd === "disconnect_oauth_provider" && args?.providerId === "xai") {
        globalThis.localStorage?.removeItem("test:xai-connected");
        return Promise.resolve();
      }
      if (cmd === "start_oauth_provider_login" || cmd === "disconnect_oauth_provider" || cmd === "cancel_oauth_provider_login") return Promise.resolve();
      if (cmd === "list_llm_providers_catalog") {
        return Promise.resolve([data.provider("groq", "Groq", "llm"), data.provider("mistral", "Mistral", "llm")]);
      }
      if (cmd === "list_search_providers_catalog") return Promise.resolve([data.provider("brave", "Brave", "search")]);
      if (cmd === "list_forecast_providers_catalog") return Promise.resolve([data.provider("nixtla", "Nixtla", "forecast")]);
      if (cmd === "list_forecast_models") return Promise.resolve(data.forecastModels());
      if (cmd === "list_mcp_connectors") return Promise.resolve(data.mcpConnectors());
      if (cmd === "gateway_get_config") return Promise.resolve(data.gatewayConfig());
      if (cmd === "gateway_status") return Promise.resolve(data.gatewayStatus());
      if (cmd === "list_ollama_models") return Promise.resolve(data.ollamaModels());
      return Promise.resolve([]);
    }),
  };
});

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en", changeLanguage: vi.fn() },
  }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key },
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

const noop = vi.fn();

export function SettingsHarness() {
  const [navState, setNavState] = useState<SettingsNavState>(DEFAULT_APP_NAV.settings);
  const handleNavChange = useCallback((partial: DeepPartial<SettingsNavState>) => {
    setNavState((current) => ({ ...current, ...partial }) as SettingsNavState);
  }, []);

  return (
    <PanelSlotProvider>
      <SettingsTab
        themeChoice="dark"
        onThemeChange={noop}
        navState={navState}
        onNavChange={handleNavChange}
        onNavReplace={handleNavChange}
        listFocused={false}
      />
      <div data-testid="settings-list"><PanelSlotTarget name="list" /></div>
      <div data-testid="settings-detail"><PanelSlotTarget name="detail" /></div>
    </PanelSlotProvider>
  );
}

export function resetSettingsTestEnvironment() {
  vi.mocked(invoke).mockClear();
  FAILED_COMMANDS.clear();
  resetMascotSettingsMock();
  const store = new Map<string, string>();
  Object.defineProperty(globalThis, "localStorage", {
    configurable: true,
    value: {
      getItem: (key: string) => store.get(key) ?? null,
      setItem: (key: string, value: string) => store.set(key, value),
      removeItem: (key: string) => store.delete(key),
      clear: () => store.clear(),
    },
  });
}

export function failInvokeCommand(command: string) {
  FAILED_COMMANDS.add(command);
}

export function restoreInvokeCommand(command: string) {
  FAILED_COMMANDS.delete(command);
}

export function invokedCommands() {
  return vi.mocked(invoke).mock.calls.map(([cmd]) => cmd);
}

export function invokeCalls() {
  return vi.mocked(invoke).mock.calls;
}

export function setXaiOAuthState({ ready, connected = false }: { ready: boolean; connected?: boolean }) {
  void ready;
  if (connected) localStorage.setItem("test:xai-connected", "true");
  else localStorage.removeItem("test:xai-connected");
}
