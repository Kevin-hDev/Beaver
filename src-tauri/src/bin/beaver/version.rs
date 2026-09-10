use crate::output::Out;

pub fn run(out: &Out) -> i32 {
    out.line(&format!("Beaver {}", env!("CARGO_PKG_VERSION")));
    if let Ok(executable) = std::env::current_exe() {
        out.line(&format!(
            "{} {}",
            out.t("Programme :", "Binary:"),
            executable.display()
        ));
    }
    0
}
