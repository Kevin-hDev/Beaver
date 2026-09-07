//! Autorité unique de l'environnement transmis au shell du terminal.
//!
//! Beaver hérite de l'environnement du processus qui l'a lancé. Quand ce
//! lanceur est lui-même un outil sans couleur — un agent, une intégration
//! continue, un shell dont la sortie est redirigée — il y laisse `NO_COLOR=1`
//! et `TERM=dumb`, et Beaver les relaie ensuite à tout ce que l'utilisateur
//! tape dans son terminal : vite, cargo et la CLI tauri se taisent, et la
//! sortie est uniformément grise.
//!
//! Ces directives décrivent la sortie du lanceur, pas celle de ce terminal, qui
//! affiche 256 couleurs quoi qu'il arrive. Le terminal ne les relaie donc pas :
//! il déclare lui-même ce qu'il sait afficher.
//!
//! Les deux plateformes passent par ici. `TERM` était déjà imposé de chaque
//! côté séparément : une seule liste, appliquée aux deux, évite qu'elles
//! divergent.

use portable_pty::CommandBuilder;

/// Ce que le terminal déclare, quel que soit son lanceur.
const DECLARED: [(&str, &str); 2] = [("TERM", "xterm-256color"), ("COLORTERM", "truecolor")];

/// Les directives de couleur du lanceur, qui ne concernent pas ce terminal.
/// `NO_COLOR` vient de la convention no-color.org, `FORCE_COLOR` de l'écosystème
/// Node, `COLOR` est posée par npm, `CLICOLOR` et `CLICOLOR_FORCE` viennent des
/// outils BSD — dont le `ls` de macOS.
const DROPPED: [&str; 5] = [
    "NO_COLOR",
    "FORCE_COLOR",
    "COLOR",
    "CLICOLOR",
    "CLICOLOR_FORCE",
];

/// Ce qu'un lanceur de shell doit savoir faire pour recevoir cet environnement.
pub(crate) trait ShellEnvironment {
    fn set(&mut self, name: &str, value: &str);
    fn unset(&mut self, name: &str);
}

/// Impose l'environnement du terminal à la commande qui lancera le shell.
pub(crate) fn apply(command: &mut impl ShellEnvironment) {
    for name in DROPPED {
        command.unset(name);
    }
    for (name, value) in DECLARED {
        command.set(name, value);
    }
}

impl ShellEnvironment for CommandBuilder {
    fn set(&mut self, name: &str, value: &str) {
        self.env(name, value);
    }

    fn unset(&mut self, name: &str) {
        self.env_remove(name);
    }
}

#[cfg(windows)]
impl ShellEnvironment for windows_spawn::Command {
    fn set(&mut self, name: &str, value: &str) {
        self.env(name, value);
    }

    fn unset(&mut self, name: &str) {
        self.env_remove(name);
    }
}
