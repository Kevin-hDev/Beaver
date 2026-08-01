use crate::services::compress::realtime_budget::RealtimeBudget;

pub(super) fn should_interrupt(
    budget: &mut Option<RealtimeBudget>,
    token_count: u32,
    has_pending_tool_call: bool,
) -> bool {
    !has_pending_tool_call
        && budget
            .as_mut()
            .is_some_and(|budget| budget.should_interrupt(token_count))
}
