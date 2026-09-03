use super::runtime::ExtensionRuntime;
use super::types::{ExtensionDiagnostic, ExtensionHostStatus, MAX_RUNTIME_DIAGNOSTICS};

impl ExtensionRuntime {
    pub(super) fn record_ui_mount_failure(
        &self,
        extension_id: &str,
        contribution_id: &str,
    ) -> Result<(), String> {
        if !self
            .ui_catalog
            .contains_contribution(extension_id, contribution_id)?
        {
            return Err(super::ui_catalog::denied());
        }
        let mut status = self
            .status
            .write()
            .map_err(|_| super::ui_catalog::unavailable())?;
        append_mount_failure(&mut status, extension_id);
        Ok(())
    }
}

fn append_mount_failure(status: &mut ExtensionHostStatus, extension_id: &str) {
    if status.diagnostics.iter().any(|diagnostic| {
        diagnostic.extension_id == extension_id
            && diagnostic.code == super::ui_contract::UI_DIAGNOSTIC_UI_MOUNT_FAILED
    }) {
        return;
    }
    if status.diagnostics.len() >= MAX_RUNTIME_DIAGNOSTICS {
        status.diagnostics.remove(0);
    }
    status.diagnostics.push(ExtensionDiagnostic {
        extension_id: extension_id.to_string(),
        stage: super::ui_contract::UI_LOADING_STAGE_MOUNT.to_string(),
        code: super::ui_contract::UI_DIAGNOSTIC_UI_MOUNT_FAILED.to_string(),
        file: None,
        line: None,
        column: None,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mount_failures_are_deduplicated_and_bounded() {
        let mut status = ExtensionHostStatus::default();
        for index in 0..MAX_RUNTIME_DIAGNOSTICS {
            status.diagnostics.push(ExtensionDiagnostic {
                extension_id: format!("com.example.{index}"),
                stage: "register".to_string(),
                code: "load_failed".to_string(),
                file: None,
                line: None,
                column: None,
            });
        }

        append_mount_failure(&mut status, "com.example.failed");
        append_mount_failure(&mut status, "com.example.failed");

        assert_eq!(status.diagnostics.len(), MAX_RUNTIME_DIAGNOSTICS);
        assert_eq!(
            status
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.extension_id == "com.example.failed")
                .count(),
            1
        );
        let diagnostic = status.diagnostics.last().unwrap();
        assert_eq!(
            diagnostic.stage,
            super::super::ui_contract::UI_LOADING_STAGE_MOUNT
        );
        assert_eq!(
            diagnostic.code,
            super::super::ui_contract::UI_DIAGNOSTIC_UI_MOUNT_FAILED
        );
    }
}
