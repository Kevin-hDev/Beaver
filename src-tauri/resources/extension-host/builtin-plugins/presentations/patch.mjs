import { strFromU8, strToU8, unzipSync, zipSync } from "fflate";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import { plainObject, requiredString } from "../common/validation.mjs";
import {
  atomicWrite,
  readWorkspaceFile,
  workspaceOutput,
} from "../common/workspace.mjs";
import { assertSafeOfficeArchive } from "../common/zip-guard.mjs";

const PLACEHOLDER = /^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,99}$/u;
const SLIDE_XML = /^ppt\/slides\/slide[1-9][0-9]*\.xml$/u;

export async function patchPresentation(arguments_, context) {
  const sourcePath = requiredString(arguments_?.sourcePath, OFFICE_LIMITS.maxPathChars);
  const outputPath = requiredString(arguments_?.outputPath, OFFICE_LIMITS.maxPathChars);
  const replacements = validateReplacements(arguments_?.replacements);
  const input = await readWorkspaceFile(
    context,
    sourcePath,
    OFFICE_EXTENSIONS.presentation,
  );
  let files;
  try {
    assertSafeOfficeArchive(input.bytes);
    files = unzipSync(input.bytes);
    let replacementCount = 0;
    for (const [name, bytes] of Object.entries(files)) {
      if (!SLIDE_XML.test(name)) continue;
      const result = replaceTextNodes(strFromU8(bytes), replacements);
      replacementCount += result.count;
      if (result.count > 0) files[name] = strToU8(result.xml);
    }
    if (replacementCount === 0) rejectOffice("invalid_input");
    const output = await workspaceOutput(
      context,
      outputPath,
      OFFICE_EXTENSIONS.presentation,
    );
    const bytes = zipSync(files, { level: 6 });
    await atomicWrite(output.path, bytes);
    return success({
      path: outputPath,
      format: "pptx",
      replacements: replacementCount,
    });
  } finally {
    input.bytes.fill(0);
    for (const value of Object.values(files ?? {})) value.fill(0);
  }
}

function validateReplacements(raw) {
  const object = plainObject(raw);
  const entries = Object.entries(object);
  if (entries.length === 0 || entries.length > 256) rejectOffice("invalid_input");
  const values = [];
  let totalText = 0;
  for (const [name, rawValue] of entries) {
    if (!PLACEHOLDER.test(name)) rejectOffice("invalid_input");
    const value = requiredString(rawValue, 32_767);
    totalText += value.length;
    values.push([`{{${name}}}`, escapeXml(value)]);
  }
  if (totalText > OFFICE_LIMITS.maxTextChars) rejectOffice("invalid_input");
  return values;
}

function replaceTextNodes(xml, replacements) {
  const chunks = [];
  let cursor = 0;
  let count = 0;
  while (cursor < xml.length) {
    const start = xml.indexOf("<a:t", cursor);
    if (start < 0) break;
    const contentStart = xml.indexOf(">", start + 4);
    const end = contentStart < 0 ? -1 : xml.indexOf("</a:t>", contentStart + 1);
    if (end < 0) rejectOffice("unsafe_archive");
    chunks.push(xml.slice(cursor, contentStart + 1));
    let content = xml.slice(contentStart + 1, end);
    for (const [token, value] of replacements) {
      const before = content;
      content = content.split(token).join(value);
      if (content !== before) count += before.split(token).length - 1;
    }
    chunks.push(content, "</a:t>");
    cursor = end + 6;
  }
  chunks.push(xml.slice(cursor));
  return { xml: chunks.join(""), count };
}

function escapeXml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;")
    .replaceAll("'", "&apos;");
}
