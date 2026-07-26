interface ThemedIconProps {
  darkSrc: string;
  lightSrc: string;
  size?: number | string;
  alt?: string;
  style?: React.CSSProperties;
}

export function ThemedIcon({ darkSrc, lightSrc, size = 16, alt = "", style }: ThemedIconProps) {
  const s = { width: size, height: size, ...style };
  return (
    <>
      <img src={darkSrc} alt={alt} className="themed-icon-dark" style={s} />
      <img src={lightSrc} alt={alt} className="themed-icon-light" style={s} />
    </>
  );
}
