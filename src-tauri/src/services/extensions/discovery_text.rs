use std::collections::HashSet;

const GENERIC_AUTO_TERMS: &[&str] = &[
    "create",
    "make",
    "read",
    "write",
    "edit",
    "update",
    "inspect",
    "patch",
    "merge",
    "file",
    "files",
    "tool",
    "tools",
    "creer",
    "faire",
    "fais",
    "lire",
    "ecrire",
    "modifier",
    "fichier",
    "fichiers",
    "outil",
    "crear",
    "leer",
    "escribir",
    "editar",
    "archivo",
    "erstellen",
    "lesen",
    "schreiben",
    "bearbeiten",
    "datei",
    "creare",
    "leggere",
    "scrivere",
    "modificare",
];

pub(super) fn auto_query(value: &str) -> String {
    terms(value)
        .into_iter()
        .filter(|term| !GENERIC_AUTO_TERMS.contains(&term.as_str()))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn terms(value: &str) -> Vec<String> {
    normalize(value)
        .split_whitespace()
        .filter(|term| term.chars().count() >= 3)
        .map(str::to_string)
        .collect::<HashSet<_>>()
        .into_iter()
        .collect()
}

pub(super) fn normalize(value: &str) -> String {
    value
        .chars()
        .flat_map(char::to_lowercase)
        .map(fold_latin)
        .map(|character| {
            if character.is_alphanumeric() {
                character
            } else {
                ' '
            }
        })
        .collect()
}

fn fold_latin(character: char) -> char {
    match character {
        'à' | 'á' | 'â' | 'ä' | 'ã' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'ö' | 'õ' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        other => other,
    }
}

pub(super) fn clip_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}
