#![cfg(test)]

pub(crate) const INSTALL_PHASE_ORDER: [&str; 15] = [
    "profile",
    "manifest",
    "download",
    "verify",
    "extract",
    "remove_archives",
    "fingerprint",
    "version",
    "receipt",
    "sync_staging",
    "probe",
    "rename",
    "sync_parent",
    "reinspect",
    "success",
];
