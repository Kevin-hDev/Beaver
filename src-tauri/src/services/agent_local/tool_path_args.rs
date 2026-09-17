use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PathUse {
    Read,
    Write,
    WorkingDirectory,
}

#[derive(Clone, Copy)]
pub struct PathField {
    pub name: &'static str,
    pub usage: PathUse,
}

const fn field(name: &'static str, usage: PathUse) -> PathField {
    PathField { name, usage }
}

const WORKDIR: &[PathField] = &[field("workdir", PathUse::WorkingDirectory)];
const PATH_READ: &[PathField] = &[field("path", PathUse::Read)];
const PATH_WRITE: &[PathField] = &[field("path", PathUse::Write)];
const IMAGE_PATHS: &[PathField] = &[
    field("input_path", PathUse::Read),
    field("output_path", PathUse::Write),
];
const FORECAST_PATH: &[PathField] = &[field("file_path", PathUse::Read)];

pub fn fields(tool_name: &str) -> &'static [PathField] {
    match tool_name {
        "bash" => WORKDIR,
        "read_file" | "list_dir" | "grep" | "glob" | "read_spreadsheet" | "read_document" => {
            PATH_READ
        }
        "write_file" | "edit_file" | "write_spreadsheet" | "write_document" => PATH_WRITE,
        "transform_image" => IMAGE_PATHS,
        "forecast_data_audit" | "forecast_run" => FORECAST_PATH,
        _ => &[],
    }
}

pub fn first_value<'a>(
    tool_name: &str,
    usage: PathUse,
    args: &'a Value,
) -> Option<&'a str> {
    value(tool_name, usage, args).and_then(Value::as_str)
}

pub fn value<'a>(tool_name: &str, usage: PathUse, args: &'a Value) -> Option<&'a Value> {
    fields(tool_name)
        .iter()
        .find(|field| field.usage == usage)
        .and_then(|field| args.get(field.name))
}

pub fn primary_value<'a>(tool_name: &str, args: &'a Value) -> Option<&'a str> {
    first_value(tool_name, PathUse::Read, args)
        .or_else(|| first_value(tool_name, PathUse::Write, args))
}
