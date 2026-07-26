use super::catalog;

pub(super) const MAX_SOURCES: usize = 48;

#[derive(Clone, Copy)]
pub(super) enum SourceSpec {
    Pypi {
        id: &'static str,
        name: &'static str,
        family: &'static str,
        package: &'static str,
    },
    Github {
        id: &'static str,
        name: &'static str,
        repository: &'static str,
        current: &'static str,
    },
    HuggingFace {
        id: &'static str,
        name: &'static str,
        repository: &'static str,
        reference: &'static str,
        current: &'static str,
    },
}

const PYPI_SOURCES: &[SourceSpec] = &[
    pypi("chronos", "Chronos", "chronos-bolt", "chronos-forecasting"),
    pypi("timesfm", "TimesFM", "timesfm-2-5", "timesfm"),
    pypi("toto", "Toto", "toto-2", "toto-2"),
    pypi("moirai", "Moirai", "moirai-2", "uni2ts"),
    pypi("flowstate", "FlowState", "flowstate", "granite-tsfm"),
    pypi("tabpfn-ts", "TabPFN-TS", "tabpfn-ts", "tabpfn-time-series"),
    pypi("tabpfn", "TabPFN", "tabpfn-ts", "tabpfn"),
    pypi("tirex", "TiRex", "tirex", "tirex-ts"),
];

const ENGINE_SOURCES: &[SourceSpec] = &[SourceSpec::Github {
    id: "kairos-engine",
    name: "Kairos",
    repository: "foundation-model-research/Kairos",
    current: "0322393840ccf6e2bfe9c663f9dcd088a5a7ee07",
}];

const fn pypi(
    id: &'static str,
    name: &'static str,
    family: &'static str,
    package: &'static str,
) -> SourceSpec {
    SourceSpec::Pypi {
        id,
        name,
        family,
        package,
    }
}

pub(super) fn all() -> Result<Vec<SourceSpec>, String> {
    let mut sources = Vec::with_capacity(MAX_SOURCES);
    sources.extend_from_slice(PYPI_SOURCES);
    sources.extend_from_slice(ENGINE_SOURCES);
    for model in catalog::FORECAST_MODELS
        .iter()
        .filter(|model| !model.is_cloud)
    {
        let (Some(repository), Some(current)) = (model.hf_repo, model.hf_revision) else {
            continue;
        };
        let reference = if model.id == "flowstate-r1.1" {
            "r1.1"
        } else {
            "main"
        };
        sources.push(SourceSpec::HuggingFace {
            id: model.id,
            name: model.display_name,
            repository,
            reference,
            current,
        });
        if sources.len() > MAX_SOURCES {
            return Err("Trop de sources Forecast".to_string());
        }
    }
    Ok(sources)
}
