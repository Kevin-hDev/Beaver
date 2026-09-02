use super::types_tools::ToolResult;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DiscoveryStatus {
    Loaded,
    AlreadyAvailable,
    NoTools,
    ProviderLimit,
    DiscoveryLimit,
    Unavailable,
}

impl DiscoveryStatus {
    pub(super) fn diagnostic_reason(self) -> super::stream_diagnostics::ExtensionDiagnosticReason {
        match self {
            Self::ProviderLimit => {
                super::stream_diagnostics::ExtensionDiagnosticReason::ProviderCapacity
            }
            Self::DiscoveryLimit => {
                super::stream_diagnostics::ExtensionDiagnosticReason::GlobalCapacity
            }
            Self::Loaded | Self::AlreadyAvailable | Self::NoTools | Self::Unavailable => {
                super::stream_diagnostics::ExtensionDiagnosticReason::DiscoveryResult
            }
        }
    }
}

pub(super) struct DiscoveryLine {
    pub plugin_id: String,
    pub plugin_name: String,
    pub status: DiscoveryStatus,
}

pub(super) fn discovery_result(lines: Vec<DiscoveryLine>) -> ToolResult {
    let incomplete = lines.iter().any(|line| {
        matches!(
            line.status,
            DiscoveryStatus::ProviderLimit
                | DiscoveryStatus::DiscoveryLimit
                | DiscoveryStatus::Unavailable
        )
    });
    let output = render(lines);
    if incomplete {
        ToolResult::partial(
            output,
            ["Certains outils correspondants n'ont pas pu être chargés."],
        )
    } else {
        ToolResult::ok(output)
    }
}

fn render(lines: Vec<DiscoveryLine>) -> String {
    lines
        .into_iter()
        .map(|line| match line.status {
            DiscoveryStatus::Loaded => {
                format!("- {} : outils chargés pour le prochain tour.", line.plugin_name)
            }
            DiscoveryStatus::AlreadyAvailable => {
                format!("- {} : outils déjà disponibles.", line.plugin_name)
            }
            DiscoveryStatus::NoTools => format!(
                "- {} : plugin actif, sans outil appelable.",
                line.plugin_name
            ),
            DiscoveryStatus::ProviderLimit => format!(
                "- {} : non chargé, car le plafond d'outils du fournisseur serait dépassé.",
                line.plugin_name
            ),
            DiscoveryStatus::DiscoveryLimit => format!(
                "- {} : non chargé, car la limite de plugins découverts pour cette session est atteinte.",
                line.plugin_name
            ),
            DiscoveryStatus::Unavailable => format!(
                "- {} : outils indisponibles dans cette requête.",
                line.plugin_name
            ),
        })
        .collect::<Vec<_>>()
        .join("\n")
}
