import { useState, type CSSProperties } from "react";
import "./voice-controls.css";

// Une mesure toutes les 50 ms côté Rust : 160 barres ≈ 8 s d'historique,
// borné — les mesures plus anciennes sortent du tableau.
const BAR_COUNT = 160;

export function VoiceSignal({ level, tick, label }: { level: number; tick: number; label?: string }) {
  const [bars, setBars] = useState<number[]>(() => Array<number>(BAR_COUNT).fill(0));
  const [lastTick, setLastTick] = useState<number | null>(null);
  // Ajout pendant le rendu, gardé par le tick : chaque instant de mesure ne
  // produit qu'une barre, même si le composant re-rend plusieurs fois.
  if (lastTick !== tick) {
    setLastTick(tick);
    // Échelle en décibels, plancher à −40 dB : l'oreille perçoit le volume en
    // logarithme — la crête brute d'une voix normale (0,1 à 0,4) dessinerait
    // des barres minuscules alors qu'on l'entend forte.
    const loudness = level > 0 ? Math.max(0, Math.min(1, 1 + (20 * Math.log10(level)) / 40)) : 0;
    setBars([...bars.slice(1), loudness]);
  }
  return (
    <div className="vc-signal" role={label ? "img" : undefined} aria-label={label} aria-hidden={label ? undefined : true}>
      {bars.map((height, index) => (
        <span key={index} style={{ "--vc-h": height } as CSSProperties} />
      ))}
    </div>
  );
}
