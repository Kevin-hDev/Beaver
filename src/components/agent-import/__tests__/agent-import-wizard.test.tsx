import {
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import i18n from "@/i18n";
import type {
  AgentImportItem,
  AgentSourceSummary,
} from "@/types/agent-import";
import { AgentImportWizard } from "../agent-import-wizard";

const invokeMock = vi.mocked(invoke);
const sourceNames = [
  "Claude Code",
  "Codex",
  "Agents",
  "Hermes Agent",
  "Qwen Code",
  "ZCode",
  "OpenClaw",
  "OpenCode",
  "Kimi Code",
];

function item(id: string, kind: "document" | "rule" | "skill"): AgentImportItem {
  return {
    id,
    name: id,
    description: `Test ${kind}`,
    sourceId: "claude",
    sourceName: "Claude Code",
    kind,
    selected: true,
    available: true,
    updateAvailable: false,
  };
}

function sources(): AgentSourceSummary[] {
  return sourceNames.map((displayName, index) => ({
    id: displayName.toLowerCase().replaceAll(" ", "-"),
    displayName,
    status: index === 0 ? "detected" : "missing",
    partial: false,
    configured: false,
    enabled: false,
    documents: [],
    rules: index === 0 ? [item("style", "rule"), item("testing", "rule")] : [],
    skills: index === 0 ? [item("review", "skill"), item("security", "skill")] : [],
  }));
}

function mockScan(result: AgentSourceSummary[]) {
  invokeMock.mockImplementation((command) => {
    if (command === "scan_external_agent_sources") return Promise.resolve(result);
    if (command === "save_external_agent_source_selection") {
      return Promise.resolve({ saved: true, conflicts: [] });
    }
    return Promise.resolve(undefined);
  });
}

beforeEach(() => {
  invokeMock.mockReset();
  mockScan(sources());
});

describe("AgentImportWizard", () => {
  it("affiche les neuf sources et sélectionne tous les skills par défaut", async () => {
    render(<AgentImportWizard />);

    for (const name of sourceNames) {
      expect(await screen.findByText(name)).toBeInTheDocument();
    }
    fireEvent.click(screen.getByText("Claude Code"));

    const checkboxes = screen.getAllByRole("checkbox");
    expect(checkboxes).toHaveLength(4);
    expect(checkboxes.every((checkbox) => (checkbox as HTMLInputElement).checked)).toBe(true);
  });

  it("gère Tout, Rien et une sélection personnalisée pour les skills", async () => {
    render(<AgentImportWizard />);
    fireEvent.click(await screen.findByText("Claude Code"));
    const section = screen
      .getByText(i18n.t("agentImport.sections.skills"))
      .closest("section");
    expect(section).not.toBeNull();
    const skillSection = within(section as HTMLElement);
    const checkboxes = skillSection.getAllByRole("checkbox");

    fireEvent.click(skillSection.getByRole("button", {
      name: i18n.t("agentImport.actions.none"),
    }));
    expect(checkboxes.every((checkbox) => !(checkbox as HTMLInputElement).checked)).toBe(true);

    fireEvent.click(skillSection.getByRole("button", {
      name: i18n.t("agentImport.actions.all"),
    }));
    fireEvent.click(checkboxes[1]);
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("agentImport.actions.confirmSource"),
    }));

    await waitFor(() => {
      const saveCall = invokeMock.mock.calls.find(
        ([command]) => command === "save_external_agent_source_selection",
      );
      expect(saveCall?.[1]).toMatchObject({
        selection: {
          sourceId: "claude-code",
          skillMode: "custom",
          selectedSkillIds: ["review"],
        },
        replaceDocuments: false,
      });
    });
  });

  it("propose Tout et Rien pour les règles", async () => {
    render(<AgentImportWizard />);
    fireEvent.click(await screen.findByText("Claude Code"));
    const section = screen
      .getByText(i18n.t("agentImport.sections.rules"))
      .closest("section");
    expect(section).not.toBeNull();
    const ruleSection = within(section as HTMLElement);

    expect(ruleSection.getByRole("button", {
      name: i18n.t("agentImport.actions.all"),
    })).toBeInTheDocument();
    fireEvent.click(ruleSection.getByRole("button", {
      name: i18n.t("agentImport.actions.none"),
    }));

    expect(ruleSection.getAllByRole("checkbox").every(
      (checkbox) => !(checkbox as HTMLInputElement).checked,
    )).toBe(true);
  });

  it("permet de continuer l'onboarding sans configurer de source", async () => {
    const onContinue = vi.fn();
    const onBack = vi.fn();
    render(<AgentImportWizard onContinue={onContinue} onBack={onBack} />);

    await screen.findByText("Claude Code");
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("onboarding.common.back"),
    }));
    fireEvent.click(screen.getByRole("button", {
      name: i18n.t("onboarding.common.continue"),
    }));

    expect(onBack).toHaveBeenCalledOnce();
    expect(onContinue).toHaveBeenCalledOnce();
    expect(invokeMock).not.toHaveBeenCalledWith(
      "save_external_agent_source_selection",
      expect.anything(),
    );
  });
});
