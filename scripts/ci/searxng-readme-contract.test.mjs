import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

// Git may materialize CRLF on Windows; contracts are about Markdown structure, not checkout EOLs.
const readme = (await readFile(new URL("../../README.md", import.meta.url), "utf8"))
  .replaceAll("\r\n", "\n");

test("les trois OS publient explicitement Python 3.14 dans le PATH utilisateur", () => {
  assert.equal((readme.match(/UV_PYTHON_BIN_DIR/g) ?? []).length, 2);
  assert.equal((readme.match(/UV_PYTHON_INSTALL_BIN/g) ?? []).length, 3);
  assert.equal((readme.match(/uv python install 3\.14/g) ?? []).length, 3);
  const windows = readme.slice(readme.indexOf("### Windows"), readme.indexOf("### Development only"));
  const installBlock = /Then install CPython[^:]+:\n\n```powershell\n([\s\S]+?)\n```/u.exec(windows)?.[1];
  assert.ok(installBlock);
  assert.equal((installBlock.match(/uv python update-shell/g) ?? []).length, 1);
});

test("le refus Linux reste dans un sous-shell sans fermer le terminal", () => {
  const linux = readme.slice(readme.indexOf("### Linux (x64)"), readme.indexOf("### Windows"));
  const block = /```bash\n([\s\S]+?)\n```/u.exec(linux)?.[1];
  assert.ok(block);
  assert.ok(block.startsWith("(\nset -e\n"));
  assert.ok(block.trimEnd().endsWith(")"));
  assert.equal((block.match(/exit 1/g) ?? []).length, 3);
  const lines = block.split("\n");
  assert.equal(lines.filter((line) => line === "(").length, 1);
  assert.equal(lines.filter((line) => line === ")").length, 1);
  assert.equal(lines[0], "(");
  assert.equal(lines.at(-1), ")");
});
