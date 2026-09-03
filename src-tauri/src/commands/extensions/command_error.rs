#[derive(Clone, Copy, Debug)]
pub(in crate::commands) enum ExtensionCommand {
    List,
    AddLocal,
    InstallGit,
    InstallNpm,
    Update,
    Remove,
    SetEnabled,
    SetShowInChat,
    ReloadHost,
    GetHostStatus,
    GetUiCatalog,
    InvokeUiAction,
    GetDiscoveryPreferences,
    SetDiscoveryPreferences,
    RecoverHost,
    OpenSource,
    GetRecoveryState,
    KeepDisabled,
    RetryLoad,
    DiscardLoadingMarker,
    RestoreRecoverySnapshot,
}

impl ExtensionCommand {
    #[cfg(test)]
    pub(in crate::commands) const ALL: [Self; 21] = [
        Self::List,
        Self::AddLocal,
        Self::InstallGit,
        Self::InstallNpm,
        Self::Update,
        Self::Remove,
        Self::SetEnabled,
        Self::SetShowInChat,
        Self::ReloadHost,
        Self::GetHostStatus,
        Self::GetUiCatalog,
        Self::InvokeUiAction,
        Self::GetDiscoveryPreferences,
        Self::SetDiscoveryPreferences,
        Self::RecoverHost,
        Self::OpenSource,
        Self::GetRecoveryState,
        Self::KeepDisabled,
        Self::RetryLoad,
        Self::DiscardLoadingMarker,
        Self::RestoreRecoverySnapshot,
    ];

    pub(in crate::commands) fn label(self) -> &'static str {
        match self {
            Self::List => "list_extensions",
            Self::AddLocal => "add_local_extension",
            Self::InstallGit => "install_git_extension",
            Self::InstallNpm => "install_npm_extension",
            Self::Update => "update_extension",
            Self::Remove => "remove_extension",
            Self::SetEnabled => "set_extension_enabled",
            Self::SetShowInChat => "set_extension_show_in_chat",
            Self::ReloadHost => "reload_extension_host",
            Self::GetHostStatus => "get_extension_host_status",
            Self::GetUiCatalog => "get_extension_ui_catalog",
            Self::InvokeUiAction => "invoke_extension_ui_action",
            Self::GetDiscoveryPreferences => "get_extension_discovery_preferences",
            Self::SetDiscoveryPreferences => "set_extension_discovery_preferences",
            Self::RecoverHost => "recover_extension_host",
            Self::OpenSource => "open_extension_source",
            Self::GetRecoveryState => "get_extension_recovery_state",
            Self::KeepDisabled => "keep_extension_disabled",
            Self::RetryLoad => "retry_extension_load",
            Self::DiscardLoadingMarker => "discard_extension_loading_marker",
            Self::RestoreRecoverySnapshot => "restore_extension_recovery_snapshot",
        }
    }
}

pub(in crate::commands) fn close<T>(
    command: ExtensionCommand,
    result: Result<T, String>,
) -> Result<T, String> {
    result.map_err(|error| crate::services::extensions::close_command_error(command.label(), error))
}
