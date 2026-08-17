#![allow(dead_code)]

use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;
use std::time::Instant;
use tokio_util::sync::CancellationToken;

use super::constants::{MAX_PROBE_SOCKET_RECORDS, PROBE_ENDPOINT_POLL_INTERVAL};
use super::durable_fs::{platform_fs, OllamaDurableFs};
use super::error::OllamaErrorCode;
use super::path_identity::{CanonicalDirectory, PathIdentityResolver};
use super::path_identity_resolver::NativePathIdentityResolver;
use super::probe::TargetValidation;
use super::spawn_profile::OllamaSpawnProfile;
use super::types::OllamaEndpoint;
use crate::services::owned_process::{OwnedProcess, OwnedProcessIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EndpointWaitResult {
    Ready,
    Cancelled,
    Deadline,
}

pub(crate) async fn wait_for_owned_endpoint(
    endpoint: &OllamaEndpoint,
    identity: OwnedProcessIdentity,
    deadline: Instant,
    cancellation: &CancellationToken,
) -> EndpointWaitResult {
    loop {
        if cancellation.is_cancelled() {
            return EndpointWaitResult::Cancelled;
        }
        if Instant::now() >= deadline {
            return EndpointWaitResult::Deadline;
        }
        if endpoint_is_owned(endpoint, identity) {
            return EndpointWaitResult::Ready;
        }
        tokio::select! {
            _ = cancellation.cancelled() => return EndpointWaitResult::Cancelled,
            _ = tokio::time::sleep(PROBE_ENDPOINT_POLL_INTERVAL) => {}
            _ = tokio::time::sleep_until(tokio::time::Instant::from_std(deadline)) => {
                return EndpointWaitResult::Deadline;
            }
        }
    }
}

fn endpoint_is_owned(endpoint: &OllamaEndpoint, expected: OwnedProcessIdentity) -> bool {
    let families = netstat2::AddressFamilyFlags::IPV4 | netstat2::AddressFamilyFlags::IPV6;
    let Ok(sockets) = netstat2::iterate_sockets_info(families, netstat2::ProtocolFlags::TCP) else {
        return false;
    };
    for (index, socket) in sockets.enumerate() {
        if index >= MAX_PROBE_SOCKET_RECORDS {
            break;
        }
        let Ok(socket) = socket else {
            continue;
        };
        let netstat2::ProtocolSocketInfo::Tcp(info) = socket.protocol_socket_info else {
            continue;
        };
        if info.state != netstat2::TcpState::Listen
            || info.local_port != endpoint.port()
            || info.local_addr != IpAddr::V4(Ipv4Addr::LOCALHOST)
        {
            continue;
        }
        if socket
            .associated_pids
            .iter()
            .take(8)
            .any(|pid| pid_belongs_to_owned_scope(*pid, expected))
        {
            return true;
        }
    }
    false
}

fn pid_belongs_to_owned_scope(pid: u32, expected: OwnedProcessIdentity) -> bool {
    pid == expected.pid
        || OwnedProcess::identity(pid)
            .is_ok_and(|identity| identity.native_scope == expected.native_scope)
}

pub(crate) fn prepare_models(profile: &OllamaSpawnProfile) -> Result<(), TargetValidation> {
    let path = profile.models_directory().path();
    match inspect_models(profile)? {
        Some(_) => Ok(()),
        None => {
            let parent = path.parent().ok_or_else(storage_deferred)?;
            NativePathIdentityResolver
                .canonical_directory(parent)
                .map_err(map_path_error)?;
            std::fs::create_dir(path).map_err(|_| storage_deferred())?;
            inspect_models(profile)?
                .map(|_| ())
                .ok_or_else(storage_deferred)
        }
    }
}

pub(crate) fn cleanup_models(profile: &OllamaSpawnProfile) -> bool {
    let path = profile.models_directory().path();
    let Ok(Some(directory)) = inspect_models(profile) else {
        return !path.exists();
    };
    platform_fs().remove_tree_verified(&directory).is_ok() && !path.exists()
}

fn inspect_models(
    profile: &OllamaSpawnProfile,
) -> Result<Option<CanonicalDirectory>, TargetValidation> {
    let path = profile.models_directory().path();
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            Err(storage_deferred())
        }
        Ok(_) => {
            let directory = NativePathIdentityResolver
                .canonical_directory(path)
                .map_err(map_path_error)?;
            if profile.models_directory().identity().is_none()
                || profile.models_directory().identity() == directory.identity()
            {
                Ok(Some(directory))
            } else {
                Err(storage_deferred())
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(storage_deferred()),
    }
}

fn map_path_error(error: OllamaErrorCode) -> TargetValidation {
    match error {
        OllamaErrorCode::OllamaModelStoreConflict => TargetValidation::Deferred {
            code: OllamaErrorCode::OllamaModelStoreConflict,
        },
        _ => storage_deferred(),
    }
}

fn storage_deferred() -> TargetValidation {
    TargetValidation::Deferred {
        code: OllamaErrorCode::OllamaStorageUnavailable,
    }
}

#[allow(dead_code)]
fn _path_marker(_: &Path) {}
