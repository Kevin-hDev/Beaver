use super::agent_loop_request_types::ApiRequestOutput;
pub(super) use super::agent_loop_request_types::ApiRequestParams;
use crate::services::agent_local::context_usage_buckets::RequestContextUsage;
use crate::services::compress::realtime_budget::RealtimeBudget;

pub(super) async fn run(params: ApiRequestParams<'_>) -> Result<ApiRequestOutput, String> {
    let completion_cancel = params.cancel.clone();
    params
        .subagents
        .prepare_for_model_request(params.messages)
        .await?;
    #[cfg(test)]
    if let Some(output) = super::agent_loop_test_request::run(
        params.request_id,
        &params.cancel,
        params.subagents,
        params.messages,
        &completion_cancel,
    )
    .await?
    {
        return Ok(output);
    }
    crate::services::agent_local::tool_result_budget::apply_budget(params.messages);
    let report = crate::services::agent_local::context_budget::prepare_for_request(
        params.messages,
        params.configured_context,
        params.tools,
        params.provider_id,
    )?;
    crate::services::agent_local::context_budget::record_repairs(
        &report,
        params.session_id,
        params.request_id,
    )
    .await;
    let breakdown = RequestContextUsage::from_request(
        params.provider_id,
        params.messages,
        params.tools,
        params.context_usage_seed,
    );
    let realtime_budget =
        RealtimeBudget::pending_for_session(params.session_id, params.configured_context).await;
    let plan_active = crate::services::agent_local::agent_loop_plan::active(
        params.session_id,
        params.plan_mode_active,
    )
    .await;
    crate::services::agent_local::stream_diagnostics_model::record_model_request(
        params.session_id,
        params.request_id,
        params.turn,
        params.messages,
    )
    .await;
    crate::services::agent_local::stream_diagnostics::mark_phase(
        params.session_id,
        params.request_id,
        "model_stream",
        "Stream modèle démarré.",
    )
    .await;
    let purpose = crate::services::llm::request_purpose::RequestPurpose::for_request(
        params.session_id,
        params.request_id,
    )
    .await;
    let mut next_attempt = 1_u32;
    let turn = super::agent_loop_turn::metric_turn(params.turn);
    let first_preparation =
        super::agent_loop_request_context::prepared_attempt(&params, 1, breakdown)
            .with_realtime_budget(realtime_budget.clone());
    let first_attempt = super::retry::retry_stream(
        params.on_event,
        params.session_id,
        params.request_id,
        turn,
        &mut next_attempt,
        params.provider_id,
        params.fast_mode,
        purpose,
        params.model,
        params.messages,
        params.tools,
        params.think,
        params.reasoning_mode,
        params.tool_result_previews,
        params.cancel.clone(),
        plan_active,
        realtime_budget,
        params.continuation_target.as_ref(),
        Some(&first_preparation),
    )
    .await;
    let (outcome, completed_attempt) = match first_attempt {
        Ok(outcome) => (outcome, 1),
        Err(error) if error == "provider_payload_too_large" => {
            let changed =
                crate::services::agent_local::context_budget::reduce_after_payload_too_large(
                    params.messages,
                    params.configured_context,
                    params.tools,
                    params.provider_id,
                )?;
            if !changed {
                return Err(error);
            }
            let _reduced_report =
                crate::services::agent_local::context_budget::prepare_for_request(
                    params.messages,
                    params.configured_context,
                    params.tools,
                    params.provider_id,
                )?;
            let breakdown = RequestContextUsage::from_request(
                params.provider_id,
                params.messages,
                params.tools,
                params.context_usage_seed,
            );
            let reduced_budget =
                RealtimeBudget::pending_for_session(params.session_id, params.configured_context)
                    .await;
            let reduced_preparation =
                super::agent_loop_request_context::prepared_attempt(&params, 2, breakdown)
                    .with_realtime_budget(reduced_budget.clone());
            crate::services::agent_local::stream_diagnostics::record_retry(
                params.session_id,
                params.request_id,
                "Requête provider réduite après un rejet de taille.",
            )
            .await;
            let outcome = super::retry::retry_stream(
                params.on_event,
                params.session_id,
                params.request_id,
                turn,
                &mut next_attempt,
                params.provider_id,
                params.fast_mode,
                purpose,
                params.model,
                params.messages,
                params.tools,
                params.think,
                params.reasoning_mode,
                params.tool_result_previews,
                params.cancel.clone(),
                plan_active,
                reduced_budget,
                params.continuation_target.as_ref(),
                Some(&reduced_preparation),
            )
            .await?;
            return super::agent_loop_request_finish::finish(
                params,
                outcome,
                2,
                plan_active,
                completion_cancel,
            )
            .await;
        }
        Err(error) => return Err(error),
    };
    super::agent_loop_request_finish::finish(
        params,
        outcome,
        completed_attempt,
        plan_active,
        completion_cancel,
    )
    .await
}

#[cfg(test)]
#[path = "agent_loop_request_fast_mode_tests.rs"]
mod fast_mode_tests;

#[cfg(test)]
#[path = "agent_loop_completion_tests.rs"]
mod completion_tests;
