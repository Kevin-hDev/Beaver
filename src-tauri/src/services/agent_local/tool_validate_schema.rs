#[derive(Clone, Copy)]
pub(super) enum Ty {
    Str,
    Int,
    Float,
    Arr,
    Obj,
    Bool,
}

pub(super) type Schema = &'static [(&'static str, Ty, bool)];

static BASH: Schema = &[("command", Ty::Str, true), ("timeout", Ty::Int, false)];
static READ_FILE: Schema = &[
    ("path", Ty::Str, true),
    ("offset", Ty::Int, false),
    ("limit", Ty::Int, false),
];
static WRITE_FILE: Schema = &[("path", Ty::Str, true), ("content", Ty::Str, true)];
static EDIT_FILE: Schema = &[
    ("path", Ty::Str, true),
    ("old_string", Ty::Str, true),
    ("new_string", Ty::Str, true),
];
static LIST_DIR: Schema = &[("path", Ty::Str, false)];
static GREP: Schema = &[
    ("pattern", Ty::Str, true),
    ("path", Ty::Str, false),
    ("glob", Ty::Str, false),
];
static GLOB: Schema = &[("pattern", Ty::Str, true), ("path", Ty::Str, false)];
static WEB_SEARCH: Schema = &[("query", Ty::Str, true)];
static WEB_FETCH: Schema = &[("url", Ty::Str, true)];
static TODO_WRITE: Schema = &[("todos", Ty::Arr, true)];
static TODO_HISTORY: Schema = &[];
static TODO_PAUSE: Schema = &[("reason", Ty::Str, false)];
static TODO_RESUME: Schema = &[("id", Ty::Str, true)];
static TODO_DELETE: Schema = &[("id", Ty::Str, false), ("active", Ty::Bool, false)];
static AGENT_DIAGNOSTICS: Schema = &[("limit", Ty::Int, false)];
static ASK_USER_CHOICE: Schema = &[("questions", Ty::Arr, true)];
static PLANMODE: Schema = &[("title", Ty::Str, true), ("content", Ty::Str, true)];
static EXITPLANMODE: Schema = &[("status", Ty::Str, true)];
static LOAD_SKILL: Schema = &[("skill_id", Ty::Str, true)];
static CREATE_BRANCH: Schema = &[("branch_name", Ty::Str, true)];
static CHECKOUT_BRANCH: Schema = &[("branch_name", Ty::Str, true)];
static DELEGATE_TASK: Schema = &[
    ("prompt", Ty::Str, true),
    ("subagent_type", Ty::Str, true),
    ("name", Ty::Str, false),
    ("display_name", Ty::Str, false),
    ("description", Ty::Str, false),
    ("subagent_id", Ty::Str, false),
];
static SUBAGENT_ID: Schema = &[("subagent_id", Ty::Str, true)];
static MESSAGE_SUBAGENT: Schema = &[("subagent_id", Ty::Str, true), ("prompt", Ty::Str, true)];
static READ_SPREADSHEET: Schema = &[
    ("path", Ty::Str, true),
    ("sheet", Ty::Str, false),
    ("range", Ty::Str, false),
    ("max_rows", Ty::Int, false),
];
static READ_DOCUMENT: Schema = &[("path", Ty::Str, true), ("pages", Ty::Str, false)];
static READ_IMAGE: Schema = &[("path", Ty::Str, true)];
static WRITE_SPREADSHEET: Schema = &[("path", Ty::Str, true), ("operations", Ty::Arr, true)];
static WRITE_DOCUMENT: Schema = &[("path", Ty::Str, true), ("content", Ty::Arr, true)];
static PROCESS_IMAGE: Schema = &[
    ("input_path", Ty::Str, true),
    ("output_path", Ty::Str, true),
    ("operations", Ty::Arr, false),
];
static SEARCH_MCP: Schema = &[
    ("mode", Ty::Str, true),
    ("query", Ty::Str, false),
    ("tool_id", Ty::Str, false),
    ("arguments", Ty::Obj, false),
];
static FORECAST_DATA_AUDIT: Schema = &[
    ("data", Ty::Str, false),
    ("file_path", Ty::Str, false),
    ("target_column", Ty::Str, true),
    ("date_column", Ty::Str, true),
    ("series_column", Ty::Str, false),
    ("covariate_columns", Ty::Arr, false),
    ("horizon", Ty::Int, true),
    ("frequency", Ty::Str, true),
    ("confidence_level", Ty::Float, false),
];
static FORECAST_ANALYZE: Schema = &[
    ("analysis_id", Ty::Str, true),
    ("action", Ty::Str, true),
    ("params", Ty::Obj, true),
];
static FORECAST_READ: Schema = &[
    ("analysis_id", Ty::Str, false),
    ("offset", Ty::Int, false),
    ("limit", Ty::Int, false),
];
static FORECAST_MODELS: Schema = &[];
static FORECAST_BACKTEST: Schema = &[
    ("analysis_id", Ty::Str, true),
    ("model_ids", Ty::Arr, false),
    ("max_windows", Ty::Int, false),
];
static FORECAST_COMPARE_MODELS: Schema = &[("analysis_id", Ty::Str, true)];

pub(super) fn schema(tool: &str) -> Option<Schema> {
    Some(match tool {
        "bash" => BASH,
        "read_file" => READ_FILE,
        "write_file" => WRITE_FILE,
        "edit_file" => EDIT_FILE,
        "list_dir" => LIST_DIR,
        "grep" => GREP,
        "glob" => GLOB,
        "web_search" => WEB_SEARCH,
        "web_fetch" => WEB_FETCH,
        "todo_write" => TODO_WRITE,
        "todo_history" => TODO_HISTORY,
        "todo_pause" => TODO_PAUSE,
        "todo_resume" => TODO_RESUME,
        "todo_delete" => TODO_DELETE,
        "agent_diagnostics" => AGENT_DIAGNOSTICS,
        "ask_user_choice" => ASK_USER_CHOICE,
        "planmode" => PLANMODE,
        "exitplanmode" => EXITPLANMODE,
        "load_skill" => LOAD_SKILL,
        "create_branch" => CREATE_BRANCH,
        "checkout_branch" => CHECKOUT_BRANCH,
        "delegate_task" => DELEGATE_TASK,
        "list_subagents" => &[],
        "get_subagent" | "cancel_subagent" | "archive_subagent" => SUBAGENT_ID,
        "message_subagent" => MESSAGE_SUBAGENT,
        "read_spreadsheet" => READ_SPREADSHEET,
        "read_document" => READ_DOCUMENT,
        "read_image" => READ_IMAGE,
        "write_spreadsheet" => WRITE_SPREADSHEET,
        "write_document" => WRITE_DOCUMENT,
        "process_image" => PROCESS_IMAGE,
        "search_mcp_tools" => SEARCH_MCP,
        "forecast_data_audit" => FORECAST_DATA_AUDIT,
        "forecast_analyze" => FORECAST_ANALYZE,
        "forecast_read" => FORECAST_READ,
        "forecast_models" => FORECAST_MODELS,
        "forecast_backtest" => FORECAST_BACKTEST,
        "forecast_compare_models" => FORECAST_COMPARE_MODELS,
        _ => return None,
    })
}
