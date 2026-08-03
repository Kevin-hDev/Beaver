const GENERATED_DIRECTORIES: &[&str] = &[
    ".git",
    "node_modules",
    "target",
    "dist",
    "build",
    ".next",
    ".nuxt",
    ".turbo",
    ".cache",
    ".venv",
    "venv",
    "__pycache__",
    "vendor",
    "Pods",
    ".gradle",
    "out",
    "coverage",
];

pub fn is_generated_component(name: &std::ffi::OsStr) -> bool {
    GENERATED_DIRECTORIES
        .iter()
        .any(|generated| name == std::ffi::OsStr::new(generated))
}
