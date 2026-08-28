interface ThemedIconProps {
  darkSrc: string;
  lightSrc: string;
  size?: number | string;
  alt?: string;
  style?: React.CSSProperties;
  className?: string;
}

export function ThemedIcon({ darkSrc, lightSrc, size = 16, alt = "", style, className = "" }: ThemedIconProps) {
  const s = { width: size, height: size, ...style };
  return (
    <>
      <img src={darkSrc} alt={alt} className={`themed-icon-dark ${className}`} style={s} />
      <img src={lightSrc} alt={alt} className={`themed-icon-light ${className}`} style={s} />
    </>
  );
}
