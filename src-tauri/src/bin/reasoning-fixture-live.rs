#[cfg(debug_assertions)]
fn main() {
    if !cl_go_dash_lib::run_live_reasoning_fixtures() {
        std::process::exit(1);
    }
}

#[cfg(not(debug_assertions))]
fn main() {
    eprintln!("debug-only fixture runner");
    std::process::exit(1);
}
