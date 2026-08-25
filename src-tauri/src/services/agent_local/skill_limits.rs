pub use crate::services::skill_manifest_policy::MAX_SKILL_MANIFEST_BYTES as MAX_SKILL_CONTENT_BYTES;
pub const MAX_SKILL_SOURCE_NAME_BYTES: usize = 256;
pub const MAX_SKILL_BUNDLE_PATH_BYTES: usize = 4096;

const SOURCE_PREFIX_BYTES: usize = "Skill source: ".len();
const DIRECTORY_PREFIX_BYTES: usize = "\nSkill directory: ".len();
const BODY_SEPARATOR_BYTES: usize = "\n\n".len();

// The manifest and Beaver's generated provenance header need distinct budgets:
// an exact-size valid manifest must remain loadable after bounded metadata is added.
pub const MAX_RESOLVED_SKILL_BYTES: usize = MAX_SKILL_CONTENT_BYTES
    + MAX_SKILL_SOURCE_NAME_BYTES
    + MAX_SKILL_BUNDLE_PATH_BYTES
    + SOURCE_PREFIX_BYTES
    + DIRECTORY_PREFIX_BYTES
    + BODY_SEPARATOR_BYTES;
