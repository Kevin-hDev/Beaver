import "./beaver-brand-icon.css";

export function BeaverBrandIcon({ size = 40 }: { size?: number | string }) {
  return (
    <span className="bbi-root" style={{ width: size, height: size }} aria-hidden="true">
      <span className="bbi-surface" />
      <span className="bbi-mark" />
    </span>
  );
}
