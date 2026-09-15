import "./voice-controls.css";

const SHAPE = [
  .08, .14, .42, .71, .86, .63, .44, .58, .79, .92, .74, .51, .29, .12, .06, .05, .07,
  .18, .47, .68, .88, .95, .77, .59, .66, .81, .62, .38, .24, .15, .09, .06, .04, .03,
];

export function VoiceSignal({ level, label }: { level: number; label?: string }) {
  const amplitude = Math.max(.04, Math.min(1, level));
  const top = SHAPE.map((height, index) => `${index * 3},${20 - height * 18}`).join(" ");
  const bottom = SHAPE.map((height, index) => `${index * 3},${20 + height * 18}`).reverse().join(" ");
  return (
    <svg className="vc-signal" viewBox="0 0 99 40" preserveAspectRatio="none" role={label ? "img" : undefined} aria-label={label} aria-hidden={label ? undefined : true}>
      <polygon points={`${top} ${bottom}`} style={{ transform: `scaleY(${amplitude})` }} />
    </svg>
  );
}
