use super::{add_parameters, policy, sandbox_roots, SANDBOX_EXEC};
use super::super::scope::{Mode, Scope};
use std::process::{Command, Output};

#[test]
fn xcrun_cache_rule_is_scoped_to_the_current_user_temp() {
    let rule = super::xcrun_cache_rule().expect("xcrun cache rule");
    let temp = super::darwin_user_temp_dir().expect("Darwin temp");

    assert!(rule.contains(temp.to_string_lossy().trim_end_matches('/')));
    assert!(rule.contains("/xcrun_db[^/]*$"));
    assert!(!rule.contains("folders/[^/]+"));
}

#[test]
fn seatbelt_allows_work_and_temp_inside_the_root_and_blocks_outside_data() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let outside = temp.path().join("outside.txt");
    let darwin_temp_probe = super::darwin_user_temp_dir().expect("Darwin temp").join(format!(
        "beaver-sandbox-probe-{}",
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::create_dir_all(&allowed).expect("allowed");
    std::fs::create_dir_all(&sandbox_temp).expect("sandbox temp");
    std::fs::write(&outside, "outside-data").expect("outside");
    std::fs::create_dir_all(crate::services::paths::data_dir()).expect("private store");
    let mut private_probe =
        tempfile::NamedTempFile::new_in(crate::services::paths::data_dir()).expect("private probe");
    std::io::Write::write_all(&mut private_probe, b"private-data").expect("private data");
    let private_probe_path =
        dunce::canonicalize(private_probe.path()).expect("canonical private probe");
    let allowed = dunce::canonicalize(allowed).expect("canonical allowed");
    let sandbox_temp = dunce::canonicalize(sandbox_temp).expect("canonical temp");
    let outside = dunce::canonicalize(outside).expect("canonical outside");
    let script = format!(
        r#"test "$TMPDIR" = '{}'
test "$TMPPREFIX" = '{}/zsh'
test -d "$TMPDIR"
mktemp >/dev/null
cat > '{}/heredoc.txt' <<'BEAVER_EOF'
line-one
line-two
BEAVER_EOF
diff <(printf same) <(printf same)
printf 'int main(void) {{ return 0; }}\n' > '{}/native.c'
/usr/bin/cc '{}/native.c' -o '{}/native'
'{}/native'
if printf pwn > '{}'; then /bin/rm -f '{}'; exit 89; fi
printf ok > '{}/inside.txt'
/bin/cat '{}' >/dev/null 2>&1 && exit 91
test "$(/bin/cat '{}')" = private-data
if printf changed > '{}'; then exit 92; fi
/bin/sh -c 'printf child'"#,
        sandbox_temp.display(),
        sandbox_temp.display(),
        allowed.display(),
        allowed.display(),
        allowed.display(),
        allowed.display(),
        allowed.display(),
        darwin_temp_probe.display(),
        darwin_temp_probe.display(),
        allowed.display(),
        outside.display(),
        private_probe_path.display(),
        private_probe_path.display(),
    );
    let output = run_sandboxed(&allowed, &sandbox_temp, &script);

    assert_success(&output);
    assert_eq!(std::fs::read_to_string(allowed.join("inside.txt")).unwrap(), "ok");
    assert_eq!(
        std::fs::read_to_string(allowed.join("heredoc.txt")).unwrap(),
        "line-one\nline-two\n"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "child");
    assert_eq!(std::fs::read_to_string(private_probe.path()).unwrap(), "private-data");
    assert!(!darwin_temp_probe.exists());
}

#[test]
#[ignore = "requires network access and installed development toolchains"]
fn seatbelt_supports_complete_local_development() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let outside = temp.path().join("outside.txt");
    let darwin_temp_probe = super::darwin_user_temp_dir().expect("Darwin temp").join(format!(
        "beaver-sandbox-probe-{}",
        uuid::Uuid::new_v4().simple()
    ));
    std::fs::create_dir_all(&allowed).expect("allowed");
    std::fs::create_dir_all(&sandbox_temp).expect("sandbox temp");
    let allowed = dunce::canonicalize(allowed).expect("canonical allowed");
    let sandbox_temp = dunce::canonicalize(sandbox_temp).expect("canonical temp");
    let script = format!(
        r#"set -eu
mkdir git-project
cd git-project
git init -q
printf tracked > tracked.txt
git add tracked.txt
git -c user.name=Beaver -c user.email=beaver@example.invalid -c commit.gpgsign=false commit -qm sandbox
test "$(git log -1 --format=%s)" = sandbox
cd ..
mkdir npm-project
cd npm-project
npm init -y >/dev/null
npm install --no-audit --no-fund is-odd >/dev/null
node -e "if (!require('is-odd')(3)) process.exit(1)"
cd ..
cargo new -q --bin rust-project
cd rust-project
printf 'itoa = "1"\n' >> Cargo.toml
printf 'fn main() {{ let mut value = itoa::Buffer::new(); println!("cargo-{{}}", value.format(42)); }}\n' > src/main.rs
test "$(cargo run -q)" = cargo-42
cd ..
mkdir go-project
cd go-project
go mod init example.invalid/sandbox >/dev/null
go get rsc.io/quote@v1.5.2 >/dev/null
cat > main.go <<'BEAVER_GO_EOF'
package main
import (
    "fmt"
    "rsc.io/quote"
)
func main() {{ fmt.Println(quote.Go()) }}
BEAVER_GO_EOF
test "$(go run .)" = "Don't communicate by sharing memory, share memory by communicating."
cd ..
diff <(printf same) <(printf same)
mkdir native-project
cd native-project
printf '#include <stdio.h>\nint main(void) {{ puts("native-ok"); return 0; }}\n' > native.c
/usr/bin/cc native.c -o native
test "$(./native)" = native-ok
cd ..
if printf pwn > '{}'; then /bin/rm -f '{}'; exit 89; fi
if printf pwn > '{}'; then exit 90; fi
printf verified
"#,
        darwin_temp_probe.display(),
        darwin_temp_probe.display(),
        outside.display(),
    );
    let output = run_sandboxed(&allowed, &sandbox_temp, &script);

    assert_success(&output);
    assert!(!outside.exists());
    assert!(!darwin_temp_probe.exists());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "verified");
    println!("git=ok\nnpm=ok\ncargo=ok\ngo=ok\nheredoc=ok\ndiff=ok\nnative=ok\ndarwin-temp=blocked\noutside-write=blocked");
}

#[test]
fn profile_capture_can_only_write_to_its_private_temp() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let shared_temp_probe = std::path::Path::new("/private/tmp").join(format!(
        "beaver-profile-probe-{}",
        uuid::Uuid::new_v4().simple()
    ));
    let xcrun_probe = super::darwin_user_temp_dir()
        .expect("Darwin temp")
        .join(format!("xcrun_db-beaver-{}", uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&allowed).expect("allowed");
    std::fs::create_dir_all(&sandbox_temp).expect("sandbox temp");
    std::fs::write(allowed.join("profile.txt"), "readable").expect("profile");
    let allowed = dunce::canonicalize(allowed).expect("canonical allowed");
    let sandbox_temp = dunce::canonicalize(sandbox_temp).expect("canonical temp");
    let script = format!(
        r#"test "$(/bin/cat '{}/profile.txt')" = readable
if printf pwn > '{}/blocked'; then exit 80; fi
if printf pwn > '{}'; then /bin/rm -f '{}'; exit 81; fi
if printf pwn > '{}'; then /bin/rm -f '{}'; exit 82; fi
printf ok > '{}/inside'"#,
        allowed.display(),
        allowed.display(),
        shared_temp_probe.display(),
        shared_temp_probe.display(),
        xcrun_probe.display(),
        xcrun_probe.display(),
        sandbox_temp.display(),
    );
    let scope = Scope {
        mode: Mode::ProfileCapture,
        roots: vec![allowed.clone()],
        read_dirs: Vec::new(),
        read_files: Vec::new(),
    };
    let output = run_sandboxed_scope(&scope, &sandbox_temp, &script);

    assert_success(&output);
    assert_eq!(std::fs::read_to_string(sandbox_temp.join("inside")).unwrap(), "ok");
    assert!(!allowed.join("blocked").exists());
    assert!(!shared_temp_probe.exists());
    assert!(!xcrun_probe.exists());
}

#[test]
fn interactive_profile_capture_keeps_path_but_cannot_escape() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let profile_home = temp.path().join("profile-home");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let nvm = profile_home.join(".nvm");
    let nvm_bin = nvm.join("versions/node/current/bin");
    let local_bin = profile_home.join(".local/bin");
    for path in [&allowed, &profile_home, &sandbox_temp, &nvm_bin, &local_bin] {
        std::fs::create_dir_all(path).expect("directory");
    }
    std::fs::write(
        nvm.join("nvm.sh"),
        "export PATH=\"$HOME/.nvm/versions/node/current/bin:$PATH\"\n",
    )
    .expect("nvm startup");
    let local_tool = local_bin.join("beaver-profile-tool");
    std::fs::write(&local_tool, "#!/bin/sh\nexit 0\n").expect("local tool");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&local_tool, std::fs::Permissions::from_mode(0o700))
            .expect("tool permissions");
    }
    let escape = allowed.join("blocked");
    let profile = profile_home.join(".zshrc");
    std::fs::write(
        &profile,
        "source \"$HOME/.nvm/nvm.sh\"\n\"$HOME/.local/bin/beaver-profile-tool\"\nexport PATH=\"$HOME/.local/bin:$PATH\"\nprintf pwn > \"$BEAVER_ESCAPE\" 2>/dev/null || true\n",
    )
    .expect("profile");
    let allowed = dunce::canonicalize(allowed).expect("allowed");
    let nvm = dunce::canonicalize(nvm).expect("nvm");
    let local_bin = dunce::canonicalize(local_bin).expect("local bin");
    let profile = dunce::canonicalize(profile).expect("profile");
    let sandbox_temp = dunce::canonicalize(sandbox_temp).expect("sandbox temp");
    let scope = Scope {
        mode: Mode::ProfileCapture,
        roots: vec![allowed.clone()],
        read_dirs: vec![nvm, local_bin],
        read_files: vec![profile],
    };
    let roots = sandbox_roots(&scope, &sandbox_temp);
    let sandbox_policy = policy(&roots, scope.mode, None);
    assert!(!sandbox_policy.contains("(allow network*)"));
    assert!(!sandbox_policy.contains("subpath \"/private/tmp\""));
    assert!(!sandbox_policy.contains("xcrun_db"));
    let mut command = Command::new(SANDBOX_EXEC);
    command
        .current_dir(&sandbox_temp)
        .env("HOME", &profile_home)
        .env("ZDOTDIR", &profile_home)
        .env("BEAVER_ESCAPE", &escape)
        .env("PATH", "/usr/bin:/bin")
        .env("TMPDIR", &sandbox_temp)
        .env("TMPPREFIX", sandbox_temp.join("zsh"));
    add_parameters(&mut command, "BEAVER_RW_DIR", &roots.write_dirs);
    add_parameters(&mut command, "BEAVER_RW_FILE", &roots.write_files);
    add_parameters(&mut command, "BEAVER_RO_DIR", &roots.read_dirs);
    add_parameters(&mut command, "BEAVER_RO_FILE", &roots.read_files);
    let output = command
        .args([
            "-p",
            &sandbox_policy,
            "/bin/zsh",
            "-l",
            "-i",
            "-c",
            "printf '%s' \"$PATH\"",
        ])
        .output()
        .expect("sandbox-exec");

    assert_success(&output);
    let captured = String::from_utf8_lossy(&output.stdout);
    assert!(captured.contains("/.nvm/versions/node/current/bin"));
    assert!(captured.contains("/.local/bin"));
    assert!(!escape.exists());
}

fn run_sandboxed(allowed: &std::path::Path, sandbox_temp: &std::path::Path, script: &str) -> Output {
    let scope = Scope::workspace(vec![allowed.to_path_buf()]);
    run_sandboxed_scope(&scope, sandbox_temp, script)
}

fn run_sandboxed_scope(scope: &Scope, sandbox_temp: &std::path::Path, script: &str) -> Output {
    let roots = sandbox_roots(scope, sandbox_temp);
    let working_dir = if scope.mode == Mode::Workspace {
        scope.roots.first().map(std::path::PathBuf::as_path).unwrap_or(sandbox_temp)
    } else {
        sandbox_temp
    };
    let mut command = Command::new(SANDBOX_EXEC);
    command
        .current_dir(working_dir)
        .env("TMPDIR", sandbox_temp)
        .env("TMPPREFIX", sandbox_temp.join("zsh"));
    add_parameters(&mut command, "BEAVER_RW_DIR", &roots.write_dirs);
    add_parameters(&mut command, "BEAVER_RW_FILE", &roots.write_files);
    add_parameters(&mut command, "BEAVER_RO_DIR", &roots.read_dirs);
    add_parameters(&mut command, "BEAVER_RO_FILE", &roots.read_files);
    command
        .args([
            "-p",
            &policy(&roots, scope.mode, super::xcrun_cache_rule().as_deref()),
            "/bin/zsh",
            "-c",
            script,
        ])
        .output()
        .expect("sandbox-exec")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "sandbox command failed ({:?})\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        bounded_output(&output.stdout),
        bounded_output(&output.stderr),
    );
}

fn bounded_output(bytes: &[u8]) -> String {
    const MAX_DIAGNOSTIC_BYTES: usize = 4_096;
    String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_DIAGNOSTIC_BYTES)]).into_owned()
}
