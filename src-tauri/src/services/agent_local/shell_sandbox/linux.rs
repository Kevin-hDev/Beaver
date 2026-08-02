use landlock::{AccessFs, ABI, Access, CompatLevel, Compatible, Ruleset, RulesetAttr};
use landlock::RulesetCreatedAttr;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};

const MAX_READ_ROOTS: usize = 64;

pub(super) fn run(
    executable: &Path,
    arguments: &[OsString],
    writable_roots: &[PathBuf],
    temp_dir: &Path,
) -> Result<i32, String> {
    apply(writable_roots, temp_dir)?;
    let error = std::process::Command::new(executable).args(arguments).exec();
    Err(error.to_string())
}

fn apply(writable_roots: &[PathBuf], temp_dir: &Path) -> Result<(), String> {
    // ABI V3 protège aussi les troncatures tout en restant compatible avec
    // davantage de noyaux que les versions Landlock plus récentes.
    let abi = ABI::V3;
    let access_all = AccessFs::from_all(abi);
    let access_read = AccessFs::from_read(abi);
    let read_roots = read_roots(writable_roots);
    let write_roots = writable_roots
        .iter()
        .map(PathBuf::as_path)
        .chain(std::iter::once(temp_dir));
    let ruleset = Ruleset::default()
        .set_compatibility(CompatLevel::HardRequirement)
        .handle_access(access_all)
        .map_err(|_| sandbox_error())?
        .create()
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(read_roots, access_read))
        .map_err(|_| sandbox_error())?
        .add_rules(landlock::path_beneath_rules(write_roots, access_all))
        .map_err(|_| sandbox_error())?
        .no_new_privs(true);
    let status = ruleset.restrict_self().map_err(|_| sandbox_error())?;
    if status.ruleset == landlock::RulesetStatus::NotEnforced {
        return Err(sandbox_error());
    }
    Ok(())
}

fn read_roots(writable_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = ["/bin", "/sbin", "/usr", "/lib", "/lib64", "/etc", "/dev"]
        .into_iter()
        .map(PathBuf::from)
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    for path in std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()) {
        if roots.len() >= MAX_READ_ROOTS {
            break;
        }
        let Some(path) = path
            .is_absolute()
            .then(|| dunce::canonicalize(path).ok())
            .flatten()
            .filter(|path| path.is_dir())
        else {
            continue;
        };
        let overlaps_writable = writable_roots
            .iter()
            .any(|root| path.starts_with(root) || root.starts_with(&path));
        if !overlaps_writable && !roots.contains(&path) {
            roots.push(path);
        }
    }
    roots
}

fn sandbox_error() -> String {
    super::launch::sandbox_error()
}
