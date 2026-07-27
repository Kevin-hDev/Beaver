import { Workbook } from "@xlsx/xlsx-populate";
import { OFFICE_EXTENSIONS, OFFICE_LIMITS } from "../common/constants.mjs";
import { rejectOffice, success } from "../common/errors.mjs";
import {
  boundedArray,
  plainObject,
  requiredString,
  scalar,
} from "../common/validation.mjs";
import {
  atomicWrite,
  readWorkspaceFile,
  workspaceOutput,
} from "../common/workspace.mjs";
import { assertSafeOfficeArchive } from "../common/zip-guard.mjs";

const CELL_ADDRESS = /^([a-zA-Z]{1,3})([1-9][0-9]{0,6})$/u;
const INVALID_SHEET = /[\\/*[\]:?]/u;

export async function createSpreadsheet(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const definitions = boundedArray(arguments_?.sheets, OFFICE_LIMITS.maxSheets);
  const sheets = definitions.map(validateSheetDefinition);
  const names = new Set(sheets.map((sheet) => sheet.name.toLocaleLowerCase("en")));
  if (names.size !== sheets.length) rejectOffice("invalid_input");
  const workbook = await Workbook.fromBlank();
  for (let index = 0; index < sheets.length; index += 1) {
    const definition = sheets[index];
    const sheet = index === 0 ? workbook.sheet(0) : workbook.addSheet(definition.name);
    if (!sheet) rejectOffice("operation_failed");
    if (index === 0) sheet.name(definition.name);
    if (definition.rows.length > 0) sheet.cell(1, 1).value(definition.rows);
  }
  const output = await workspaceOutput(context, path, OFFICE_EXTENSIONS.spreadsheet);
  const bytes = await workbook.output("node:buffer");
  await atomicWrite(output.path, bytes);
  return success({ path, format: "xlsx", sheets: sheets.length });
}

export async function inspectSpreadsheet(arguments_, context) {
  const path = requiredString(arguments_?.path, OFFICE_LIMITS.maxPathChars);
  const maxRows = boundedInteger(arguments_?.maxRows ?? 50, 1, 200);
  const maxColumns = boundedInteger(arguments_?.maxColumns ?? 30, 1, 100);
  const input = await readWorkspaceFile(context, path, OFFICE_EXTENSIONS.spreadsheet);
  try {
    const workbook = await loadWorkbook(input.bytes);
    const sheets = workbook.sheets();
    if (sheets.length > OFFICE_LIMITS.maxSheets) rejectOffice("file_too_large");
    const summaries = sheets.map((sheet) => {
      const used = sheet.usedRange();
      if (!used) return { name: sheet.name(), rows: 0, columns: 0, preview: [] };
      const rows = used.endCell().rowNumber();
      const columns = used.endCell().columnNumber();
      const previewRows = Math.min(rows, maxRows);
      const previewColumns = Math.min(columns, maxColumns);
      const preview = sheet.range(1, 1, previewRows, previewColumns).value();
      return {
        name: sheet.name(),
        rows,
        columns,
        truncated: rows > previewRows || columns > previewColumns,
        preview,
      };
    });
    return success({ path, format: "xlsx", sheets: summaries });
  } finally {
    input.bytes.fill(0);
  }
}

export async function updateSpreadsheet(arguments_, context) {
  const sourcePath = requiredString(arguments_?.sourcePath, OFFICE_LIMITS.maxPathChars);
  const outputPath = requiredString(arguments_?.outputPath, OFFICE_LIMITS.maxPathChars);
  const changes = boundedArray(arguments_?.changes, OFFICE_LIMITS.maxChanges)
    .map(validateChange);
  const input = await readWorkspaceFile(context, sourcePath, OFFICE_EXTENSIONS.spreadsheet);
  try {
    const workbook = await loadWorkbook(input.bytes);
    if (workbook.sheets().length > OFFICE_LIMITS.maxSheets) rejectOffice("file_too_large");
    for (const change of changes) {
      const sheet = workbook.sheet(change.sheet);
      if (!sheet) rejectOffice("invalid_input");
      sheet.cell(change.cell).value(change.value);
    }
    const output = await workspaceOutput(
      context,
      outputPath,
      OFFICE_EXTENSIONS.spreadsheet,
    );
    const bytes = await workbook.output("node:buffer");
    await atomicWrite(output.path, bytes);
  } finally {
    input.bytes.fill(0);
  }
  return success({ path: outputPath, format: "xlsx", changes: changes.length });
}

async function loadWorkbook(bytes) {
  assertSafeOfficeArchive(bytes);
  return Workbook.fromData(bytes);
}

function validateSheetDefinition(raw) {
  const definition = plainObject(raw);
  const name = validSheetName(definition.name);
  const rows = boundedArray(
    definition.rows,
    OFFICE_LIMITS.maxRowsPerSheet,
    true,
  );
  let cells = 0;
  const values = rows.map((rawRow) => {
    const row = boundedArray(rawRow, OFFICE_LIMITS.maxColumns, true).map(scalar);
    cells += row.length;
    if (cells > OFFICE_LIMITS.maxCells) rejectOffice("invalid_input");
    return row;
  });
  return { name, rows: values };
}

function validateChange(raw) {
  const change = plainObject(raw);
  const sheet = validSheetName(change.sheet);
  const cell = requiredString(change.cell, 10).toUpperCase();
  const match = CELL_ADDRESS.exec(cell);
  if (!match || columnNumber(match[1]) > 16_384 || Number(match[2]) > 1_048_576) {
    rejectOffice("invalid_input");
  }
  return { sheet, cell, value: scalar(change.value) };
}

function validSheetName(value) {
  const name = requiredString(value, 31);
  if (
    INVALID_SHEET.test(name)
    || name.toLocaleLowerCase("en") === "history"
  ) {
    rejectOffice("invalid_input");
  }
  return name;
}

function columnNumber(letters) {
  let value = 0;
  for (const letter of letters.toUpperCase()) {
    value = value * 26 + letter.charCodeAt(0) - 64;
  }
  return value;
}

function boundedInteger(value, minimum, maximum) {
  if (!Number.isInteger(value) || value < minimum || value > maximum) {
    rejectOffice("invalid_input");
  }
  return value;
}
