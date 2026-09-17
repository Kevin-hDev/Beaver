// Navigation-only domains preserve the existing Rust paths and behavior.
// Use real submodules only if repeated cross-domain defects justify stricter boundaries.
include!("contract/modules.inc.rs");
include!("installation/modules.inc.rs");
include!("host/modules.inc.rs");
include!("permissions/modules.inc.rs");
include!("events/modules.inc.rs");
include!("storage/modules.inc.rs");
include!("tools/modules.inc.rs");
include!("ui/modules.inc.rs");
include!("contract/api_exports.inc.rs");
