use serde_json::Value;
use std::path::{Path, PathBuf};

/// Résout un chemin relatif/absolu/tilde en chemin absolu.
pub fn resolve_path(path: &str, working_dir: &Path) -> PathBuf {
    if let Some(stripped) = path.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(path.strip_prefix("~/").unwrap_or(stripped));
        }
    }
    let raw = Path::new(path);
    if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        working_dir.join(raw)
    }
}

pub fn try_value_as_u32(val: &Value, field: &str) -> Result<u32, String> {
    if let Some(n) = val.as_u64() {
        return u32::try_from(n).map_err(|_| format!("{field} trop grand"));
    }
    if let Some(n) = val.as_i64() {
        return u32::try_from(n).map_err(|_| format!("{field} invalide"));
    }
    val.as_str()
        .ok_or_else(|| format!("{field} invalide"))?
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{field} invalide"))
}

pub fn try_value_as_u16(val: &Value, field: &str) -> Result<u16, String> {
    let n = try_value_as_u32(val, field)?;
    u16::try_from(n).map_err(|_| format!("{field} trop grand"))
}

/// Extrait un f64 d'une Value, tolère string ou number.
pub fn value_as_f64(val: &Value) -> Option<f64> {
    if let Some(n) = val.as_f64() {
        return Some(n);
    }
    val.as_str().and_then(|s| s.trim().parse::<f64>().ok())
}

/// Traduit les noms de fonctions Excel français → anglais.
const FR_FORMULAS: &[(&str, &str)] = &[
    ("SOMME", "SUM"),
    ("MOYENNE", "AVERAGE"),
    ("NB", "COUNT"),
    ("NBVAL", "COUNTA"),
    ("NB.SI", "COUNTIF"),
    ("NB.VIDE", "COUNTBLANK"),
    ("SOMME.SI", "SUMIF"),
    ("SOMME.SI.ENS", "SUMIFS"),
    ("MAX", "MAX"),
    ("MIN", "MIN"),
    ("SI", "IF"),
    ("ET", "AND"),
    ("OU", "OR"),
    ("NON", "NOT"),
    ("RECHERCHEV", "VLOOKUP"),
    ("RECHERCHEH", "HLOOKUP"),
    ("INDEX", "INDEX"),
    ("EQUIV", "MATCH"),
    ("CONCATENER", "CONCATENATE"),
    ("GAUCHE", "LEFT"),
    ("DROITE", "RIGHT"),
    ("MAJUSCULE", "UPPER"),
    ("MINUSCULE", "LOWER"),
    ("NBCAR", "LEN"),
    ("ARRONDI", "ROUND"),
    ("ABS", "ABS"),
    ("RACINE", "SQRT"),
    ("PUISSANCE", "POWER"),
    ("AUJOURDHUI", "TODAY"),
    ("MAINTENANT", "NOW"),
    ("ANNEE", "YEAR"),
    ("MOIS", "MONTH"),
    ("JOUR", "DAY"),
    ("VRAI", "TRUE"),
    ("FAUX", "FALSE"),
    ("STXT", "MID"),
    ("CHERCHE", "SEARCH"),
    ("TROUVE", "FIND"),
    ("ENT", "INT"),
    ("MOD", "MOD"),
];

/// Normalise une formule : traduit le français, remplace `;` par `,`.
pub fn normalize_formula(formula: &str) -> String {
    let mut result = String::with_capacity(formula.len());
    let mut in_string = false;
    let mut index = 0usize;
    while index < formula.len() {
        let rest = &formula[index..];
        let Some(ch) = rest.chars().next() else {
            break;
        };
        if ch == '"' {
            in_string = !in_string;
            result.push(ch);
            index += ch.len_utf8();
            continue;
        }
        if !in_string {
            let mut replaced = false;
            for &(fr, en) in FR_FORMULAS {
                let pattern = format!("{fr}(");
                if rest.starts_with(&pattern) {
                    result.push_str(en);
                    result.push('(');
                    index += pattern.len();
                    replaced = true;
                    break;
                }
            }
            if replaced {
                continue;
            }
            if ch == ';' {
                result.push(',');
                index += 1;
                continue;
            }
        }
        result.push(ch);
        index += ch.len_utf8();
    }
    result
}

/// Valide et normalise une couleur hex (ex: "FF0000", "#FF0000", "ff0000").
/// Retourne la forme canonique RRGGBB (6 chiffres, majuscules, sans '#').
/// Les backends ajoutent le préfixe alpha si nécessaire (rust_xlsxwriter = RRGGBB,
/// umya = "FF" + RRGGBB).
///
/// Une valeur vide/null est traitée comme "absente" et retourne `None` (pas d'erreur).
/// Une valeur non-vide mais invalide échoue fermé.
pub fn validate_color_hex(val: &Value, field: &str) -> Result<Option<String>, String> {
    // Vide/null → absent (ignoré silencieusement par le caller).
    if val.is_null() {
        return Ok(None);
    }
    let raw = match val.as_str() {
        Some(s) => s,
        None => return Err(format!("{field} doit être une chaîne")),
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let cleaned = trimmed.trim_start_matches('#');
    let is_hex = cleaned.len() == 6 && cleaned.chars().all(|c| c.is_ascii_hexdigit());
    if !is_hex {
        return Err(format!(
            "{field} couleur invalide (format attendu RRGGBB, ex: 'FF0000')"
        ));
    }
    Ok(Some(cleaned.to_ascii_uppercase()))
}

/// Mappe un nom de style de bordure vers l'identifiant OOXML.
pub fn border_style_name(val: &Value) -> Result<&'static str, String> {
    match val
        .as_str()
        .map(|s| s.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("thin") => Ok("thin"),
        Some("medium") => Ok("medium"),
        Some("thick") => Ok("thick"),
        _ => Err("border_style invalide (valeurs: thin|medium|thick)".to_string()),
    }
}

/// Coerce une Value en Vec<Value> pour les arrays de `values` dans set_row.
pub fn coerce_values_array(val: &Value) -> Option<Vec<Value>> {
    if let Some(arr) = val.as_array() {
        return Some(arr.clone());
    }
    if let Some(s) = val.as_str() {
        if let Ok(parsed) = serde_json::from_str::<Value>(s.trim()) {
            if let Some(arr) = parsed.as_array() {
                return Some(arr.clone());
            }
        }
    }
    None
}
