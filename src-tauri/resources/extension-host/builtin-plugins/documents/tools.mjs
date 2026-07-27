import {
  Document,
  HeadingLevel,
  Packer,
  Paragraph,
  PatchType,
  TextRun,
  patchDocument,
} from "docx";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  optionalString,
  plainObject,
  requiredString,
} from "../common/validation.mjs";
import {
  atomicWrite,
  readWorkspaceFile,
  workspaceOutput,
} from "../common/workspace.mjs";
import { assertSafeOfficeArchive } from "../common/zip-guard.mjs";

const PLACEHOLDER = /^[a-zA-Z0-9][a-zA-Z0-9_.-]{0,99}$/u;

export async function createDocument(arguments_, context) {
  const blocks = boundedArray(arguments_?.blocks, OFFICE_LIMITS.maxBlocks);
  const title = optionalString(arguments_?.title, 300);
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const children = blocks.map(toParagraph);
  const totalText = blocks.reduce((sum, block) => sum + block.text.length, 0);
  if (totalText > OFFICE_LIMITS.maxTextChars) rejectOffice("invalid_input");
  const document = new Document({
    creator: "Beaver",
    title,
    sections: [{ children }],
  });
  const output = await workspaceOutput(context, path, OFFICE_EXTENSIONS.document);
  const bytes = await Packer.toBuffer(document);
  await atomicWrite(output.path, bytes);
  return success({ path, format: "docx", blocks: blocks.length });
}

export async function patchDocumentTemplate(arguments_, context) {
  const sourcePath = requiredString(
    arguments_?.sourcePath,
    OFFICE_LIMITS.maxPathChars,
  );
  const outputPath = requiredString(
    arguments_?.outputPath,
    OFFICE_LIMITS.maxPathChars,
  );
  const replacements = plainObject(arguments_?.replacements);
  const entries = Object.entries(replacements);
  if (entries.length === 0 || entries.length > 256) rejectOffice("invalid_input");
  const patches = Object.create(null);
  let totalText = 0;
  for (const [name, rawValue] of entries) {
    if (!PLACEHOLDER.test(name)) rejectOffice("invalid_input");
    const value = requiredString(rawValue, 32_767);
    totalText += value.length;
    patches[name] = {
      type: PatchType.PARAGRAPH,
      children: [new TextRun(value)],
    };
  }
  if (totalText > OFFICE_LIMITS.maxTextChars) rejectOffice("invalid_input");
  const input = await readWorkspaceFile(
    context,
    sourcePath,
    OFFICE_EXTENSIONS.document,
  );
  const output = await workspaceOutput(
    context,
    outputPath,
    OFFICE_EXTENSIONS.document,
  );
  try {
    assertSafeOfficeArchive(input.bytes);
    const bytes = await patchDocument({
      data: input.bytes,
      outputType: "nodebuffer",
      patches,
    });
    await atomicWrite(output.path, bytes);
  } finally {
    input.bytes.fill(0);
  }
  return success({ path: outputPath, format: "docx", replacements: entries.length });
}

function toParagraph(rawBlock) {
  const block = plainObject(rawBlock);
  const type = requiredString(block.type, 20);
  const text = requiredString(block.text, 32_767);
  if (type === "paragraph") return new Paragraph({ text });
  if (type === "bullet") {
    return new Paragraph({
      text,
      bullet: { level: normalizeLevel(block.level, 1, 6) - 1 },
    });
  }
  if (type === "heading") {
    const level = normalizeLevel(block.level, 1, 6);
    return new Paragraph({ text, heading: HeadingLevel[`HEADING_${level}`] });
  }
  rejectOffice("invalid_input");
}

function normalizeLevel(value, minimum, maximum) {
  const level = value ?? Math.max(1, minimum);
  if (!Number.isInteger(level) || level < minimum || level > maximum) {
    rejectOffice("invalid_input");
  }
  return level;
}
