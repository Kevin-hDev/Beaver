use serde::Serialize;

const MAX_SCENARIOS: usize = 64;

#[derive(Debug, Serialize)]
pub(super) struct FixtureScenario {
    pub(super) requirement: &'static str,
    run_id: String,
    status: &'static str,
    request_count: usize,
    reasoning_event_count: usize,
    decisions: Vec<String>,
}

pub(super) fn validate_session(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> Result<(), String> {
    let users = session
        .messages
        .iter()
        .filter(|message| message.role == "user")
        .count();
    let has_tool = session
        .messages
        .iter()
        .any(|message| message.role == "tool")
        || session
            .diagnostic_runs
            .iter()
            .any(|run| !run.recent_tools.is_empty() || run.last_tool.is_some());
    let has_model_result = session
        .diagnostic_runs
        .iter()
        .any(|run| run.events.iter().any(|event| event.phase == "model_result"));
    let has_payload = session.diagnostic_runs.iter().any(|run| {
        run.events
            .iter()
            .any(|event| event.phase == "provider_payload")
    });
    // DeepSeek rejoue uniquement dans la chaîne d'outil : un seul tour complet
    // prouve capture, rejeu réseau et persistance sans inventer un rejeu utilisateur.
    let minimum_turns = if session.provider == "deepseek" { 1 } else { 2 };
    (users >= minimum_turns
        && session.diagnostic_runs.len() >= minimum_turns
        && has_tool
        && has_model_result
        && has_payload
        && session.diagnostic_runs.len() <= MAX_SCENARIOS)
        .then_some(())
        .ok_or_else(super::unavailable)
}

pub(super) fn collect(
    session: &crate::services::agent_local::types_session::AgentSession,
) -> Vec<FixtureScenario> {
    session
        .diagnostic_runs
        .iter()
        .take(MAX_SCENARIOS)
        .flat_map(|run| {
            let decisions = run
                .events
                .iter()
                .filter(|event| event.phase == "reasoning")
                .map(|event| event.message.clone())
                .collect::<Vec<_>>();
            let request_count = run
                .events
                .iter()
                .filter(|event| event.phase == "provider_payload")
                .count();
            let completed = run.status == "completed" && request_count > 0;
            let capture =
                has_decision(&decisions, "captured") && has_decision(&decisions, "persisted");
            let replay = has_decision(&decisions, "replayed");
            let mut scenarios = Vec::with_capacity(2);
            if capture {
                scenarios.push(FixtureScenario {
                    requirement: "capture_and_persist",
                    run_id: run.request_id.clone(),
                    status: if completed { "passe" } else { "bloque" },
                    request_count,
                    reasoning_event_count: decisions.len(),
                    decisions: decisions.clone(),
                });
            }
            if replay {
                scenarios.push(FixtureScenario {
                    requirement: "replay_and_continue",
                    run_id: run.request_id.clone(),
                    status: if completed && capture {
                        "passe"
                    } else {
                        "bloque"
                    },
                    request_count,
                    reasoning_event_count: decisions.len(),
                    decisions,
                });
            }
            scenarios
        })
        .take(MAX_SCENARIOS)
        .collect()
}

pub(super) fn proves_required_scenarios(scenarios: &[FixtureScenario]) -> bool {
    ["capture_and_persist", "replay_and_continue"]
        .into_iter()
        .all(|requirement| {
            scenarios
                .iter()
                .any(|scenario| scenario.requirement == requirement && scenario.status == "passe")
        })
}

fn has_decision(decisions: &[String], decision: &str) -> bool {
    let marker = format!("decision=\"{decision}\"");
    decisions.iter().any(|value| value.contains(&marker))
}
