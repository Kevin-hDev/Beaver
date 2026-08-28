const ROUTING_IDENTITY = String.raw`(?:provider|provider_id|chat_provider_id|canonical_provider_id|connection_id|route_id)`;
const PROVIDER_VALUE = String.raw`(?:"[^"]+"|(?:[A-Za-z_][A-Za-z0-9_]*::)*[A-Z][A-Z0-9_]*)`;

export function scanRust(rawSource) {
  const source = stripTestModules(rawSource);
  const decisions = [];
  const comparison = new RegExp(
    String.raw`\b${ROUTING_IDENTITY}\b\s*(?:==|!=)\s*${PROVIDER_VALUE}|${PROVIDER_VALUE}\s*(?:==|!=)\s*\b${ROUTING_IDENTITY}\b`,
    "g",
  );
  const routeMatch = new RegExp(
    String.raw`\bmatch\s+(?:&\s*)?\b${ROUTING_IDENTITY}\b`,
    "g",
  );
  const matchesMacro = new RegExp(
    String.raw`\bmatches!\s*\(\s*(?:&\s*)?\b${ROUTING_IDENTITY}\b`,
    "g",
  );
  const namedCapability = /(?<!fn\s)\bis_[a-zA-Z0-9_]+\s*\(\s*&?\s*model(?:_id)?\b/g;
  for (const [kind, expression] of [
    ["comparison", comparison],
    ["match", routeMatch],
    ["matches", matchesMacro],
    ["named_predicate", namedCapability],
  ]) {
    for (const match of source.matchAll(expression)) {
      decisions.push({ kind, line: lineOf(source, match.index ?? 0) });
    }
  }
  return decisions.sort((left, right) => left.line - right.line);
}

function stripTestModules(source) {
  const testModule = /#\[cfg\([^\]]*\btest\b[^\]]*\)\]\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+[A-Za-z_][A-Za-z0-9_]*\s*\{/g;
  const ranges = [];
  for (const match of source.matchAll(testModule)) {
    const start = match.index ?? 0;
    const open = start + match[0].lastIndexOf("{");
    const close = matchingBrace(source, open);
    if (close >= 0) ranges.push([start, close + 1]);
  }
  let stripped = source;
  for (const [start, end] of ranges.reverse()) {
    const blank = stripped.slice(start, end).replace(/[^\n]/g, " ");
    stripped = `${stripped.slice(0, start)}${blank}${stripped.slice(end)}`;
  }
  return stripped;
}

function matchingBrace(source, open) {
  let depth = 0;
  let blockCommentDepth = 0;
  let quoted = false;
  let lineComment = false;
  for (let index = open; index < source.length; index += 1) {
    const current = source[index];
    const next = source[index + 1];
    if (lineComment) {
      if (current === "\n") lineComment = false;
      continue;
    }
    if (blockCommentDepth > 0) {
      if (current === "/" && next === "*") {
        blockCommentDepth += 1;
        index += 1;
      } else if (current === "*" && next === "/") {
        blockCommentDepth -= 1;
        index += 1;
      }
      continue;
    }
    if (quoted) {
      if (current === "\\") index += 1;
      else if (current === '"') quoted = false;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
    } else if (current === "/" && next === "*") {
      blockCommentDepth = 1;
      index += 1;
    } else if (current === '"') {
      quoted = true;
    } else if (current === "{") {
      depth += 1;
    } else if (current === "}" && --depth === 0) {
      return index;
    }
  }
  return -1;
}

function lineOf(source, index) {
  let line = 1;
  for (let cursor = 0; cursor < index; cursor += 1) {
    if (source.charCodeAt(cursor) === 10) line += 1;
  }
  return line;
}
