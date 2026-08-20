const JOB_HEADING = /^[A-Za-z_][A-Za-z0-9_-]*:\s*(?:#.*)?$/u;

export function jobSection(workflow, jobName) {
  const lines = workflow.split(/\r?\n/u);
  const start = lines.findIndex(
    (line) => line === `${jobName}:` || line === `  ${jobName}:`,
  );
  if (start < 0) return "";

  const indentation = lines[start].startsWith("  ") ? "  " : "";
  const remaining = lines.slice(start + 1);
  const end = remaining.findIndex((line) => {
    if (!line.startsWith(indentation)) return false;
    return JOB_HEADING.test(line.slice(indentation.length));
  });
  return (end < 0 ? remaining : remaining.slice(0, end)).join("\n");
}
