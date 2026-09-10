const MAX_SVG_LENGTH = 256 * 1024;
const MAX_PREFIX_LENGTH = 128;
const MAX_CLASS_RULES = 256;
const SVG_NAMESPACE = "http://www.w3.org/2000/svg";
const XLINK_NAMESPACE = "http://www.w3.org/1999/xlink";

const CLASS_RULE_RE = /\.([_a-zA-Z][\w-]*)\s*\{([^}]*)\}/g;
const SAFE_PREFIX_RE = /^[a-zA-Z0-9_-]+$/;
const LOCAL_REFERENCE_RE = /^#([_a-zA-Z][\w:.-]*)$/;
const LOCAL_URL_RE = /url\(\s*#([_a-zA-Z][\w:.-]*)\s*\)/gi;
const ANY_URL_RE = /url\s*\(/i;
const EXTERNAL_PROTOCOL_RE = /(?:data|file|https?|javascript):/i;

const SAFE_ELEMENTS = new Set([
  "circle",
  "clippath",
  "defs",
  "desc",
  "ellipse",
  "g",
  "line",
  "lineargradient",
  "path",
  "polygon",
  "polyline",
  "radialgradient",
  "rect",
  "stop",
  "symbol",
  "title",
  "use",
]);

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
  if (!isValidInput(raw, prefix)) return "";

  // XML parsing is the authority for element boundaries; text replacement can
  // accidentally rebuild a forbidden tag from two individually harmless parts.
  const document = new DOMParser().parseFromString(raw, "image/svg+xml");
  const root = document.documentElement;
  if (
    document.querySelector("parsererror")
    || root.localName.toLowerCase() !== "svg"
  ) {
    return "";
  }

  removeUnsafeElements(root);
  const classStyles = collectClassStyles(root);
  normalizeElements(root, classStyles, prefix);
  return new XMLSerializer().serializeToString(root);
}

function isValidInput(raw: string, prefix: string): boolean {
  return raw.length > 0
    && raw.length <= MAX_SVG_LENGTH
    && prefix.length > 0
    && prefix.length <= MAX_PREFIX_LENGTH
    && SAFE_PREFIX_RE.test(prefix)
    && !/<\s*!doctype\b/i.test(raw);
}

function collectClassStyles(root: Element): Map<string, SvgDeclarations> {
  const styles = new Map<string, SvgDeclarations>();
  for (const style of root.querySelectorAll("style")) {
    for (const rule of (style.textContent ?? "").matchAll(CLASS_RULE_RE)) {
      if (styles.size >= MAX_CLASS_RULES && !styles.has(rule[1])) break;
      const declarations = parseDeclarations(rule[2]);
      if (Object.keys(declarations).length > 0) styles.set(rule[1], declarations);
    }
    style.remove();
  }
  return styles;
}

function removeUnsafeElements(root: Element): void {
  for (const element of root.querySelectorAll("*")) {
    const name = element.localName.toLowerCase();
    if (name !== "style" && !SAFE_ELEMENTS.has(name)) element.remove();
  }
}

function normalizeElements(
  root: Element,
  classStyles: Map<string, SvgDeclarations>,
  prefix: string,
): void {
  for (const element of [root, ...root.querySelectorAll("*")]) {
    inlineStyles(element, classStyles);
    sanitizeAttributes(element);
    scopeAttributes(element, prefix);
  }
}

function inlineStyles(element: Element, styles: Map<string, SvgDeclarations>): void {
  const declarations: SvgDeclarations = {};
  for (const className of element.getAttribute("class")?.split(/\s+/).filter(Boolean) ?? []) {
    Object.assign(declarations, styles.get(className));
  }
  Object.assign(declarations, parseDeclarations(element.getAttribute("style") ?? ""));
  element.removeAttribute("style");
  for (const [name, value] of Object.entries(declarations)) {
    element.setAttribute(name, value);
  }
}

function parseDeclarations(raw: string): SvgDeclarations {
  const declarations: SvgDeclarations = {};
  for (const part of raw.split(";")) {
    const separator = part.indexOf(":");
    if (separator < 1) continue;
    const name = part.slice(0, separator).trim().toLowerCase();
    const value = part.slice(separator + 1).trim();
    if (SVG_STYLE_ATTRS.has(name) && value && isSafeValue(value)) {
      declarations[name] = value;
    }
  }
  return declarations;
}

function sanitizeAttributes(element: Element): void {
  for (const attribute of [...element.attributes]) {
    const name = attribute.name.toLowerCase();
    if (
      name.startsWith("on")
      || name === "style"
      || (name.includes(":") && name !== "xlink:href" && name !== "xmlns:xlink")
      || !isSafeAttribute(name, attribute.value)
    ) {
      element.removeAttribute(attribute.name);
    }
  }
}

function isSafeAttribute(name: string, value: string): boolean {
  if (name === "href" || name === "xlink:href") return LOCAL_REFERENCE_RE.test(value);
  if (name === "xmlns") return value === SVG_NAMESPACE;
  if (name === "xmlns:xlink") return value === XLINK_NAMESPACE;
  return isSafeValue(value);
}

function isSafeValue(value: string): boolean {
  if (EXTERNAL_PROTOCOL_RE.test(value)) return false;
  const withoutLocalUrls = value.replace(LOCAL_URL_RE, "");
  return !ANY_URL_RE.test(withoutLocalUrls);
}

function scopeAttributes(element: Element, prefix: string): void {
  const id = element.getAttribute("id");
  if (id) element.setAttribute("id", `${prefix}${id}`);

  for (const attribute of [...element.attributes]) {
    let value = attribute.value.replace(LOCAL_URL_RE, `url(#${prefix}$1)`);
    const reference = value.match(LOCAL_REFERENCE_RE);
    if ((attribute.name === "href" || attribute.name === "xlink:href") && reference) {
      value = `#${prefix}${reference[1]}`;
    }
    element.setAttribute(attribute.name, value);
  }

  const classes = element.getAttribute("class")?.split(/\s+/).filter(Boolean);
  if (classes?.length) {
    element.setAttribute("class", classes.map((name) => `${prefix}${name}`).join(" "));
  }
}
