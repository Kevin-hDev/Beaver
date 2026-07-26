import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ActiveBadge, ScheduleBadge } from "../badges";
import type { WakeupSchedule } from "@/types/wakeup";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/i18n", () => ({
  default: { t: (key: string) => key, language: "fr" },
}));

// Mock l'icône Clock (vient de @phosphor-icons/react) pour éviter le rendu SVG.
vi.mock("@/components/ui/icons", () => ({
  Clock: () => null,
}));

afterEach(cleanup);

describe("ActiveBadge", () => {
  it("affiche le label actif avec la classe active", () => {
    const { getByText } = render(<ActiveBadge active={true} />);
    const badge = getByText("heartbeat.badges.active");
    expect(badge.className).toContain("wk-badge-active");
  });

  it("affiche le label inactif avec la classe inactive", () => {
    const { getByText } = render(<ActiveBadge active={false} />);
    const badge = getByText("heartbeat.badges.inactive");
    expect(badge.className).toContain("wk-badge-inactive");
  });

  it("utilise les clés i18n pour les labels", () => {
    const { getByText } = render(<ActiveBadge active={true} />);
    // Le mock t retourne la clé → on vérifie la clé exacte.
    expect(getByText("heartbeat.badges.active")).toBeTruthy();
  });
});

describe("ScheduleBadge", () => {
  it("affiche le schedule formaté pour un wakeup daily", () => {
    const schedule: WakeupSchedule = { kind: "daily", time: "08:00" };
    const { container } = render(<ScheduleBadge schedule={schedule} />);
    const badge = container.querySelector(".wk-badge-schedule");
    expect(badge).toBeTruthy();
    // formatSchedule produit un texte — on vérifie juste qu'il y a du contenu.
    expect(badge?.textContent).toBeTruthy();
  });

  it("affiche le schedule formaté pour un wakeup weekly", () => {
    const schedule: WakeupSchedule = {
      kind: "weekly",
      weekday: 1,
      time: "09:30",
    };
    const { container } = render(<ScheduleBadge schedule={schedule} />);
    const badge = container.querySelector(".wk-badge-schedule");
    // formatSchedule produit "mar. 09h30" (formatage français).
    expect(badge?.textContent).toContain("09h30");
  });

  it("applique la classe wk-badge-schedule", () => {
    const schedule = { kind: "once", datetime: "2026-01-01T10:00" } as WakeupSchedule;
    const { container } = render(<ScheduleBadge schedule={schedule} />);
    expect(container.querySelector(".wk-badge-schedule")).toBeTruthy();
  });
});
