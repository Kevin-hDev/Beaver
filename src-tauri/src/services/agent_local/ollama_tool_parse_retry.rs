//! Détection du bug Ollama "tool-call parser crash" (issue #16383).
//!
//! Quand Qwen3 génère un appel d'outil avec une légère erreur de formatage
//! (balises imbriquées dans le désordre, ex. `</parameter></function>`),
//! le parser d'Ollama est trop strict : il crash au lieu de tolérer la dérive.
//!
//! Deux manifestations selon le moment où Ollama détecte l'erreur :
//! - HTTP 500 avec body `{"error":"expected element type <function> but have <parameter>"}`
//!   si le crash survient avant ou pendant la génération
//! - Chunk NDJSON `{"error":"XML syntax error on line 3: element <function> closed by </parameter>"}`
//!   si le crash survient en plein stream
//!
//! Contournement : retenter la requête (le modèle générera probablement un
//! format correct au 2e essai, car le bug est intermittent).

use regex::Regex;
use std::sync::LazyLock;

/// Signature du bug : une erreur XML émise par le parser Ollama qui mentionne
/// au moins une balise tool-call (`<function>` ou `<parameter>`).
/// Ollama formule ce crash de plusieurs façons selon où il détecte l'erreur,
/// d'où l'expression régulière plutôt qu'une correspondance exacte.
static TOOL_PARSE_CRASH_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(element\s+<function>|<parameter>|<function>)")
        .expect("regex tool parse crash")
});

/// Mots-clés d'erreurs XML/parser qui accompagnent les balises tool-call.
/// Au moins un doit être présent en plus d'une balise pour confirmer le bug.
static XML_ERROR_KEYWORDS: &[&str] = &[
    "xml syntax error",
    "expected element type",
    "closed by",
    "but have",
    "element type",
];

/// Nombre maximum de retries pour ce bug précis.
/// Au-delà, on laisse l'erreur remonter (évite une boucle infinie si le
/// modèle produit systématiquement un format cassé).
pub const MAX_PARSER_RETRIES: u32 = 2;

/// Renvoie `true` si le texte correspond au crash du parser tool-call.
///
/// On exige DEUX conditions pour éviter les faux positifs :
/// 1. Une erreur XML/parser (mot-clé caractéristique)
/// 2. Une balise tool-call (`<function>` ou `<parameter>`)
///
/// **Important** : le body d'erreur vient de `resp.text()` (JSON brut de Go),
/// où les `<` et `>` sont échappés en `\u003c` / `\u003e`. On normalise avant
/// de comparer, sinon la détection échoue silencieusement.
pub fn is_tool_parse_crash(text: &str) -> bool {
    let normalized = normalize_json_escapes(text);
    let lower = normalized.to_lowercase();
    let has_tool_tag = TOOL_PARSE_CRASH_RE.is_match(&normalized);
    let has_xml_error = XML_ERROR_KEYWORDS.iter().any(|k| lower.contains(k));
    has_tool_tag && has_xml_error
}

/// Remplace les échappements JSON de Go (`\u003c`, `\u003e`, `\u0026`) par
/// leurs caractères réels. Permet de détecter les balises `<function>` même
/// dans le body brut non décodé renvoyé par `resp.text()`.
fn normalize_json_escapes(text: &str) -> String {
    text.replace("\\u003c", "<")
        .replace("\\u003e", ">")
        .replace("\\u0026", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_http_500_variant() {
        let msg = r#"{"error":"expected element type <function> but have <parameter>"}"#;
        assert!(is_tool_parse_crash(msg));
    }

    #[test]
    fn detects_mid_stream_variant() {
        let msg = "XML syntax error on line 3: element <function> closed by </parameter>";
        assert!(is_tool_parse_crash(msg));
    }

    #[test]
    fn detects_plain_text_http_variant() {
        assert!(is_tool_parse_crash(
            "expected element type <function> but have <parameter>"
        ));
    }

    #[test]
    fn detects_case_insensitive() {
        let msg = "Expected Element Type <FUNCTION> but have <Parameter>";
        assert!(is_tool_parse_crash(msg));
    }

    #[test]
    fn does_not_match_unrelated_500() {
        assert!(!is_tool_parse_crash("internal server error"));
        assert!(!is_tool_parse_crash("model not found"));
        assert!(!is_tool_parse_crash("OOM: out of memory"));
    }

    #[test]
    fn does_not_match_missing_keyword() {
        // Balise tool-call seule sans mot-clé d'erreur XML → pas le bug
        assert!(!is_tool_parse_crash("missing <function> definition"));
    }

    #[test]
    fn does_not_match_xml_error_without_tool_tag() {
        // Erreur XML mais sans balise tool-call → pas le bug
        assert!(!is_tool_parse_crash("XML syntax error: unexpected EOF"));
    }

    #[test]
    fn detects_http500_with_json_escaped_tags() {
        // Body BRUT tel que renvoyé par resp.text() : Go échappe < en \u003c.
        // C'est le cas réel observé en production — sans normalisation, la
        // détection échouait silencieusement et le retry ne se déclenchait pas.
        let body = r#"{"error":"XML syntax error on line 14: element \u003cparameter\u003e closed by \u003c/function\u003e"}"#;
        assert!(is_tool_parse_crash(body));
    }

    #[test]
    fn detects_http500_escaped_variant_2() {
        let body = r#"{"error":"expected element type \u003cfunction\u003e but have \u003cparameter\u003e"}"#;
        assert!(is_tool_parse_crash(body));
    }

    #[test]
    fn detects_mid_stream_with_json_escaped_tags() {
        let body = r#"{"error":"XML syntax error on line 3: element \u003cfunction\u003e closed by \u003c/parameter\u003e"}"#;
        assert!(is_tool_parse_crash(body));
    }
}
