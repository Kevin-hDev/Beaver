use std::path::Path;

const TEXT_EXTENSIONS: &[&str] = &[
    "md", "txt", "ts", "tsx", "js", "jsx", "json", "yaml", "yml", "toml", "rs", "py", "sh", "css",
    "html", "xml", "csv", "sql", "env", "cfg", "conf", "ini", "log", "svelte", "vue",
];

pub(super) fn extension(name: &str) -> Option<String> {
    Path::new(name)
        .extension()?
        .to_str()
        .map(str::to_ascii_lowercase)
}

pub(super) fn is_text_name(name: &str) -> bool {
    extension(name).as_deref().is_some_and(is_text_extension)
}

pub(super) fn is_text_extension(extension: &str) -> bool {
    TEXT_EXTENSIONS.contains(&extension)
}

pub(super) fn declared_text_type(extension: &str, declared: &str) -> bool {
    if declared.eq_ignore_ascii_case(extension) || declared.eq_ignore_ascii_case("text/plain") {
        return true;
    }
    let declared = declared.to_ascii_lowercase();
    match extension {
        "md" => declared == "text/markdown",
        "ts" | "tsx" => matches!(
            declared.as_str(),
            "text/typescript" | "application/typescript"
        ),
        "js" | "jsx" => matches!(
            declared.as_str(),
            "text/javascript" | "application/javascript"
        ),
        "json" => matches!(declared.as_str(), "application/json" | "text/json"),
        "yaml" | "yml" => matches!(declared.as_str(), "application/yaml" | "text/yaml"),
        "toml" => declared == "application/toml",
        "py" => declared == "text/x-python",
        "sh" => matches!(declared.as_str(), "text/x-shellscript" | "application/x-sh"),
        "css" => declared == "text/css",
        "html" => declared == "text/html",
        "xml" => matches!(declared.as_str(), "application/xml" | "text/xml"),
        "csv" => declared == "text/csv",
        "sql" => declared == "application/sql",
        _ => false,
    }
}
