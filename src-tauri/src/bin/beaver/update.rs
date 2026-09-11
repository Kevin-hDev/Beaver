// La vérification et le téléchargement signé sont indépendants de Tauri :
// check_app_update_detailed ne lit aucun AppHandle ni état global, et
// download_verified_update valide l’URL, le manifeste, la taille et SHA-256.
// Seuls les chemins du binaire principal et des ressources sont fournis au
// worker, qui attend le PID de cette CLI pendant 120 s avant d’appliquer.
use crate::output::Out;

pub fn run(out: &Out, args: &[String]) -> i32 {
    if !args.is_empty() {
        out.line(out.t("Usage : beaver update", "Usage: beaver update"));
        return 2;
    }
    let runtime = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => return unavailable(out, "runtime"),
    };
    let check = match runtime.block_on(cl_go_dash_lib::cli_support::check_app_update_detailed()) {
        Ok(check) => check,
        Err(error) => return unavailable(out, &error),
    };
    let update = match check {
        cl_go_dash_lib::cli_support::UpdateCheck::UpToDate => {
            out.line(out.t("À jour.", "Up to date."));
            return 0;
        }
        cl_go_dash_lib::cli_support::UpdateCheck::Unknown(reason) => {
            return unavailable(out, reason)
        }
        cl_go_dash_lib::cli_support::UpdateCheck::Available(update) => update,
    };
    out.line(&format!(
        "{} {}",
        out.t("Mise à jour disponible :", "Update available:"),
        update.version
    ));
    if crate::app_detect::app_is_running() {
        out.line(out.t(
            "Fermez Beaver puis relancez beaver update pour l’appliquer.",
            "Close Beaver, then run beaver update again to apply it.",
        ));
        return 1;
    }
    match crate::app_detect::confirm_while_closed(
        || {
            crate::output::confirmation(
                out,
                "Télécharger et installer cette mise à jour ? [o/N]",
                "Download and install this update? [y/N]",
            )
        },
        crate::app_detect::app_is_running,
    ) {
        crate::app_detect::ClosedConfirmation::Cancelled => {
            out.line(out.t("Mise à jour annulée.", "Update cancelled."));
            return 0;
        }
        crate::app_detect::ClosedConfirmation::AppOpened => {
            out.line(out.t(
                "Fermez Beaver puis relancez beaver update pour l’appliquer.",
                "Close Beaver, then run beaver update again to apply it.",
            ));
            return 1;
        }
        crate::app_detect::ClosedConfirmation::Confirmed => {}
    }
    out.line(out.t("Téléchargement…", "Downloading…"));
    match runtime.block_on(cl_go_dash_lib::cli_support::download_and_launch_app_update(
        update,
    )) {
        Ok(()) => {
            out.line(out.t(
                "La mise à jour se poursuit en arrière-plan. Relancez beaver --version dans une minute pour vérifier.",
                "The update is continuing in the background. Run beaver --version again in one minute to verify.",
            ));
            0
        }
        Err(error) => failure(
            out,
            &error,
            "Mise à jour impossible. Réessayez plus tard.",
            "Unable to update. Try again later.",
        ),
    }
}

fn unavailable(out: &Out, detail: &str) -> i32 {
    failure(
        out,
        detail,
        "Vérification impossible. Réessayez plus tard.",
        "Unable to check for updates. Try again later.",
    )
}

fn failure(out: &Out, detail: &str, fr: &str, en: &str) -> i32 {
    eprintln!("[beaver update] {}", safe_detail(detail));
    out.line(out.t(fr, en));
    1
}

fn safe_detail(detail: &str) -> String {
    // stderr est une frontière terminal : elle reste bornée et sans caractères de contrôle.
    detail
        .chars()
        .take(512)
        .map(|character| {
            if character.is_control() {
                '?'
            } else {
                character
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "update_tests.rs"]
mod tests;
