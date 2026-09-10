use std::io::IsTerminal;

const KIB: u64 = 1024;
const MIB: u64 = KIB * 1024;
const GIB: u64 = MIB * 1024;
const MAX_CLI_TOKEN_CHARS: usize = 64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Fr,
    En,
}

pub fn lang_from_locale(locale: &str) -> Lang {
    if locale.to_lowercase().starts_with("fr") {
        Lang::Fr
    } else {
        Lang::En
    }
}

pub fn lang() -> Lang {
    lang_from_locale(&sys_locale::get_locale().unwrap_or_default())
}

pub struct Out {
    pub lang: Lang,
    // Utilisé dès Task 4 par les sorties doctor ; garder la détection TTY centralisée ici.
    #[allow(dead_code)]
    color: bool,
}

impl Out {
    pub fn new() -> Self {
        Self {
            lang: lang(),
            color: std::io::stdout().is_terminal(),
        }
    }

    pub fn t(&self, fr: &'static str, en: &'static str) -> &'static str {
        match self.lang {
            Lang::Fr => fr,
            Lang::En => en,
        }
    }

    #[allow(dead_code)] // Utilisé dès Task 4 par doctor.
    pub fn ok(&self, label: &str) {
        if self.color {
            println!("  \x1b[32m✓\x1b[0m {label}");
        } else {
            println!("  ✓ {label}");
        }
    }

    #[allow(dead_code)] // Utilisé dès Task 4 par doctor.
    pub fn fail(&self, label: &str, advice: &str) {
        if self.color {
            println!("  \x1b[31m✗\x1b[0m {label}");
        } else {
            println!("  ✗ {label}");
        }
        println!("    → {advice}");
    }

    pub fn line(&self, text: &str) {
        println!("{text}");
    }
}

#[allow(dead_code)] // Utilisé dès Task 3 par status.
pub fn format_size(bytes: u64) -> String {
    format_size_with(bytes, lang())
}

pub fn format_size_with(bytes: u64, lang: Lang) -> String {
    let (value, fr_unit, en_unit, decimals) = if bytes < KIB {
        (bytes as f64, "o", "B", 0)
    } else if bytes < MIB {
        (bytes as f64 / KIB as f64, "Ko", "KB", 0)
    } else if bytes < GIB {
        (bytes as f64 / MIB as f64, "Mo", "MB", 1)
    } else {
        (bytes as f64 / GIB as f64, "Go", "GB", 1)
    };
    let number = match decimals {
        0 => format!("{value:.0}"),
        _ => format!("{value:.1}"),
    };
    match lang {
        Lang::Fr => format!("{} {fr_unit}", number.replace('.', ",")),
        Lang::En => format!("{number} {en_unit}"),
    }
}

pub fn safe_cli_token(value: &str) -> &str {
    if !value.is_empty()
        && value.chars().count() <= MAX_CLI_TOKEN_CHARS
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        value
    } else {
        "?"
    }
}

pub fn print_help(out: &Out) {
    out.line(out.t("Usage : beaver <commande>", "Usage: beaver <command>"));
    for (command, fr, en, ready) in [
        ("--version", "version installée", "installed version", true),
        ("paths", "chemins des données", "data paths", false),
        ("status", "état local", "local status", false),
        ("doctor", "diagnostic local", "local diagnostics", false),
        ("logs", "lecture des journaux", "read logs", false),
        ("update", "mise à jour", "update", false),
        ("cleanup", "nettoyage sûr", "safe cleanup", false),
    ] {
        let suffix = if ready {
            ""
        } else {
            out.t(" (à venir)", " (coming soon)")
        };
        out.line(&format!("  {command:<10} {}{suffix}", out.t(fr, en)));
    }
}

pub fn print_unknown_command(out: &Out, command: &str) {
    out.line(&format!(
        "{} `{}`",
        out.t("Commande inconnue :", "Unknown command:"),
        safe_cli_token(command)
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_fr_donne_fr() {
        assert_eq!(lang_from_locale("fr-FR"), Lang::Fr);
        assert_eq!(lang_from_locale("fr"), Lang::Fr);
    }

    #[test]
    fn locale_autre_donne_en() {
        assert_eq!(lang_from_locale("en-US"), Lang::En);
        assert_eq!(lang_from_locale(""), Lang::En);
    }

    #[test]
    fn format_size_lisible() {
        assert_eq!(format_size_with(1_500_000_000, Lang::Fr), "1,4 Go");
        assert_eq!(format_size_with(1_500_000_000, Lang::En), "1.4 GB");
        assert_eq!(format_size_with(512, Lang::Fr), "512 o");
    }

    #[test]
    fn argument_terminal_est_borne_et_sans_controle() {
        assert_eq!(safe_cli_token("logs\u{1b}[31m"), "?");
        assert_eq!(safe_cli_token(&"a".repeat(65)), "?");
        assert_eq!(safe_cli_token("status"), "status");
    }
}
