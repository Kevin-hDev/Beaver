mod app_detect;
mod doctor;
mod logs;
mod output;
mod paths_cmd;
mod status;
mod tail;
mod version;

const MAX_ARGS: usize = 64;

fn main() {
    let out = output::Out::new();
    let args = std::env::args()
        .skip(1)
        .take(MAX_ARGS + 1)
        .collect::<Vec<_>>();
    let code = if args.len() > MAX_ARGS {
        out.line(out.t("Trop d'arguments.", "Too many arguments."));
        2
    } else {
        dispatch(&out, &args)
    };
    std::process::exit(code);
}

fn dispatch(out: &output::Out, args: &[String]) -> i32 {
    match args.first().map(String::as_str) {
        Some("--version") | Some("-V") => version::run(out),
        Some("paths") => paths_cmd::run(out),
        Some("status") => status::run(out),
        Some("doctor") => doctor::run(out),
        Some("logs") => logs::run(out, &args[1..]),
        Some("--help") | Some("-h") | None => {
            output::print_help(out);
            0
        }
        Some(other) => {
            output::print_unknown_command(out, other);
            output::print_help(out);
            2
        }
    }
}
