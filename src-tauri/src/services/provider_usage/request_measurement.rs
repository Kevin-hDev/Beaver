use std::time::Instant;

use super::request_journal::{
    served_tier, ProviderRequestMetric, RequestMetricStatus, ServiceTierServed,
};
use super::{RequestUsage, UsageApiFormat, UsageWorkload};

pub(crate) struct RequestMeasurementContext<'a> {
    pub connection_id: &'a str,
    pub canonical_provider_id: &'a str,
    pub api_format: UsageApiFormat,
    pub model: &'a str,
    pub session_id: Option<&'a str>,
    pub request_id: &'a str,
    pub turn: Option<u32>,
    pub attempt: u32,
    pub workload: UsageWorkload,
    pub fast_mode: crate::services::llm::fast_mode::FastModeRequest,
}

pub(crate) struct RequestMeasurement {
    metric: ProviderRequestMetric,
    requested_workload: UsageWorkload,
    started: Instant,
}

impl RequestMeasurement {
    pub(crate) fn start(context: RequestMeasurementContext<'_>) -> Option<Self> {
        let metric = ProviderRequestMetric {
            started_at_ms: chrono::Utc::now().timestamp_millis(),
            connection_id: context.connection_id.to_string(),
            canonical_provider_id: context.canonical_provider_id.to_string(),
            api_format: context.api_format,
            model: context.model.to_string(),
            session_id: context.session_id.map(str::to_string),
            request_id: context.request_id.to_string(),
            turn: context.turn,
            attempt: context.attempt,
            workload: workload_name(context.workload).to_string(),
            fast_requested: context.fast_mode.fast_requested(),
            origin: if context.session_id.is_some() {
                "manual_chat"
            } else {
                "automation"
            }
            .to_string(),
            ..Default::default()
        };
        metric.is_valid().then_some(Self {
            metric,
            requested_workload: context.workload,
            started: Instant::now(),
        })
    }

    pub(crate) fn mark_headers(&mut self) {
        let elapsed = elapsed_ms(self.started);
        self.metric.timing.headers_ms.get_or_insert(elapsed);
    }

    pub(crate) fn mark_first_event(&mut self) {
        let elapsed = elapsed_ms(self.started);
        self.metric.timing.first_event_ms.get_or_insert(elapsed);
    }

    pub(crate) fn mark_first_useful(&mut self) {
        let elapsed = elapsed_ms(self.started);
        self.metric.timing.first_useful_ms.get_or_insert(elapsed);
    }

    pub(crate) fn observe_response_metadata(&mut self, value: &serde_json::Value) {
        if self.metric.canonical_provider_id == "openai" {
            let observed = value
                .get("service_tier")
                .or_else(|| value.pointer("/response/service_tier"))
                .and_then(serde_json::Value::as_str)
                .map(served_tier)
                .unwrap_or_default();
            if observed != ServiceTierServed::Unknown {
                self.metric.service_tier_served = observed;
            }
        }
        if self.metric.canonical_provider_id != "openrouter" {
            return;
        }
        let selected = value
            .pointer("/openrouter_metadata/endpoints/available")
            .and_then(serde_json::Value::as_array)
            .and_then(|endpoints| {
                endpoints
                    .iter()
                    .find(|endpoint| endpoint["selected"].as_bool() == Some(true))
            });
        let Some((provider, model)) = selected.and_then(routed_endpoint) else {
            return;
        };
        self.metric.routed_provider = Some(provider.to_string());
        self.metric.routed_model = Some(model.to_string());
    }

    #[cfg(test)]
    pub(crate) fn timing(&self) -> &super::request_journal::RequestTiming {
        &self.metric.timing
    }

    #[cfg(test)]
    pub(crate) fn routed_endpoint(&self) -> (Option<&str>, Option<&str>) {
        (
            self.metric.routed_provider.as_deref(),
            self.metric.routed_model.as_deref(),
        )
    }

    #[cfg(test)]
    pub(crate) fn fast_observation(&self) -> (bool, ServiceTierServed) {
        (self.metric.fast_requested, self.metric.service_tier_served)
    }

    pub(crate) async fn finish(
        mut self,
        status: RequestMetricStatus,
        usage: Option<&RequestUsage>,
        usage_complete: bool,
    ) {
        self.metric.timing.total_ms = elapsed_ms(self.started);
        self.metric.status = status;
        self.metric.usage = usage.cloned();
        self.metric.usage_complete = usage_complete;
        if let Some(session_id) = self.metric.session_id.as_deref() {
            let (origin, workload) =
                super::types::context_for_session(session_id, self.requested_workload).await;
            self.metric.origin = origin_name(origin).to_string();
            self.metric.workload = workload_name(workload).to_string();
        } else {
            self.metric.origin = "automation".to_string();
        }
        if super::request_journal::record(self.metric.clone())
            .await
            .is_ok()
        {
            super::emit_update(&self.metric.connection_id);
        }
    }
}

fn routed_endpoint(value: &serde_json::Value) -> Option<(&str, &str)> {
    let provider = value["provider"].as_str()?.trim();
    let model = value["model"].as_str()?.trim();
    (super::request_journal_validation::valid_router_label(provider)
        && super::request_journal_validation::valid_label(model, 128))
    .then_some((provider, model))
}

fn elapsed_ms(started: Instant) -> u64 {
    started.elapsed().as_millis().try_into().unwrap_or(u64::MAX)
}

fn workload_name(value: UsageWorkload) -> &'static str {
    match value {
        UsageWorkload::Primary => "primary",
        UsageWorkload::Subagent => "subagent",
        UsageWorkload::Compression => "compression",
    }
}

fn origin_name(value: super::UsageOrigin) -> &'static str {
    match value {
        super::UsageOrigin::ManualChat => "manual_chat",
        super::UsageOrigin::ExternalChannel => "external_channel",
        super::UsageOrigin::Automation => "automation",
    }
}
