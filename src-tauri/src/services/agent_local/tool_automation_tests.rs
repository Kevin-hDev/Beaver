use super::tool_automation::public_value;
use crate::models::{ScheduledWakeup, WakeupSchedule};

#[test]
fn public_view_omits_runtime_internals() {
    let wakeup = ScheduledWakeup {
        id: "id".into(),
        name: "audit".into(),
        model: "private-model".into(),
        provider: "private-provider".into(),
        prompt: "inspect".into(),
        schedule: WakeupSchedule::Daily {
            time: "08:00".into(),
        },
        description: String::new(),
        project_id: None,
        active: true,
        paused_by_global: false,
        created_at: String::new(),
    };

    let view = public_value(&wakeup);

    assert!(view.get("working_dir").is_none());
    assert!(view.get("tool_names").is_none());
    assert!(view.get("model").is_none());
    assert_eq!(view["name"], "audit");
}
