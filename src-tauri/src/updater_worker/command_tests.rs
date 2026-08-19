#[cfg(unix)]
use std::ffi::OsString;
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
use super::{run_bounded_output, run_status, CommandSpec};

#[cfg(unix)]
#[test]
fn captures_only_bounded_direct_command_output() {
    let spec = CommandSpec::new("/usr/bin/printf", vec![OsString::from("beaver")]);
    assert_eq!(
        run_bounded_output(&spec, Duration::from_secs(1), 16).unwrap(),
        b"beaver"
    );
    assert!(run_bounded_output(&spec, Duration::from_secs(1), 3).is_err());
}

#[cfg(unix)]
#[test]
fn requires_a_successful_direct_program() {
    assert!(run_status(
        &CommandSpec::new("/usr/bin/true", vec![]),
        Duration::from_secs(1)
    )
    .is_ok());
    assert!(run_status(
        &CommandSpec::new("/usr/bin/false", vec![]),
        Duration::from_secs(1)
    )
    .is_err());
    assert!(run_status(
        &CommandSpec::new("relative", vec![]),
        Duration::from_secs(1)
    )
    .is_err());
}

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::os::windows::io::{AsHandle, AsRawHandle};

#[cfg(windows)]
use super::{spawn_background, CommandSpec};

/// Le helper de mise a jour relance Beaver puis disparait. Ce test reproduit
/// cette chaine avec trois roles portes par le meme binaire de test : le role
/// helper se confine dans un Job `kill_on_close`, relance la cible par le vrai
/// chemin de production, puis sort. La cible doit survivre a la fermeture de ce
/// Job, sans quoi l'application disparait a la fin de chaque mise a jour.
#[cfg(windows)]
#[test]
fn the_relaunched_application_survives_the_helper_job_closing() {
    const ROLE: &str = "BEAVER_UPDATE_RELAUNCH_ROLE";
    const PRET: &str = "BEAVER_UPDATE_RELAUNCH_READY";
    const SURVIVANT: &str = "BEAVER_UPDATE_RELAUNCH_SURVIVED";
    const NOM: &str = concat!(
        "updater_worker::command::tests::",
        "the_relaunched_application_survives_the_helper_job_closing",
    );

    match std::env::var(ROLE).ok().as_deref() {
        Some("cible") => {
            let pret = std::env::var_os(PRET).expect("marqueur de demarrage");
            let survivant = std::env::var_os(SURVIVANT).expect("marqueur de survie");
            std::fs::write(pret, b"pret").expect("ecrire le marqueur de demarrage");
            // Le helper sort pendant cette attente : seul un lancement affranchi
            // de son Job permet d'ecrire le second marqueur.
            std::thread::sleep(std::time::Duration::from_secs(2));
            std::fs::write(survivant, b"survivant").expect("ecrire le marqueur de survie");
        }
        Some("helper") => relance_depuis_un_job_destructeur(ROLE, PRET, NOM),
        _ => {
            let dossier = tempfile::tempdir().expect("dossier des marqueurs");
            let pret = dossier.path().join("pret");
            let survivant = dossier.path().join("survivant");
            let binaire = std::env::current_exe().expect("binaire de test");
            let statut = std::process::Command::new(binaire)
                .args(["--exact", NOM, "--nocapture"])
                .env(ROLE, "helper")
                .env(PRET, &pret)
                .env(SURVIVANT, &survivant)
                .status()
                .expect("helper confine");
            assert!(
                statut.success(),
                "le helper confine doit se terminer normalement"
            );

            let echeance = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !survivant.exists() && std::time::Instant::now() < echeance {
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            assert!(
                survivant.exists(),
                "l'application relancee est morte avec le Job du helper",
            );
        }
    }
}

#[cfg(windows)]
fn relance_depuis_un_job_destructeur(role: &str, pret: &str, nom: &str) -> ! {
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    let job = windows_spawn::Job::create().expect("Job du helper");
    // Le Job autorise l'affranchissement, comme celui qui confine reellement la
    // chaine de mise a jour : sans cette autorisation le lancement echouerait au
    // lieu de prouver la survie.
    let mut limites: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limites.BasicLimitInformation.LimitFlags =
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_BREAKAWAY_OK;
    let configure = unsafe {
        SetInformationJobObject(
            job.as_handle().as_raw_handle(),
            JobObjectExtendedLimitInformation,
            std::ptr::addr_of!(limites).cast(),
            u32::try_from(std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())
                .expect("taille des limites"),
        )
    };
    assert_ne!(configure, 0, "le Job du helper doit etre destructeur");
    let confine = unsafe {
        AssignProcessToJobObject(
            job.as_handle().as_raw_handle(),
            windows_sys::Win32::System::Threading::GetCurrentProcess(),
        )
    };
    assert_ne!(confine, 0, "le helper doit entrer dans son propre Job");

    std::env::set_var(role, "cible");
    let binaire = std::env::current_exe().expect("binaire de test");
    let spec = CommandSpec::new(
        binaire,
        vec![
            OsString::from("--exact"),
            OsString::from(nom),
            OsString::from("--nocapture"),
        ],
    );
    let mut enfant = spawn_background(&spec).expect("relance de l'application");

    let marqueur = std::path::PathBuf::from(std::env::var_os(pret).expect("marqueur de demarrage"));
    let echeance = std::time::Instant::now() + std::time::Duration::from_secs(10);
    while !marqueur.exists() && std::time::Instant::now() < echeance {
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
    if !marqueur.exists() {
        let _ = enfant.kill();
        let _ = enfant.wait();
        std::process::exit(3);
    }
    // Sortir sans tuer l'enfant : seule la fermeture du Job decide de son sort.
    std::process::exit(0);
}

/// Un Job qui refuse l'affranchissement doit faire echouer la relance plutot que
/// de lancer une application confinee : une relance confinee mourrait avec le
/// helper tout en signalant une mise a jour reussie.
#[cfg(windows)]
#[test]
fn a_relaunch_that_cannot_break_away_fails_instead_of_staying_confined() {
    const ROLE: &str = "BEAVER_UPDATE_CONFINED_ROLE";
    const DEMARRE: &str = "BEAVER_UPDATE_CONFINED_STARTED";
    const NOM: &str = concat!(
        "updater_worker::command::tests::",
        "a_relaunch_that_cannot_break_away_fails_instead_of_staying_confined",
    );

    match std::env::var(ROLE).ok().as_deref() {
        Some("cible") => {
            let demarre = std::env::var_os(DEMARRE).expect("marqueur de demarrage");
            std::fs::write(demarre, b"demarre").expect("ecrire le marqueur de demarrage");
        }
        Some("helper") => refuse_une_relance_confinee(ROLE, NOM),
        _ => {
            let dossier = tempfile::tempdir().expect("dossier du marqueur");
            let demarre = dossier.path().join("demarre");
            let binaire = std::env::current_exe().expect("binaire de test");
            let statut = std::process::Command::new(binaire)
                .args(["--exact", NOM, "--nocapture"])
                .env(ROLE, "helper")
                .env(DEMARRE, &demarre)
                .status()
                .expect("helper confine");
            assert!(
                statut.success(),
                "le helper doit refuser la relance sans lancer d'application",
            );
            assert!(
                !demarre.exists(),
                "aucune application ne doit demarrer dans un Job sans affranchissement",
            );
        }
    }
}

#[cfg(windows)]
fn refuse_une_relance_confinee(role: &str, nom: &str) -> ! {
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    let job = windows_spawn::Job::create().expect("Job du helper");
    let mut limites: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
    limites.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    let configure = unsafe {
        SetInformationJobObject(
            job.as_handle().as_raw_handle(),
            JobObjectExtendedLimitInformation,
            std::ptr::addr_of!(limites).cast(),
            u32::try_from(std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>())
                .expect("taille des limites"),
        )
    };
    assert_ne!(configure, 0, "le Job du helper doit etre destructeur");
    let confine = unsafe {
        AssignProcessToJobObject(
            job.as_handle().as_raw_handle(),
            windows_sys::Win32::System::Threading::GetCurrentProcess(),
        )
    };
    assert_ne!(confine, 0, "le helper doit entrer dans son propre Job");

    std::env::set_var(role, "cible");
    let binaire = std::env::current_exe().expect("binaire de test");
    let spec = CommandSpec::new(
        binaire,
        vec![
            OsString::from("--exact"),
            OsString::from(nom),
            OsString::from("--nocapture"),
        ],
    );
    match spawn_background(&spec) {
        Ok(mut enfant) => {
            let _ = enfant.kill();
            let _ = enfant.wait();
            std::process::exit(4);
        }
        Err(_) => std::process::exit(0),
    }
}
