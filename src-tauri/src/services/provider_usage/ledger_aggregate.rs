use super::pricing::ResolvedCost;
use super::request_usage::RequestUsage;
use super::types::{UsageAggregate, UsageBreakdown, UsageOrigin, UsageWorkload};
use chrono::NaiveDate;
use std::collections::BTreeMap;

pub fn add(
    target: &mut UsageBreakdown,
    origin: UsageOrigin,
    workload: UsageWorkload,
    usage: &RequestUsage,
    cost: ResolvedCost,
) {
    add_total(&mut target.totals, usage, cost);
    let origin_total = match origin {
        UsageOrigin::ManualChat => &mut target.origins.manual_chat,
        UsageOrigin::ExternalChannel => &mut target.origins.external_channel,
        UsageOrigin::Automation => &mut target.origins.automation,
    };
    add_total(origin_total, usage, cost);
    let workload_total = match workload {
        UsageWorkload::Primary => &mut target.workloads.primary,
        UsageWorkload::Subagent => &mut target.workloads.subagent,
        UsageWorkload::Compression => &mut target.workloads.compression,
    };
    add_total(workload_total, usage, cost);
}

pub fn sum_since(days: &BTreeMap<String, UsageBreakdown>, from: NaiveDate) -> UsageBreakdown {
    let mut total = UsageBreakdown::default();
    for (date, day) in days {
        if NaiveDate::parse_from_str(date, "%Y-%m-%d").is_ok_and(|date| date >= from) {
            merge(&mut total, day);
        }
    }
    total
}

fn add_total(target: &mut UsageAggregate, usage: &RequestUsage, cost: ResolvedCost) {
    target.request_count = target.request_count.saturating_add(1);
    if !usage.is_empty() {
        target.usage_request_count = target.usage_request_count.saturating_add(1);
    }
    target.tokens.input_tokens = target
        .tokens
        .input_tokens
        .saturating_add(usage.input_tokens.unwrap_or(0));
    target.tokens.output_tokens = target
        .tokens
        .output_tokens
        .saturating_add(usage.output_tokens.unwrap_or(0));
    target.tokens.cached_input_tokens = target
        .tokens
        .cached_input_tokens
        .saturating_add(usage.cached_input_tokens.unwrap_or(0));
    target.tokens.reasoning_output_tokens = target
        .tokens
        .reasoning_output_tokens
        .saturating_add(usage.reasoning_output_tokens.unwrap_or(0));
    target.tokens.total_tokens = target
        .tokens
        .total_tokens
        .saturating_add(usage.total_tokens.unwrap_or(0));
    if let Some(micros) = cost.micros {
        target.cost_usd_micros = target.cost_usd_micros.saturating_add(micros);
        target.priced_request_count = target.priced_request_count.saturating_add(1);
        if cost.exact {
            target.exact_cost_request_count = target.exact_cost_request_count.saturating_add(1);
        }
    }
}

fn merge(target: &mut UsageBreakdown, source: &UsageBreakdown) {
    merge_total(&mut target.totals, &source.totals);
    merge_total(&mut target.origins.manual_chat, &source.origins.manual_chat);
    merge_total(
        &mut target.origins.external_channel,
        &source.origins.external_channel,
    );
    merge_total(&mut target.origins.automation, &source.origins.automation);
    merge_total(&mut target.workloads.primary, &source.workloads.primary);
    merge_total(&mut target.workloads.subagent, &source.workloads.subagent);
    merge_total(
        &mut target.workloads.compression,
        &source.workloads.compression,
    );
}

fn merge_total(target: &mut UsageAggregate, source: &UsageAggregate) {
    target.tokens.input_tokens = target
        .tokens
        .input_tokens
        .saturating_add(source.tokens.input_tokens);
    target.tokens.output_tokens = target
        .tokens
        .output_tokens
        .saturating_add(source.tokens.output_tokens);
    target.tokens.cached_input_tokens = target
        .tokens
        .cached_input_tokens
        .saturating_add(source.tokens.cached_input_tokens);
    target.tokens.reasoning_output_tokens = target
        .tokens
        .reasoning_output_tokens
        .saturating_add(source.tokens.reasoning_output_tokens);
    target.tokens.total_tokens = target
        .tokens
        .total_tokens
        .saturating_add(source.tokens.total_tokens);
    target.request_count = target.request_count.saturating_add(source.request_count);
    target.usage_request_count = target
        .usage_request_count
        .saturating_add(source.usage_request_count);
    target.cost_usd_micros = target
        .cost_usd_micros
        .saturating_add(source.cost_usd_micros);
    target.priced_request_count = target
        .priced_request_count
        .saturating_add(source.priced_request_count);
    target.exact_cost_request_count = target
        .exact_cost_request_count
        .saturating_add(source.exact_cost_request_count);
}
