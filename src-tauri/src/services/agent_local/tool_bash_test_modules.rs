use super::*;

#[cfg(unix)]
#[path = "tool_bash_test_support.rs"]
mod test_support;

#[path = "tool_bash_workdir_tests.rs"]
mod workdir_tests;

#[cfg(unix)]
#[path = "tool_bash_lifecycle_tests.rs"]
mod lifecycle_tests;

#[cfg(unix)]
#[path = "tool_bash_input_tests.rs"]
mod input_tests;

#[cfg(unix)]
#[path = "tool_bash_command_output_tests.rs"]
mod output_tests;

#[cfg(unix)]
#[path = "tool_bash_detached_tests.rs"]
mod detached_tests;
