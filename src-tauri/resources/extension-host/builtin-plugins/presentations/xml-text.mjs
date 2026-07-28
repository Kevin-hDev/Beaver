import { rejectOffice } from "../common/errors.mjs";
import { containsInvalidXmlCharacter } from "../common/validation.mjs";

const PARAGRAPH = /<a:p(?:\s[^>]*)?>[\s\S]*?<\/a:p>/gu;
const XML_ENTITY = /&(?:amp|lt|gt|quot|apos|#[0-9]+|#x[0-9a-fA-F]+);/gu;

export function replaceParagraphTokens(xml, replacements) {
  let count = 0;
  const updated = xml.replace(PARAGRAPH, (paragraph) => {
    const result = replaceParagraph(paragraph, replacements);
    count += result.count;
    return result.xml;
  });
  return { xml: updated, count };
}

export function escapeXml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}

function replaceParagraph(paragraph, replacements) {
  const nodes = textNodes(paragraph);
  if (nodes.length === 0) return { xml: paragraph, count: 0 };
  const text = nodes.map((node) => decodeXml(node.content)).join("");
  const result = replaceTokens(text, replacements);
  if (result.count === 0) return { xml: paragraph, count: 0 };
  const chunks = distribute(result.text, nodes);
  let cursor = 0;
  let xml = "";
  for (let index = 0; index < nodes.length; index += 1) {
    const node = nodes[index];
    xml += paragraph.slice(cursor, node.contentStart);
    xml += escapeXml(chunks[index]);
    cursor = node.contentEnd;
  }
  xml += paragraph.slice(cursor);
  return { xml, count: result.count };
}

function textNodes(paragraph) {
  const nodes = [];
  let cursor = 0;
  while (cursor < paragraph.length) {
    const start = paragraph.indexOf("<a:t", cursor);
    if (start < 0) break;
    const contentStart = paragraph.indexOf(">", start + 4);
    const contentEnd = contentStart < 0
      ? -1
      : paragraph.indexOf("</a:t>", contentStart + 1);
    if (contentStart < 0 || contentEnd < 0) rejectOffice("unsafe_archive");
    nodes.push({
      contentStart: contentStart + 1,
      contentEnd,
      content: paragraph.slice(contentStart + 1, contentEnd),
    });
    cursor = contentEnd + 6;
  }
  return nodes;
}

function replaceTokens(text, replacements) {
  const values = new Map(replacements);
  const pattern = new RegExp(
    replacements.map(([token]) => escapePattern(token)).join("|"),
    "gu",
  );
  let count = 0;
  const updated = text.replace(pattern, (token) => {
    count += 1;
    return values.get(token);
  });
  return { text: updated, count };
}

function distribute(text, nodes) {
  const characters = Array.from(text);
  const chunks = [];
  let cursor = 0;
  for (let index = 0; index < nodes.length; index += 1) {
    const remaining = characters.length - cursor;
    const length = index === nodes.length - 1
      ? remaining
      : Math.min(Array.from(decodeXml(nodes[index].content)).length, remaining);
    chunks.push(characters.slice(cursor, cursor + length).join(""));
    cursor += length;
  }
  return chunks;
}

function decodeXml(value) {
  if (value.replace(XML_ENTITY, "").includes("&")) {
    rejectOffice("unsafe_archive");
  }
  const decoded = value.replace(XML_ENTITY, (entity) => {
    if (entity === "&amp;") return "&";
    if (entity === "&lt;") return "<";
    if (entity === "&gt;") return ">";
    if (entity === "&quot;") return '"';
    if (entity === "&apos;") return "'";
    const radix = entity.startsWith("&#x") ? 16 : 10;
    const source = entity.slice(radix === 16 ? 3 : 2, -1);
    const codePoint = Number.parseInt(source, radix);
    if (!Number.isSafeInteger(codePoint) || codePoint > 0x10ffff) {
      rejectOffice("unsafe_archive");
    }
    return String.fromCodePoint(codePoint);
  });
  if (containsInvalidXmlCharacter(decoded)) rejectOffice("unsafe_archive");
  return decoded;
}

function escapePattern(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
