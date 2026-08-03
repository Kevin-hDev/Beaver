import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

const shell = fs.readFileSync("install.sh", "utf8");
const powershell = fs.readFileSync("install.ps1", "utf8");
const powershellBytes = fs.readFileSync("install.ps1");

function lines(source) {
  return source.split(/\r?\n/).length - 1;
}

function assertBalanced(source) {
  const stack = [];
  const pairs = new Map([["}", "{"], [")", "("], ["]", "["]]);
  let quote = "";
  for (let index = 0; index < source.length; index += 1) {
    const character = source[index];
    if (quote) {
      if (quote === "'" && character === "'" && source[index + 1] === "'") {
        index += 1;
      } else if (quote === '"' && character === "`") {
        index += 1;
      } else if (character === quote) {
        quote = "";
      }
      continue;
    }
    if (character === "'" || character === '"') {
      quote = character;
    } else if (character === "#") {
      index = source.indexOf("\n", index);
      if (index === -1) break;
    } else if ("{([".includes(character)) {
      stack.push(character);
    } else if (pairs.has(character)) {
      assert.equal(stack.pop(), pairs.get(character), `délimiteur ${character}`);
    }
  }
  assert.equal(quote, "");
  assert.deepEqual(stack, []);
}

test("les deux installateurs restent petits et ciblent Beaver", () => {
  assert.ok(lines(shell) < 200);
  assert.ok(lines(powershell) < 200);
  for (const source of [shell, powershell]) {
    assert.match(source, /Kevin-hDev\/Beaver/);
    assert.match(source, /update-manifest\.json/);
    assert.match(source, /2147483648/);
    assert.match(source, /release-assets\.githubusercontent\.com/);
    const historicalRepository = `Kevin-hDev/${["CL", "GO", "DASH"].join("-")}`;
    assert.ok(!source.includes(historicalRepository));
    assert.doesNotMatch(source, /\b(?:eval|Invoke-Expression)\b/i);
  }
});

test("le script shell borne et vérifie chaque téléchargement", () => {
  const curlInvocation = shell.match(/code=\$\("\$CURL"[\s\S]*?"\$current"\)/)?.[0] ?? "";
  assert.match(shell, /MAX_API_BYTES=524288/);
  assert.match(shell, /MAX_MANIFEST_BYTES=65536/);
  assert.match(shell, /--max-filesize "\$limit"/);
  assert.match(shell, /\[ "\$redirects" -lt 3 \]/);
  assert.match(shell, /manifest_values "\$VERSION" "\$ASSET_NAME"/);
  assert.match(shell, /sha256_file "\$platform" "\$asset"/);
  assert.match(shell, /\[ "\$actual_hash" = "\$expected_hash" \]/);
  assert.match(shell, /apt-get install -y "\$asset"/);
  assert.match(shell, /Print :CFBundleExecutable/);
  assert.match(shell, /CL-GO\.app/);
  assert.match(shell, /package_installed beaver/);
  assert.match(shell, /stage_inode=.*stat -f/);
  assert.match(shell, /\$\{stage##\*\/\}/);
  assert.doesNotMatch(shell, /\.Beaver\.app\.backup-/);
  assert.notEqual(curlInvocation, "");
  assert.doesNotMatch(curlInvocation, /(?:--location|(?:^|\s)-[A-Za-z]*L[A-Za-z]*)/);
});

test("PowerShell désactive les redirections implicites et vérifie le SHA", () => {
  assertBalanced(powershell);
  assert.match(powershell, /AllowAutoRedirect = \$false/);
  assert.match(powershell, /MaxResponseHeadersLength = 64/);
  assert.match(powershell, /ResponseHeadersRead/);
  assert.match(powershell, /CancellationTokenSource/);
  assert.match(powershell, /Get-FileHash -LiteralPath \$assetPath -Algorithm SHA256/);
  assert.match(powershell, /Beaver_\$\{version\}_x64-setup\.exe/);
  assert.match(powershell, /-ArgumentList @\("\/S", "\/D=\$installDirectory"\)/);
  assert.doesNotMatch(powershell, /Invoke-(?:WebRequest|RestMethod)/);
});

test("le script PowerShell reste compatible avec Windows PowerShell 5.1", () => {
  assert.deepEqual([...powershellBytes.subarray(0, 3)], [0xef, 0xbb, 0xbf]);
});
