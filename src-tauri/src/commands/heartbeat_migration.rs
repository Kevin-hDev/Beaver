use super::{ConflictDecision, MigrationStatusView};
use std::path::Path;
use uuid::Uuid;

pub(super) async fn status_at(
    root: &Path,
    timezone: Option<&str>,
) -> Result<MigrationStatusView, String> {
    let timezone = timezone
        .map(crate::commands::heartbeat_validation::timezone)
        .transpose()?;
    let status = crate::services::automations::migration::migrate_legacy(root, timezone)
        .await
        .map_err(|_| "migration_unavailable")?;
    Ok(MigrationStatusView::from(status))
}

pub(super) async fn resolve_at(
    root: &Path,
    legacy_id: String,
    decision: ConflictDecision,
    timezone: Option<String>,
) -> Result<Option<Uuid>, String> {
    use crate::services::automations::migration_conflict::ConflictResolution;
    let resolution = match decision {
        ConflictDecision::RemoveHistorical => ConflictResolution::RemoveHistorical,
        ConflictDecision::ImportAsNew => ConflictResolution::ImportAsNew {
            timezone: crate::commands::heartbeat_validation::timezone(
                timezone.as_deref().ok_or("invalid_timezone")?,
            )?,
        },
    };
    crate::services::automations::migration_conflict::resolve_audited_at(
        root,
        &super::store::ui_actor(),
        &legacy_id,
        resolution,
    )
    .await
    .map_err(|_| "migration_unavailable".into())
}

impl From<crate::services::automations::migration::AutomationMigrationStatus>
    for MigrationStatusView
{
    fn from(value: crate::services::automations::migration::AutomationMigrationStatus) -> Self {
        use crate::services::automations::migration::AutomationMigrationStatus as Status;
        match value {
            Status::Ready => Self {
                status: "ready",
                conflicts: Vec::new(),
            },
            Status::NeedsTimezone => Self {
                status: "needs_timezone",
                conflicts: Vec::new(),
            },
            Status::Conflicts(conflicts) => Self {
                status: "conflicts",
                conflicts,
            },
        }
    }
}
