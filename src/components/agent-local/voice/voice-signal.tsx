import "./voice-controls.css";

export function VoiceSignal({ level }: { level: number }) {
  const bars = [0.12, 0.28, 0.5, 0.72, 0.9];
  return (
    <span className="vc-signal" aria-hidden="true">
      {bars.map((threshold) => <span key={threshold} className={level >= threshold ? "vc-bar vc-bar-on" : "vc-bar"} />)}
    </span>
  );
}
