use super::tool_roots_entries::{
    push_private_read_dir, push_resource_write_dir, push_resource_write_file,
};
use super::tool_roots_collect::collect_into;
#[cfg(test)]
use super::tool_roots_collect::collect_from;
use super::tool_roots_path::{canonical_dir, contains_executable};
use std::path::{Path, PathBuf};

pub(super) const MAX_READ_ROOTS: usize = 160;
pub(super) const MAX_WRITE_ROOTS: usize = 100;

#[derive(Default)]
pub(super) struct ToolRoots {
    pub read_dirs: Vec<PathBuf>,
    pub read_files: Vec<PathBuf>,
    pub write_dirs: Vec<PathBuf>,
    pub write_files: Vec<PathBuf>,
    pub read_limit_reached: bool,
    pub write_limit_reached: bool,
}

pub(super) fn collect(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
) -> ToolRoots {
    collect_with_access(
        workspace_roots,
        platform_read_dirs,
        package_prefixes,
        executable,
        true,
    )
}

pub(super) fn collect_read_only(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
) -> ToolRoots {
    collect_with_access(
        workspace_roots,
        platform_read_dirs,
        package_prefixes,
        executable,
        false,
    )
}

fn collect_with_access(
    workspace_roots: &[PathBuf],
    platform_read_dirs: &[&str],
    package_prefixes: &[&str],
    executable: Option<&Path>,
    allow_writes: bool,
) -> ToolRoots {
    let (configured_path, configured_overflow) = super::super::shell_environment::entries();
    let max_paths = super::super::shell_environment::MAX_PATH_INPUTS;
    let mut path_inputs = Vec::with_capacity(max_paths + 1);
    if let Some(parent) = executable.and_then(Path::parent) {
        path_inputs.push(parent.to_path_buf());
    }
    path_inputs.extend(
        configured_path
            .into_iter()
            .take(max_paths + 1 - path_inputs.len()),
    );
    let path_overflow = configured_overflow || path_inputs.len() > max_paths;
    path_inputs.truncate(max_paths);
    let platform = platform_read_dirs.iter().map(PathBuf::from).collect::<Vec<_>>();
    let packages = package_prefixes.iter().map(PathBuf::from).collect::<Vec<_>>();
    let home = dirs::home_dir().and_then(|path| canonical_dir(&path));
    let writable_cache_dirs = if allow_writes {
        home.as_deref()
            .map(|home| super::tool_cache_roots::collect(home, &path_inputs, path_overflow))
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let mut roots = ToolRoots::default();
    if allow_writes {
        push_private_read_dir(
            &mut roots,
            &crate::services::paths::data_dir(),
            workspace_roots,
        );
    }
    collect_into(
        &mut roots,
        workspace_roots,
        &platform,
        &packages,
        home.as_deref(),
        &path_inputs,
        path_overflow || !allow_writes,
        &writable_cache_dirs,
    );
    if allow_writes {
        append_agent_resources(&mut roots, workspace_roots, &path_inputs);
    }
    super::super::shell_diagnostics::record_root_limits(
        path_overflow,
        roots.read_limit_reached,
        roots.write_limit_reached,
    );
    roots
}

fn append_agent_resources(
    roots: &mut ToolRoots,
    workspace_roots: &[PathBuf],
    path_inputs: &[PathBuf],
) {
    let resources = super::super::agent_resource_access::current();
    append_resource_access(roots, workspace_roots, path_inputs, resources);
}

fn append_resource_access(
    roots: &mut ToolRoots,
    workspace_roots: &[PathBuf],
    path_inputs: &[PathBuf],
    resources: super::super::agent_resource_access::AgentResourceAccess,
) {
    let path_dirs = path_inputs
        .iter()
        .filter_map(|path| canonical_dir(path))
        .filter(|path| contains_executable(path))
        .collect::<Vec<_>>();
    for path in resources.directories {
        push_resource_write_dir(roots, &path, workspace_roots, &path_dirs);
    }
    for path in resources.files {
        push_resource_write_file(roots, &path, workspace_roots, &path_dirs);
    }
}

#[cfg(test)]
#[path = "tool_roots_tests.rs"]
mod tests;
