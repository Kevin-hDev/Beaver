import { useEffect, useState } from "react";
import type { CompressionProfilesController } from "@/hooks/use-compression-profiles";
import type {
  CompressionBandSettings,
  CompressionProfile,
  CompressionWindowBand,
} from "@/types/compression-profile.generated";
import { CompressionBudgetPreview } from "./compression-budget-preview";
import { CompressionContentSection } from "./compression-content-section";
import { CompressionFailureSection } from "./compression-failure-section";
import { CompressionRangeTabs } from "./compression-range-tabs";
import { CompressionSummarySection } from "./compression-summary-section";
import { CompressionTriggerSection } from "./compression-trigger-section";
import { CompressionUnder64Warning } from "./compression-under64-warning";

interface CompressionProfileEditorProps {
  profile: CompressionProfile;
  currentWindow: number;
  controller: CompressionProfilesController;
}

function bandForWindow(window: number): CompressionWindowBand | null {
  if (window <= 0) return null;
  if (window < 64_000) return "under_64k";
  if (window < 128_000) return "compact";
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
}: CompressionProfileEditorProps) {
  const [profile, setProfile] = useState(confirmed);
  const [editedBand, setEditedBand] = useState<CompressionWindowBand>(
    bandForWindow(currentWindow) ?? "compact",
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
        active={bandForWindow(currentWindow)}
        onChange={setEditedBand}
      />
      <CompressionUnder64Warning
        toggleVisible={editedBand === "under_64k"}
        enabled={profile.allow_under_64k}
        onChange={(allow_under_64k) => update({ ...profile, allow_under_64k })}
      />
      <div
        className="cpa-body"
        data-disabled={under64Disabled ? "true" : undefined}
      >
        <CompressionTriggerSection
          profile={profile}
          band={band}
          disabled={under64Disabled}
          onProfileChange={update}
          onBandChange={updateBand}
        />
        <CompressionSummarySection
          profile={profile}
          band={band}
          disabled={under64Disabled}
          onProfileChange={update}
          onBandChange={updateBand}
        />
        <CompressionContentSection
          band={band}
          disabled={under64Disabled}
          onChange={updateBand}
          onCopy={() => update({
            ...profile,
            under_64k: band,
            compact: band,
            large: band,
          })}
        />
        <CompressionFailureSection
          profile={profile}
          disabled={under64Disabled}
          onChange={update}
        />
      </div>
      <footer className="cpa-foot">
        <CompressionBudgetPreview
          key={editedBand}
          profileId={profile.id}
          profileRevision={confirmed.revision}
          band={editedBand}
          currentWindow={currentWindow}
        />
      </footer>
    </div>
  );
}
