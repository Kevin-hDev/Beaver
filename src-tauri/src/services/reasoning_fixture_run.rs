//! Contexte de run debug propriétaire des seuls outils de fixture.

use serde_json::{json, Value};

use super::reasoning_fixture_tools::FixtureToolset;

const WRITE_NOTE: &str = "fixture.write_note";
const READ_NOTE: &str = "fixture.read_note";

pub struct FixtureRunContext {
    tools: FixtureToolset,
}

impl FixtureRunContext {
    pub async fn start() -> Result<Self, String> {
        Ok(Self {
            tools: super::reasoning_fixture_tools::isolated_toolset().await?,
        })
    }

    /// Ces définitions ne sont jamais ajoutées au catalogue Agent Local normal.
    pub fn definitions(&self) -> [Value; 2] {
        [
            definition(
                WRITE_NOTE,
                "Write deterministic fixture data.",
                json!({
                    "type": "object", "properties": {"value": {"type": "string"}},
                    "required": ["value"], "additionalProperties": false
                }),
            ),
            definition(
                READ_NOTE,
                "Read deterministic fixture data.",
                json!({
                    "type": "object", "properties": {}, "additionalProperties": false
                }),
            ),
        ]
    }

    pub async fn dispatch(&mut self, tool_id: &str, arguments: &Value) -> Result<Value, String> {
        if !self.definitions().iter().any(|definition| {
            definition.pointer("/function/name").and_then(Value::as_str) == Some(tool_id)
        }) {
            return Err(unavailable());
        }
        self.tools
            .execute(tool_id, arguments)
            .await
            .map_err(|_| unavailable())
    }

    #[cfg(test)]
    fn root_for_test(&self) -> std::path::PathBuf {
        self.tools.root_for_test()
    }
}

fn definition(name: &str, description: &str, parameters: Value) -> Value {
    json!({"type":"function", "function":{"name":name,"description":description,"parameters":parameters}})
}

fn unavailable() -> String {
    "Outil de fixture indisponible".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn run_owns_a_closed_toolset_and_drops_it_after_an_error() {
        let root = {
            let mut run = FixtureRunContext::start().await.expect("run");
            let names = run
                .definitions()
                .map(|value| value["function"]["name"].clone());
            assert_eq!(names, [json!(WRITE_NOTE), json!(READ_NOTE)]);
            assert_eq!(
                run.dispatch(WRITE_NOTE, &json!({"value":"fixture"}))
                    .await
                    .unwrap(),
                json!({"written":true})
            );
            assert!(run
                .dispatch("bash", &json!({"command":"pwd"}))
                .await
                .is_err());
            run.root_for_test()
        };
        assert!(!root.exists());
    }
}
