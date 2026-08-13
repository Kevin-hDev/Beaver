import { chmod, mkdir, writeFile } from "node:fs/promises";
import { dirname, join } from "node:path";

export async function writeExecutable(path, body) {
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, body, "utf8");
  await chmod(path, 0o755);
}

export async function createFakeCommands(fakeBin) {
  const cargoLog = join(fakeBin, "cargo-invocations.log");
  await writeFile(cargoLog, "", "utf8");
  await writeExecutable(join(fakeBin, "uname"), "#!/bin/bash\nprintf 'Darwin\\n'\n");
  await writeExecutable(join(fakeBin, "node"), "#!/bin/bash\nexit 0\n");
  await writeExecutable(join(fakeBin, "codesign"), "#!/bin/bash\nexit 0\n");
  await writeExecutable(
    join(fakeBin, "ditto"),
    `#!/bin/bash
set -euo pipefail
if [[ -d "$2" ]]; then
  /bin/cp -R "$1"/. "$2"/
else
  /bin/cp -R "$1" "$2"
fi
`,
  );
  await writeExecutable(
    join(fakeBin, "cargo"),
    `#!/bin/bash
set -euo pipefail
: "\${FIXTURE_CARGO_LOG:?}"
/usr/bin/printf 'cargo\\n' >> "$FIXTURE_CARGO_LOG"
profile="dev"
for argument in "$@"; do
  [[ "$argument" == "e2e" ]] && profile="e2e"
done
target_root="\${CARGO_TARGET_DIR:-target}"
release_root="$target_root/release"
/bin/mkdir -p "$release_root"
/usr/bin/printf '%s-helper\\n' "$profile" > "$release_root/cl-go-dash-helper"
/bin/chmod 755 "$release_root/cl-go-dash-helper"
`,
  );
  return cargoLog;
}
