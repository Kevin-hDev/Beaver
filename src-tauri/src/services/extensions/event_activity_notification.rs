use serde_json::Value;

pub(super) fn receive(
    params: Option<&Value>,
    work: &super::work_supervision::ExtensionWorkServices,
    authority: &super::host_reader::HostAuthority,
) -> Result<(), String> {
    let activity: super::types::ExtensionEventActivity = serde_json::from_value(
        params
            .cloned()
            .ok_or_else(|| "Réponse de l'hôte d'extensions invalide.".to_string())?,
    )
    .map_err(|_| "Réponse de l'hôte d'extensions invalide.".to_string())?;
    if !activity.is_bounded() {
        return Err("Réponse de l'hôte d'extensions invalide.".to_string());
    }
    work.event_router().update_host_activity(
        &authority.identity,
        authority.generation.number,
        activity,
    );
    Ok(())
}
