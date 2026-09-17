// Navigation-only domains preserve the existing Rust paths and behavior.
// Use real submodules only if repeated cross-domain defects justify stricter boundaries.
include!("execution/modules.inc.rs");
include!("conversations/modules.inc.rs");
include!("context/modules.inc.rs");
include!("tools/modules.inc.rs");
include!("permissions/modules.inc.rs");
include!("subagents/modules.inc.rs");
include!("prompts/modules.inc.rs");
include!("diagnostics/modules.inc.rs");
include!("extensions/modules.inc.rs");
