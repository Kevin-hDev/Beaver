import { useEffect, useState } from "react";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type {
  CompressionBandSettings,
  CompressionProfile,
  CompressionLimitsView,
  CompressionWindowBand,
} from "@/types/compression-profile.generated";
import { CompressionBudgetPreview } from "./compression-budget-preview";
import { CompressionContentSection } from "./compression-content-section";
import { CompressionRangeTabs } from "./compression-range-tabs";
import { CompressionSummarySection } from "./compression-summary-section";
import { CompressionTriggerSection } from "./compression-trigger-section";
import { CompressionUnder64Warning } from "./compression-under64-warning";

interface CompressionProfileEditorProps {
  profile: CompressionProfile;
  currentWindow: number;
  controller: CompressionProfilesController;
  limits: CompressionLimitsView;
  automaticEnabled: boolean;
}

function bandForWindow(
  window: number,
  limits: CompressionLimitsView,
): CompressionWindowBand | null {
  if (window <= 0) return null;
  if (window < limits.under_64k_upper_exclusive) return "under_64k";
  if (window < limits.compact_upper_exclusive) return "compact";
  return "large";
}

function bandSettings(
  profile: CompressionProfile,
  band: CompressionWindowBand,
): CompressionBandSettings {
  return band === "under_64k" ? profile.under_64k : profile[band];
}

function withBand(
  profile: CompressionProfile,
  band: CompressionWindowBand,
  settings: CompressionBandSettings,
): CompressionProfile {
  if (band === "under_64k") return { ...profile, under_64k: settings };
  return { ...profile, [band]: settings };
}

export function CompressionProfileEditor({
  profile: confirmed,
  currentWindow,
  controller,
  limits,
  automaticEnabled,
}: CompressionProfileEditorProps) {
  const [profile, setProfile] = useState(confirmed);
  const [editedBand, setEditedBand] = useState<CompressionWindowBand>(
    bandForWindow(currentWindow, limits) ?? "compact",
  );
  const band = bandSettings(profile, editedBand);
  const under64Disabled = editedBand === "under_64k" && !profile.allow_under_64k;

  useEffect(() => {
    // La révision confirmée évolue après l'enregistrement. Garder le même
    // composant préserve le focus et la position du curseur dans les prompts.
    // eslint-disable-next-line react-hooks/set-state-in-effect -- synchronisation d'une autorité backend persistée
    setProfile((current) => {
      if (current.revision === confirmed.revision) return current;
      const currentContent = { ...current, revision: confirmed.revision };
      return JSON.stringify(currentContent) === JSON.stringify(confirmed)
        ? currentContent
        : confirmed;
    });
  }, [confirmed]);

  const update = (next: CompressionProfile) => {
    setProfile(next);
    void controller.save(next);
  };
  const updateBand = (next: CompressionBandSettings) => {
    update(withBand(profile, editedBand, next));
  };

  return (
    <div className="cpe-editor">
      <CompressionRangeTabs
        edited={editedBand}
        active={bandForWindow(currentWindow, limits)}
        onChange={setEditedBand}
      />
      <CompressionUnder64Warning
        toggleVisible={editedBand === "under_64k"}
        enabled={profile.allow_under_64k}
        onChange={(allow_under_64k) => update({ ...profile, allow_under_64k })}
      />
      <div className="cpa-body">
        <CompressionContentSection
          band={band}
          limits={limits}
          disabled={under64Disabled}
          onChange={updateBand}
          onCopy={() => update({
            ...profile,
            under_64k: band,
            compact: band,
            large: band,
          })}
        />
        <CompressionTriggerSection
          profile={profile}
          limits={limits}
          disabled={!automaticEnabled}
          onProfileChange={update}
        />
        <CompressionSummarySection
          profile={profile}
          band={band}
          limits={limits}
          quantityDisabled={under64Disabled}
          onProfileChange={update}
          onBandChange={updateBand}
          onResetPrompts={() => { void controller.resetPrompts(profile.id); }}
        />
      </div>
      <footer className="cpa-foot">
        <CompressionBudgetPreview
          profileId={profile.id}
          profileRevision={confirmed.revision}
          band={editedBand}
        />
      </footer>
    </div>
  );
}
