import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ReasoningContinuitySelector } from "../reasoning-continuity-selector";
import type { ContinuityCapability } from "@/types/agent-session.generated";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

afterEach(cleanup);

const optional: ContinuityCapability = {
  requirement: "optional",
  local_available: true,
  remote_available: false,
  state: "available",
  explanation_key: "agentLocal.continuityOptional",
};

function agentLocalTranslations(value: unknown): Record<string, string> {
  if (typeof value !== "object" || value === null || !("agentLocal" in value)) {
    throw new Error("invalid test translation file");
  }
  const agentLocal = value.agentLocal;
  if (typeof agentLocal !== "object" || agentLocal === null) {
    throw new Error("invalid test translation group");
  }
  return agentLocal as Record<string, string>;
}

function renderSelector(capability: ContinuityCapability | undefined, setting = "off") {
  const onChange = vi.fn();
  const view = render(
    <ReasoningContinuitySelector
      capability={capability}
      setting={setting as "off" | "local" | "remote"}
      onChange={onChange}
    />,
  );
  return { ...view, onChange };
}

describe("ReasoningContinuitySelector", () => {
  it("reste absent quand Rust ne confirme pas une route et un modèle live", () => {
    const { container } = renderSelector(undefined);

    expect(container.firstChild).toBeNull();
  });

  it("laisse Off par défaut et propose seulement Local lorsque la capacité optional le permet", () => {
    renderSelector(optional);

    expect(screen.getByRole("radiogroup", { name: "agentLocal.continuityTitle" })).toBeTruthy();
    expect(screen.getByRole("radio", { name: "agentLocal.continuityOff" })).toBeChecked();
    expect(screen.getByRole("radio", { name: "agentLocal.continuityLocal" })).toBeTruthy();
    expect(screen.queryByRole("radio", { name: "agentLocal.continuityRemote" })).toBeNull();
  });

  it("propose Distant seulement lorsque Rust le marque disponible et documenté", () => {
    renderSelector({ ...optional, remote_available: true });

    expect(screen.getByRole("radio", { name: "agentLocal.continuityRemote" })).toBeTruthy();
  });

  it("verrouille une continuité required sans offrir de désactivation", () => {
    renderSelector({
      requirement: "required",
      local_available: true,
      remote_available: true,
      state: "locked",
      explanation_key: "agentLocal.continuityRequired",
    }, "local");

    expect(screen.getByText("agentLocal.continuityRequired")).toBeTruthy();
    expect(screen.queryByRole("radio", { name: "agentLocal.continuityOff" })).toBeNull();
    expect(screen.queryByRole("radiogroup")).toBeNull();
  });

  it("réévalue la capacité et affiche une barrière après un changement de compte ou modèle", () => {
    const { rerender } = renderSelector(optional);
    expect(screen.getByRole("radiogroup")).toBeTruthy();

    rerender(
      <ReasoningContinuitySelector
        capability={undefined}
        setting="off"
        onChange={vi.fn()}
      />,
    );

    expect(screen.queryByRole("radiogroup")).toBeNull();
  });

  it("envoie uniquement un choix permis par la capacité Rust", () => {
    const { onChange } = renderSelector({ ...optional, remote_available: true });

    fireEvent.click(screen.getByRole("radio", { name: "agentLocal.continuityRemote" }));

    expect(onChange).toHaveBeenCalledWith("remote");
  });

  it("ne promet pas de chiffrement des sessions locales et toutes les clés existent", () => {
    const keys = [
      "continuityTitle",
      "continuityOff",
      "continuityLocal",
      "continuityRemote",
      "continuityOptional",
      "continuityRequired",
    ];
    for (const locale of ["fr", "en", "es", "de", "it", "zh", "ja"]) {
      // Les sept fichiers sont choisis dans une liste fermée du dépôt.
      // eslint-disable-next-line security/detect-non-literal-fs-filename
      const source = readFileSync(`src/i18n/${locale}.json`, "utf8");
      const translations = agentLocalTranslations(JSON.parse(source) as unknown);
      for (const key of keys) expect(translations[key]).toEqual(expect.any(String));
      expect(translations.continuityLocal).not.toMatch(/encrypt|chiffr|verschl|cifrad/i);
    }
  });
});
