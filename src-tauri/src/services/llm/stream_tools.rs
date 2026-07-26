//! Accumulateur de `tool_calls` arrivant en fragments SSE (format OpenAI-compat).
//!
//! **Quirks gérés** :
//! - OpenAI / Mistral / DeepSeek / Cerebras / OpenRouter : `arguments` arrivent en fragments
//!   JSON fragmentés. On concatène par `index`.
//! - Groq : envoie chaque `tool_call` **complet** en un seul chunk. Même algo fonctionne.
//! - Gemini OpenAI-compat : champ `index` absent — fallback via ordre d'arrivée des `id`.

use serde_json::Value;
use std::collections::HashMap;

const MAX_TOOL_CALLS: usize = 32;
const MAX_TOOL_ARGUMENT_CHARS: usize = 256 * 1024;
const MAX_TOOL_ID_CHARS: usize = 256;
const MAX_TOOL_NAME_CHARS: usize = 128;
const MAX_EXTRA_CONTENT_CHARS: usize = 64 * 1024;

pub type FinalizedToolCalls = (Vec<(String, Value)>, Vec<String>, Vec<Option<Value>>);

#[derive(Debug, Clone, Default)]
pub struct ToolCallAcc {
    pub id: String,
    pub name: String,
    /// Arguments bruts concaténés. Parsés en JSON Value à la finalisation.
    pub arguments: String,
    pub argument_chars: usize,
    pub extra_content: Option<Value>,
    pub exceeded_limit: bool,
}

#[derive(Debug, Default)]
pub struct ToolCallAccumulator {
    /// Map par index (ou index synthétique pour Gemini).
    pub by_index: HashMap<usize, ToolCallAcc>,
    /// Ordre d'insertion (pour préserver l'ordre final).
    pub order: Vec<usize>,
    /// Compteur pour Gemini (index absent).
    pub synthetic_counter: usize,
}

impl ToolCallAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ingère un `delta.tool_calls[]` JSON array.
    pub fn ingest(&mut self, tool_calls: &[Value]) {
        for tc in tool_calls {
            let index = tc["index"].as_u64().map(|v| v as usize).unwrap_or_else(|| {
                // Quirk Gemini : pas d'index → on synthétise un nouveau à chaque `id` non vu.
                let id = tc["id"].as_str().unwrap_or("");
                if !id.is_empty() {
                    // Nouvel id jamais vu → incrémenter
                    let existing = self.by_index.values().find(|a| a.id == id);
                    if existing.is_none() {
                        let i = self.synthetic_counter;
                        self.synthetic_counter += 1;
                        return i;
                    }
                }
                // Par défaut 0 (cas simple non-parallèle)
                0
            });

            if !self.by_index.contains_key(&index) && self.by_index.len() >= MAX_TOOL_CALLS {
                continue;
            }
            let entry = self.by_index.entry(index).or_insert_with(|| {
                self.order.push(index);
                ToolCallAcc::default()
            });

            if let Some(id) = tc["id"].as_str() {
                if id.chars().count() > MAX_TOOL_ID_CHARS {
                    entry.exceeded_limit = true;
                } else if !id.is_empty() && entry.id.is_empty() {
                    entry.id = id.to_string();
                }
            }
            if let Some(name) = tc["function"]["name"].as_str() {
                if name.chars().count() > MAX_TOOL_NAME_CHARS {
                    entry.exceeded_limit = true;
                } else if !name.is_empty() && entry.name.is_empty() {
                    entry.name = name.to_string();
                }
            }
            if let Some(args_frag) = tc["function"]["arguments"].as_str() {
                append_bounded(
                    &mut entry.arguments,
                    &mut entry.argument_chars,
                    args_frag,
                    MAX_TOOL_ARGUMENT_CHARS,
                    &mut entry.exceeded_limit,
                );
            }
            if !tc["extra_content"].is_null() {
                if value_len(&tc["extra_content"]) <= MAX_EXTRA_CONTENT_CHARS {
                    entry.extra_content = Some(tc["extra_content"].clone());
                } else {
                    entry.exceeded_limit = true;
                }
            }
        }
    }

    pub fn has_pending(&self) -> bool {
        !self.order.is_empty()
    }

    /// Finalise : retourne (tool_calls au format Ollama, IDs alignés).
    /// Parse chaque `arguments` string comme JSON Value.
    pub fn finalize(self) -> FinalizedToolCalls {
        // Tri par order d'apparition (préserve l'ordre des parallel calls).
        let mut ordered: Vec<usize> = self.order.clone();
        ordered.dedup();

        let mut tool_calls = Vec::with_capacity(ordered.len());
        let mut ids = Vec::with_capacity(ordered.len());
        let mut extra_content = Vec::with_capacity(ordered.len());

        for idx in ordered {
            let Some(acc) = self.by_index.get(&idx) else {
                continue;
            };
            if acc.name.is_empty() || acc.exceeded_limit {
                continue;
            }
            let args_value: Value = if acc.arguments.trim().is_empty() {
                Value::Object(serde_json::Map::new())
            } else {
                serde_json::from_str(&acc.arguments).unwrap_or_else(|_| {
                    // Argument non-parseable → on passe le string brut pour debug.
                    Value::String(acc.arguments.clone())
                })
            };
            tool_calls.push((acc.name.clone(), args_value));
            ids.push(acc.id.clone());
            extra_content.push(acc.extra_content.clone());
        }
        (tool_calls, ids, extra_content)
    }
}

fn append_bounded(
    target: &mut String,
    current_chars: &mut usize,
    fragment: &str,
    max_chars: usize,
    exceeded: &mut bool,
) {
    let fragment_chars = fragment.chars().count();
    if current_chars.saturating_add(fragment_chars) > max_chars {
        *exceeded = true;
        return;
    }
    *current_chars += fragment_chars;
    target.push_str(fragment);
}

fn value_len(value: &Value) -> usize {
    serde_json::to_string(value)
        .map(|s| s.chars().count())
        .unwrap_or(MAX_EXTRA_CONTENT_CHARS + 1)
}

#[cfg(test)]
#[path = "stream_tools_tests.rs"]
mod tests;
