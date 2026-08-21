import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const readme = await readFile(new URL("../../README.md", import.meta.url), "utf8");

test("les trois OS publient explicitement Python 3.14 dans le PATH utilisateur", () => {
  assert.equal((readme.match(/UV_PYTHON_BIN_DIR/g) ?? []).length, 3);
  assert.equal((readme.match(/UV_PYTHON_INSTALL_BIN/g) ?? []).length, 3);
  assert.equal((readme.match(/uv python install 3\.14/g) ?? []).length, 3);
});

test("le refus Linux reste dans un sous-shell sans fermer le terminal", () => {
  const linux = readme.slice(readme.indexOf("### Linux (x64)"), readme.indexOf("### Windows"));
  assert.match(linux, /\(\nset -e\n/);
  assert.match(linux, /curl -LsSf[\s\S]+\n\)/);
});
