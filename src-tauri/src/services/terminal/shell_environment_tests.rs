use super::shell_environment::{apply, ShellEnvironment};
#[cfg(unix)]
use portable_pty::CommandBuilder;
#[cfg(unix)]
use std::ffi::OsStr;

#[cfg(unix)]
const REFUSED: [&str; 6] = [
    "NO_COLOR",
    "NODE_DISABLE_COLORS",
    "FORCE_COLOR",
    "COLOR",
    "CLICOLOR",
    "CLICOLOR_FORCE",
];

#[cfg(unix)]
fn launched_without_color() -> CommandBuilder {
    let mut command = CommandBuilder::new("/bin/sh");
    for name in REFUSED {
        command.env(name, "0");
    }
    command.env("NO_COLOR", "1");
    command.env("TERM", "dumb");
    command
}

/// Le défaut réel : Beaver lancé depuis un agent héritait de `NO_COLOR=1` et le
/// transmettait au shell, qui rendait alors toute sa sortie en gris.
#[test]
#[cfg(unix)]
fn le_terminal_declare_ses_couleurs_a_la_place_de_son_lanceur() {
    let mut command = launched_without_color();

    apply(&mut command);

    for name in REFUSED {
        assert_eq!(command.get_env(name), None, "{name} a survécu");
    }
    assert_eq!(command.get_env("TERM"), Some(OsStr::new("xterm-256color")));
    assert_eq!(command.get_env("COLORTERM"), Some(OsStr::new("truecolor")));
}

/// Seules les directives de couleur sont retirées : le reste de l'environnement
/// de l'utilisateur doit atteindre son shell intact.
#[test]
#[cfg(unix)]
fn le_reste_de_l_environnement_traverse_intact() {
    let mut command = CommandBuilder::new("/bin/sh");
    command.env("EDITOR", "/usr/bin/vim");
    command.env("LANG", "fr_FR.UTF-8");

    apply(&mut command);

    assert_eq!(command.get_env("EDITOR"), Some(OsStr::new("/usr/bin/vim")));
    assert_eq!(command.get_env("LANG"), Some(OsStr::new("fr_FR.UTF-8")));
}

/// L'ordre compte : retirer après avoir posé effacerait ce qui vient d'être
/// déclaré.
#[test]
fn ce_qui_est_declare_survit_a_ce_qui_est_retire() {
    struct Journal(Vec<String>);

    impl ShellEnvironment for Journal {
        fn set(&mut self, name: &str, value: &str) {
            self.0.push(format!("pose {name}={value}"));
        }

        fn unset(&mut self, name: &str) {
            self.0.push(format!("retire {name}"));
        }
    }

    let mut journal = Journal(Vec::new());

    apply(&mut journal);

    let dernier_retrait = journal
        .0
        .iter()
        .rposition(|ligne| ligne.starts_with("retire"))
        .expect("au moins un retrait");
    let premiere_pose = journal
        .0
        .iter()
        .position(|ligne| ligne.starts_with("pose"))
        .expect("au moins une pose");
    assert!(dernier_retrait < premiere_pose);
}
