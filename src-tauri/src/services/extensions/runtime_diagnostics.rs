use super::protocol::HostDiagnostic;
use super::types::ExtensionDiagnostic;

pub fn from_host(
    extension_id: String,
    diagnostic: HostDiagnostic,
) -> Result<ExtensionDiagnostic, String> {
    const STAGES: &[&str] = &["import", "activate", "register"];
    const CODES: &[&str] = &[
        "module_not_found",
        "syntax_error",
        "activation_failed",
        "registration_failed",
        "import_failed",
    ];
    if !STAGES.contains(&diagnostic.stage.as_str())
        || !CODES.contains(&diagnostic.code.as_str())
        || diagnostic
            .file
            .as_ref()
            .is_some_and(|file| file.len() > 128 || file.contains('/') || file.contains('\\'))
        || diagnostic.line.is_some_and(|line| line > 10_000_000)
        || diagnostic.column.is_some_and(|column| column > 10_000_000)
    {
        return Err("Diagnostic d'extension invalide.".to_string());
    }
    Ok(ExtensionDiagnostic {
        extension_id,
        stage: diagnostic.stage,
        code: diagnostic.code,
        file: diagnostic.file,
        line: diagnostic.line,
        column: diagnostic.column,
    })
}
