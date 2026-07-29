use super::extension_tool_set::ExtensionToolSet;

pub async fn record_selection(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
    phase: &str,
) {
    let names = tools.selected_extension_names();
    if !names.is_empty() {
        super::stream_diagnostics::record_extension_tools(session_id, request_id, phase, &names)
            .await;
    }
    if !tools.omitted_plugin_ids.is_empty() {
        super::stream_diagnostics::record_extension_tools(
            session_id,
            request_id,
            "extension_plugins_omitted",
            &tools.omitted_plugin_ids,
        )
        .await;
    }
    record_omitted_core_tools(tools, session_id, request_id).await;
}

async fn record_omitted_core_tools(
    tools: &ExtensionToolSet,
    session_id: &str,
    request_id: &str,
) {
    if tools.omitted_tool_names.is_empty() && tools.additional_omitted_tools == 0 {
        return;
    }
    let mut omitted = tools.omitted_tool_names.clone();
    if tools.additional_omitted_tools > 0 {
        omitted.push(format!(
            "+{} additional tools",
            tools.additional_omitted_tools
        ));
    }
    super::stream_diagnostics::record_extension_tools(
        session_id,
        request_id,
        "provider_core_tools_omitted",
        &omitted,
    )
    .await;
}

pub async fn refresh_and_record(
    tools: &mut ExtensionToolSet,
    session_id: &str,
    request_id: &str,
) -> Result<(), String> {
    tools.refresh_from_session(session_id).await?;
    record_selection(tools, session_id, request_id, "extension_tools_refreshed").await;
    Ok(())
}
