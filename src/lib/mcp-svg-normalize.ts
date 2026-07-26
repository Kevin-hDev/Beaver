const STYLE_BLOCK_RE = /<style\b[^>]*>([\s\S]*?)<\/style>/gi;
const CLASS_RULE_RE = /\.([_a-zA-Z][\w-]*)\s*\{([^}]*)\}/g;
const CLASS_ATTR_RE = /\sclass="([^"]*)"/;
const STYLE_ATTR_RE = /\sstyle="([^"]*)"/;

const SVG_STYLE_ATTRS = new Set([
  "clip-path",
  "clip-rule",
  "display",
  "fill",
  "fill-opacity",
  "fill-rule",
  "filter",
  "mask",
  "opacity",
  "paint-order",
  "stop-color",
  "stop-opacity",
  "stroke",
  "stroke-dasharray",
  "stroke-dashoffset",
  "stroke-linecap",
  "stroke-linejoin",
  "stroke-miterlimit",
  "stroke-opacity",
  "stroke-width",
]);

type SvgDeclarations = Record<string, string>;

export function prepareMcpSvg(raw: string, prefix: string): string {
  const classStyles = collectClassStyles(raw);
  const withoutStyleBlocks = raw.replace(STYLE_BLOCK_RE, "");
  const withPresentationAttrs = applyElementStyles(withoutStyleBlocks, classStyles);
  return scopeSvgIds(withPresentationAttrs, prefix);
}

function collectClassStyles(svg: string): Map<string, SvgDeclarations> {
  const styles = new Map<string, SvgDeclarations>();
  for (const block of svg.matchAll(STYLE_BLOCK_RE)) {
    for (const rule of block[1].matchAll(CLASS_RULE_RE)) {
      const declarations = parseDeclarations(rule[2]);
      if (Object.keys(declarations).length > 0) {
        styles.set(rule[1], declarations);
      }
    }
  }
  return styles;
}

function applyElementStyles(svg: string, styles: Map<string, SvgDeclarations>): string {
  return svg.replace(/<([a-zA-Z][\w:-]*)([^>]*)>/g, (match, tag: string, attrs: string) => {
    if (tag.startsWith("/")) return match;
    const hasStyleAttr = STYLE_ATTR_RE.test(attrs);
    const classDeclarations = declarationsForClasses(attrs, styles);
    const styleDeclarations = declarationsForStyle(attrs);
    const declarations = { ...classDeclarations, ...styleDeclarations };
    if (Object.keys(declarations).length === 0) {
      return hasStyleAttr ? `<${tag}${attrs.replace(/\sstyle="[^"]*"/g, "")}>` : match;
    }
    const cleanedAttrs = removeAttributes(
      attrs.replace(/\sstyle="[^"]*"/g, ""),
      Object.keys(declarations),
    );
    const selfClosing = /\/\s*$/.test(cleanedAttrs);
    const baseAttrs = selfClosing ? cleanedAttrs.replace(/\/\s*$/, "") : cleanedAttrs;
    return `<${tag}${baseAttrs}${attrsToString(declarations)}${selfClosing ? " /" : ""}>`;
  });
}

function declarationsForClasses(attrs: string, styles: Map<string, SvgDeclarations>): SvgDeclarations {
  const classAttr = attrs.match(CLASS_ATTR_RE);
  if (!classAttr) return {};
  return classAttr[1]
    .split(/\s+/)
    .filter(Boolean)
    .reduce<SvgDeclarations>((acc, className) => ({ ...acc, ...styles.get(className) }), {});
}

function declarationsForStyle(attrs: string): SvgDeclarations {
  const styleAttr = attrs.match(STYLE_ATTR_RE);
  return styleAttr ? parseDeclarations(styleAttr[1]) : {};
}

function parseDeclarations(raw: string): SvgDeclarations {
  const declarations: SvgDeclarations = {};
  for (const part of raw.split(";")) {
    const separator = part.indexOf(":");
    if (separator < 1) continue;
    const name = part.slice(0, separator).trim();
    const value = part.slice(separator + 1).trim();
    if (SVG_STYLE_ATTRS.has(name) && value) {
      declarations[name] = value;
    }
  }
  return declarations;
}

function removeAttributes(attrs: string, names: string[]): string {
  return names.reduce((next, name) => {
    // eslint-disable-next-line security/detect-non-literal-regexp -- name is escaped, no injection vector
    const pattern = new RegExp(`\\s${escapeRegExp(name)}="[^"]*"`, "g");
    return next.replace(pattern, "");
  }, attrs);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function attrsToString(declarations: SvgDeclarations): string {
  return Object.entries(declarations)
    .map(([name, value]) => ` ${name}="${escapeAttribute(value)}"`)
    .join("");
}

function escapeAttribute(value: string): string {
  return value.replace(/&/g, "&amp;").replace(/"/g, "&quot;");
}

function scopeSvgIds(svg: string, prefix: string): string {
  let scoped = svg.replace(/\bid="([^"]+)"/g, `id="${prefix}$1"`);
  scoped = scoped.replace(/url\(#([^)]+)\)/g, `url(#${prefix}$1)`);
  scoped = scoped.replace(/xlink:href="#([^"]+)"/g, `xlink:href="#${prefix}$1"`);
  scoped = scoped.replace(/(^|[\s<])href="#([^"]+)"/g, `$1href="#${prefix}$2"`);
  scoped = scoped.replace(/class="([^"]+)"/g, (_match, classes: string) => {
    const next = classes.split(/\s+/).filter(Boolean).map((c) => `${prefix}${c}`).join(" ");
    return `class="${next}"`;
  });
  return scoped;
}
