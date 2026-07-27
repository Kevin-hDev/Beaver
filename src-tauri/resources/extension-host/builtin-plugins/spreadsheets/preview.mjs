export function spreadsheetPreview(sheet, rows, columns, budget) {
  const preview = [];
  let budgetTruncated = false;
  for (let rowNumber = 1; rowNumber <= rows && !budgetTruncated; rowNumber += 1) {
    const row = [];
    for (let columnNumber = 1; columnNumber <= columns; columnNumber += 1) {
      const value = previewValue(sheet.cell(rowNumber, columnNumber).value());
      if (!budget.take(value, 2)) {
        budgetTruncated = true;
        break;
      }
      row.push(value);
    }
    if (row.length > 0) preview.push(row);
  }
  return { preview, budgetTruncated };
}

function previewValue(value) {
  if (
    value === null
    || typeof value === "string"
    || typeof value === "boolean"
    || (typeof value === "number" && Number.isFinite(value))
  ) {
    return value;
  }
  if (value instanceof Date && Number.isFinite(value.getTime())) {
    return value.toISOString();
  }
  return null;
}
