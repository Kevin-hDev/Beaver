import type { CSSProperties, SVGProps } from "react";

type Size = number | string | undefined;

export function svgSizeProps(
  size: Size,
  style?: CSSProperties,
): Pick<SVGProps<SVGSVGElement>, "width" | "height" | "style"> {
  if (typeof size === "number") {
    return { width: size, height: size, style };
  }
  if (typeof size === "string") {
    return { style: { width: size, height: size, ...style } };
  }
  return { style };
}
