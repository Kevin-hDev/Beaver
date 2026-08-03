use crate::services::agent_local::types_tools::ToolResult;
use serde_json::Value;
use std::path::Path;

pub(super) struct DelegateIdentity {
    pub(super) prompt: String,
    pub(super) name: String,
    pub(super) description: String,
}

pub(super) fn resolve(
    args: &Value,
    working_dir: &Path,
    subagent_type: &str,
    mission_prompt: String,
) -> Result<DelegateIdentity, ToolResult> {
    let agent = args["agent_path"]
        .as_str()
        .map(|path| super::agent_definition::load(path, working_dir))
        .transpose()?;
    if agent
        .as_ref()
        .is_some_and(|definition| definition.profile != subagent_type)
    {
        return Err(ToolResult::validation(
            "agent_profile_mismatch",
            "Le profil demandé ne correspond pas à la définition d'agent.",
        ));
    }

    let prompt = agent.as_ref().map_or(mission_prompt.clone(), |definition| {
        format!(
            "<specialized_agent>\n{}\n</specialized_agent>\n\n<mission>\n{}\n</mission>",
            definition.body, mission_prompt
        )
    });
    if prompt.chars().count() > super::subagent_instruction_delivery::MAX_PROMPT_SIZE {
        return Err(ToolResult::validation(
            "agent_prompt_too_long",
            "Instructions du sous-agent trop longues.",
        ));
    }

    let supplied_name = args["display_name"]
        .as_str()
        .or_else(|| args["name"].as_str());
    let name = agent.as_ref().map_or_else(
        || super::subagent_profile::clean_name(supplied_name, subagent_type),
        |definition| definition.name.clone(),
    );
    let legacy_label = super::subagent_profile::legacy_mission_label(
        supplied_name,
        subagent_type,
    );
    let description_owned = agent
        .as_ref()
        .map(|definition| definition.description.clone())
        .or_else(|| {
            args["description"]
                .as_str()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(ToString::to_string)
                .or(legacy_label)
        });
    let description =
        super::subagent_profile::clean_description(description_owned.as_deref(), &prompt);

    Ok(DelegateIdentity {
        prompt,
        name,
        description,
    })
}
