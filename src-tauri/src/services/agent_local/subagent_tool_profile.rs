use serde_json::Value;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SubagentToolProfile {
    Explorer,
    Coder,
}

impl SubagentToolProfile {
    pub fn from_session_type(value: Option<&str>) -> Result<Self, String> {
        match value {
            Some("explorer") => Ok(Self::Explorer),
            Some("coder") => Ok(Self::Coder),
            _ => Err("Profil de sous-agent invalide.".to_string()),
        }
    }

    pub fn tool_names(self, skills_enabled: bool) -> Vec<&'static str> {
        let mut names = match self {
            Self::Explorer => vec![
                "bash",
                "read_file",
                "list_dir",
                "grep",
                "glob",
                "web_search",
                "web_fetch",
            ],
            Self::Coder => vec![
                "bash",
                "read_file",
                "write_file",
                "edit_file",
                "list_dir",
                "grep",
                "glob",
                "web_search",
                "web_fetch",
            ],
        };
        if self == Self::Coder && skills_enabled {
            names.push("load_skill");
        }
        names
    }

    pub fn definitions(self, skills_enabled: bool) -> Vec<Value> {
        let allowed = self.tool_names(skills_enabled);
        super::tool_definitions::get_tool_definitions()
            .into_iter()
            .filter(|definition| {
                definition_name(definition)
                    .is_some_and(|name| allowed.contains(&name))
            })
            .map(|mut definition| {
                if let Some(name) = definition_name(&definition) {
                    if let Some(description) = self.description(name) {
                        definition["function"]["description"] = Value::String(description.into());
                    }
                }
                definition
            })
            .collect()
    }

    #[cfg(test)]
    pub fn definition_names(self, skills_enabled: bool) -> Vec<&'static str> {
        let allowed = self.tool_names(skills_enabled);
        self.definitions(skills_enabled)
            .iter()
            .filter_map(definition_name)
            .filter_map(|name| allowed.iter().copied().find(|allowed| *allowed == name))
            .collect()
    }

    #[cfg(test)]
    pub fn prompt_tool_names(self, skills_enabled: bool) -> Vec<&'static str> {
        self.tool_names(skills_enabled)
    }

    pub fn allows(self, tool_name: &str, skills_enabled: bool) -> bool {
        self.tool_names(skills_enabled).contains(&tool_name)
    }

    pub fn prompt_tools(self, skills_enabled: bool) -> String {
        self.definitions(skills_enabled)
            .iter()
            .filter_map(|definition| {
                let function = definition.get("function")?;
                Some(format!(
                    "- {}: {}",
                    function.get("name")?.as_str()?,
                    function.get("description")?.as_str()?
                ))
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn description(self, tool_name: &str) -> Option<&'static str> {
        match (self, tool_name) {
            (Self::Explorer, "bash") => Some("Run one approved informational command without a shell. Pipes, redirections, mutations, network commands, find, and paths outside the project are refused."),
            (Self::Coder, "bash") => Some("Run build, test, Git, and development commands inside the assigned worktree. Explicit attempts to leave the worktree are refused."),
            (_, "read_file") => Some("Read a UTF-8 text file inside the assigned directory with line numbers."),
            (Self::Coder, "write_file") => Some("Create or replace a file inside the assigned worktree. Read existing files first."),
            (Self::Coder, "edit_file") => Some("Replace one exact string in a previously read file inside the assigned worktree."),
            (_, "list_dir") => Some("List files and directories inside the assigned directory."),
            (_, "grep") => Some("Search file contents inside the assigned directory with a regular expression."),
            (_, "glob") => Some("Find files by name pattern inside the assigned directory."),
            (_, "web_search") => Some("Search the web for current information."),
            (_, "web_fetch") => Some("Fetch and extract content from a web URL."),
            (Self::Coder, "load_skill") => Some("Load one skill from the accessible skill list."),
            _ => None,
        }
    }
}

fn definition_name(definition: &Value) -> Option<&str> {
    definition.get("function")?.get("name")?.as_str()
}
