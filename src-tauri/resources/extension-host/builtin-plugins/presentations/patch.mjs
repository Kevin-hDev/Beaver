import {
  strFromU8,
  strToU8,
  unzipSync,
  zipSync,
} from "../common/formats/archive.mjs";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import { plainObject, requiredString } from "../common/validation.mjs";
import {
  atomicWrite,
  readWorkspaceFile,
  workspaceOutput,
} from "../common/workspace.mjs";
import { assertSafeOfficeArchive } from "../common/zip-guard.mjs";
import { replaceParagraphTokens } from "./xml-text.mjs";

const PLACEHOLDER = /^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,99}$/u;
const SLIDE_XML = /^ppt\/slides\/slide[1-9][0-9]*\.xml$/u;

export async function patchPresentation(arguments_, context) {
  const sourcePath = requiredString(arguments_?.sourcePath, OFFICE_LIMITS.maxPathChars);
  const outputPath = requiredString(arguments_?.outputPath, OFFICE_LIMITS.maxPathChars);
  const replacements = validateReplacements(arguments_?.replacements);
  const output = await workspaceOutput(
    context,
    outputPath,
    OFFICE_EXTENSIONS.presentation,
  );
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
      const result = replaceParagraphTokens(strFromU8(bytes), replacements);
      replacementCount += result.count;
      if (result.count > 0) files[name] = strToU8(result.xml);
    }
    if (replacementCount === 0) rejectOffice("invalid_input");
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
    values.push([`{{${name}}}`, value]);
  }
  if (totalText > OFFICE_LIMITS.maxTextChars) rejectOffice("invalid_input");
  return values;
}
