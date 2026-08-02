use super::{add_parameters, policy, sandbox_roots, SANDBOX_EXEC};
use std::process::{Command, Output};

#[test]
fn seatbelt_allows_work_and_temp_inside_the_root_and_blocks_outside_data() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let outside = temp.path().join("outside.txt");
    std::fs::create_dir_all(&allowed).expect("allowed");
    std::fs::create_dir_all(&sandbox_temp).expect("sandbox temp");
    std::fs::write(&outside, "outside-data").expect("outside");
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
printf ok > '{}/inside.txt'
/bin/cat '{}' >/dev/null 2>&1 && exit 91
/bin/sh -c 'printf child'"#,
        sandbox_temp.display(),
        sandbox_temp.display(),
        allowed.display(),
        allowed.display(),
        outside.display(),
    );
    let output = run_sandboxed(&allowed, &sandbox_temp, &script);

    assert_success(&output);
    assert_eq!(std::fs::read_to_string(allowed.join("inside.txt")).unwrap(), "ok");
    assert_eq!(
        std::fs::read_to_string(allowed.join("heredoc.txt")).unwrap(),
        "line-one\nline-two\n"
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout), "child");
}

#[test]
#[ignore = "requires network access and installed development toolchains"]
fn seatbelt_supports_complete_local_development() {
    let temp = tempfile::tempdir().expect("temp");
    let allowed = temp.path().join("allowed");
    let sandbox_temp = temp.path().join("sandbox-temp");
    let outside = temp.path().join("outside.txt");
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
if printf pwn > '{}'; then exit 90; fi
printf verified
"#,
        outside.display(),
    );
    let output = run_sandboxed(&allowed, &sandbox_temp, &script);

    assert_success(&output);
    assert!(!outside.exists());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "verified");
    println!("git=ok\nnpm=ok\ncargo=ok\ngo=ok\nheredoc=ok\ndiff=ok\noutside-write=blocked");
}

fn run_sandboxed(allowed: &std::path::Path, sandbox_temp: &std::path::Path, script: &str) -> Output {
    let roots = sandbox_roots(std::slice::from_ref(&allowed.to_path_buf()), sandbox_temp);
    let mut command = Command::new(SANDBOX_EXEC);
    command
        .current_dir(allowed)
        .env("TMPDIR", sandbox_temp)
        .env("TMPPREFIX", sandbox_temp.join("zsh"));
    add_parameters(&mut command, "BEAVER_RW_DIR", &roots.write_dirs);
    add_parameters(&mut command, "BEAVER_RW_FILE", &roots.write_files);
    add_parameters(&mut command, "BEAVER_RO_DIR", &roots.read_dirs);
    add_parameters(&mut command, "BEAVER_RO_FILE", &roots.read_files);
    command
        .args(["-p", &policy(&roots), "/bin/zsh", "-c", script])
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
