use std::path::Path;

#[cfg(any(test, target_os = "macos"))]
use plist::{Dictionary, Value};

use super::super::MigrationError;

#[cfg(any(test, target_os = "linux"))]
pub(in crate::services::autostart_migration) fn linux_desktop(
    name: &str,
    executable: &Path,
) -> Result<Vec<u8>, MigrationError> {
    let executable = validated_text(executable)?;
    let executable = desktop_exec_quote(executable);
    Ok(format!(
        "[Desktop Entry]\nType=Application\nVersion=1.0\nName={name}\nExec={executable} {}\nStartupNotify=false\nTerminal=false\n",
        crate::app_events::AUTOSTART_ARG
    )
    .into_bytes())
}

#[cfg(any(test, target_os = "macos"))]
pub(in crate::services::autostart_migration) fn macos_launch_agent(
    name: &str,
    executable: &Path,
) -> Result<Vec<u8>, MigrationError> {
    let executable = validated_text(executable)?;
    let mut document = Dictionary::new();
    document.insert("Label".to_string(), Value::String(name.to_string()));
    document.insert(
        "ProgramArguments".to_string(),
        Value::Array(vec![
            Value::String(executable.to_string()),
            Value::String(crate::app_events::AUTOSTART_ARG.to_string()),
        ]),
    );
    document.insert("RunAtLoad".to_string(), Value::Boolean(true));
    let mut output = Vec::new();
    Value::Dictionary(document)
        .to_writer_xml(&mut output)
        .map_err(|_| MigrationError::Setup)?;
    Ok(output)
}

fn validated_text(path: &Path) -> Result<&str, MigrationError> {
    let value = path.to_str().ok_or(MigrationError::Setup)?;
    (!value.chars().any(char::is_control))
        .then_some(value)
        .ok_or(MigrationError::Setup)
}

#[cfg(any(test, target_os = "linux"))]
fn desktop_exec_quote(value: &str) -> String {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for character in value.chars() {
        match character {
            '%' => output.push_str("%%"),
            '"' | '`' | '$' | '\\' => {
                output.push('\\');
                output.push(character);
            }
            _ => output.push(character),
        }
    }
    output.push('"');
    output
}
