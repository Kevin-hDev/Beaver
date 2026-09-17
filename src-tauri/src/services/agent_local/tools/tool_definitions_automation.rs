use serde_json::Value;

pub fn automation_definition() -> Value {
    let schedule = serde_json::json!({
        "oneOf": [
            {
                "type": "object",
                "properties": {
                    "kind": {"const": "once"},
                    "local_datetime": {"type": "string"},
                    "timezone": {"type": "string"}
                },
                "required": ["kind", "local_datetime", "timezone"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "kind": {"const": "cron"},
                    "expression": {"type": "string"},
                    "timezone": {"type": "string"}
                },
                "required": ["kind", "expression", "timezone"],
                "additionalProperties": false
            },
            {
                "type": "object",
                "properties": {
                    "kind": {"const": "after_completion"},
                    "delay_minutes": {"type": "integer", "minimum": 1, "maximum": 525600}
                },
                "required": ["kind", "delay_minutes"],
                "additionalProperties": false
            }
        ]
    });
    let patch = serde_json::json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "description": {"anyOf": [{"type":"string"}, {"type":"null"}]},
            "prompt": {"type": "string"},
            "model": {"type": "string"},
            "schedule": schedule.clone(),
            "status": {"type":"string", "enum":["active", "disabled"]}
        },
        "additionalProperties": false
    });
    super::tool_definitions::tool_def(
        "manage_automation",
        "List, inspect, create, update, review history, or delete automations. A new automation inherits this session's provider and defaults to its current model; another compatible model from the same provider may be selected. resume_session always targets this session. Do not update or delete another automation unless the user's request gives a reason to do so.",
        serde_json::json!({
            "oneOf": [
                {
                    "type":"object",
                    "properties":{"action":{"const":"list"}},
                    "required":["action"],
                    "additionalProperties":false
                },
                {
                    "type":"object",
                    "properties":{
                        "action":{"const":"get"},
                        "automation_id":{"type":"string", "format":"uuid"}
                    },
                    "required":["action", "automation_id"],
                    "additionalProperties":false
                },
                {
                    "type":"object",
                    "properties":{
                        "action":{"const":"create"},
                        "name":{"type":"string"},
                        "description":{"type":"string"},
                        "prompt":{"type":"string"},
                        "target_mode":{"type":"string", "enum":["new_session", "resume_session"]},
                        "model":{"type":"string"},
                        "schedule":schedule,
                        "status":{"type":"string", "enum":["active", "disabled"]}
                    },
                    "required":["action", "name", "prompt", "target_mode", "schedule"],
                    "additionalProperties":false
                },
                {
                    "type":"object",
                    "properties":{
                        "action":{"const":"update"},
                        "automation_id":{"type":"string", "format":"uuid"},
                        "patch":patch
                    },
                    "required":["action", "automation_id", "patch"],
                    "additionalProperties":false
                },
                {
                    "type":"object",
                    "properties":{
                        "action":{"const":"history"},
                        "automation_id":{"type":"string", "format":"uuid"},
                        "limit":{"type":"integer", "minimum":1, "maximum":100},
                        "cursor":{"type":"string"}
                    },
                    "required":["action", "automation_id"],
                    "additionalProperties":false
                },
                {
                    "type":"object",
                    "properties":{
                        "action":{"const":"delete"},
                        "automation_id":{"type":"string", "format":"uuid"}
                    },
                    "required":["action", "automation_id"],
                    "additionalProperties":false
                }
            ]
        }),
    )
}
