import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { getVersion, getTauriVersion } from "@tauri-apps/api/app";
import { open } from "@tauri-apps/plugin-shell";
import logo from "@/assets/logo.png";
import { ArrowSquareOut } from "@/components/ui/icons";
import { IS_LINUX, IS_MAC, IS_WINDOWS } from "@/lib/platform";
import { BRAND } from "@/lib/brand";
import { SettingsCard } from "./settings-card";
import "./about-settings.css";

const GITHUB_URL = `https://github.com/${BRAND.repository}`;

export function AboutSettings() {
  const { t } = useTranslation();
  const [appVersion, setAppVersion] = useState("");
  const [tauriVersion, setTauriVersion] = useState("");

  useEffect(() => {
    getVersion().then(setAppVersion).catch(() => {});
    getTauriVersion().then(setTauriVersion).catch(() => {});
  }, []);

  const platform = IS_MAC ? "macOS"
    : IS_WINDOWS ? "Windows"
    : IS_LINUX ? "Linux" : "—";

  return (
    <div className="as-root">
      <div className="as-inner">
        <div className="as-hero">
          <img src={logo} alt="" className="as-app-logo" />
          <h2 className="as-title">
            {BRAND.displayName}
          </h2>
          <span className="as-subtitle">
            {t("about.description")}
          </span>
        </div>

        <SettingsCard>
          <InfoRow label={t("about.version")} value={appVersion || "—"} />
          <InfoRow label={t("about.tauri")} value={tauriVersion || "—"} />
          <InfoRow label={t("about.os")} value={platform} />
        </SettingsCard>

        <div className="as-links">
          <button
            type="button"
            onClick={() => void open(GITHUB_URL)}
            className="btn btn-sm btn-secondary as-github-btn"
          >
            {t("about.viewOnGithub")} <ArrowSquareOut size="var(--icon-sm)" />
          </button>
        </div>
      </div>
    </div>
  );
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="as-info-row">
      <span className="as-info-label">
        {label}
      </span>
      <span className="as-info-value">{value}</span>
    </div>
  );
}
