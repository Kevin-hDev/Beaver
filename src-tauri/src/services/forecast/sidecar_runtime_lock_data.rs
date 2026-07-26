macro_rules! platform_locks {
    ($platform:literal) => {
        pub(super) fn lock_for_runtime(runtime: &str) -> &'static str {
            match runtime {
                "chronos" => lock!("chronos", $platform),
                "timesfm" => lock!("timesfm", $platform),
                "toto" => lock!("toto", $platform),
                "moirai" => lock!("moirai", $platform),
                "flowstate" => lock!("flowstate", $platform),
                "tabpfn" => lock!("tabpfn", $platform),
                "tirex" => lock!("tirex", $platform),
                "kairos" => lock!("kairos", $platform),
                "sundial" => lock!("sundial", $platform),
                _ => "",
            }
        }
    };
}

pub(super) fn source_for_runtime(runtime: &str) -> Option<&'static str> {
    (runtime == "kairos").then_some(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/resources/forecast-sidecar/runtime-locks/kairos.source.lock"
    )))
}

macro_rules! lock {
    ($runtime:literal, $platform:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/forecast-sidecar/runtime-locks/",
            $runtime,
            ".",
            $platform,
            ".lock"
        ))
    };
}

#[cfg(target_os = "macos")]
platform_locks!("macos");

#[cfg(target_os = "linux")]
platform_locks!("linux");

#[cfg(target_os = "windows")]
platform_locks!("windows");
