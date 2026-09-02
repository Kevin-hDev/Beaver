use super::host_process::HostProcess;
use super::protocol::{HelloResult, LoadResult};
use super::types::ExtensionApiLevel;
use serde_json::json;

pub(super) async fn load_specs(
    process: &HostProcess,
    specs: &[super::protocol::HostExtensionSpec],
    responses: &mut Vec<LoadResult>,
    recovery: &super::runtime_sync::RecoveryPreflight,
) -> Result<(), ()> {
    if process.request("host.reset", json!({})).await.is_err() {
        return Err(());
    }
    let mut loaded = Vec::with_capacity(specs.len());
    for specification in specs {
        match process
            .load(specification, recovery.attempts_for(&specification.id))
            .await
            .and_then(super::runtime::parse::<LoadResult>)
        {
            Ok(response) => loaded.push(response),
            Err(_) => return Err(()),
        }
    }
    responses.extend(loaded);
    Ok(())
}

pub(super) async fn validate_hello(process: &HostProcess) -> Result<HelloResult, String> {
    let hello =
        super::runtime::parse::<HelloResult>(process.request("host.hello", json!({})).await?)?;
    if hello.api_version != super::types::BEAVER_API_VERSION {
        return Err(super::error_codes::HOST_INCOMPATIBLE.to_string());
    }
    super::runtime_version::validate_node(&hello.node_version)?;
    Ok(hello)
}

pub(super) fn official_api_level(
    specs: &[super::protocol::HostExtensionSpec],
) -> ExtensionApiLevel {
    if specs
        .iter()
        .any(|spec| spec.manifest.api_level == ExtensionApiLevel::Advanced)
    {
        ExtensionApiLevel::Advanced
    } else {
        ExtensionApiLevel::Stable
    }
}
