use crate::services::agent_local::ollama_base_url;
use reqwest::Client;
use std::time::Duration;
const DEFAULT_MODEL: &str = "gemma4:e2b";
const TIMEOUT: Duration = Duration::from_secs(600);
const CHUNK_MAX_CHARS: usize = 8000;

pub async fn translate_text(
    text: &str,
    target_lang: &str,
    model: Option<&str>,
) -> Result<String, String> {
    let lang_name = match target_lang {
        "fr" => "French",
        "es" => "Spanish",
        "de" => "German",
        "zh" => "Simplified Chinese",
        _ => return Err(format!("Langue non supportée : {target_lang}")),
    };

    let chosen_model = model.unwrap_or(DEFAULT_MODEL).to_string();
    let chunks = chunk_by_headings(text, CHUNK_MAX_CHARS);
    eprintln!(
        "[translator] model={chosen_model} lang={target_lang} total_chars={} chunks={}",
        text.len(),
        chunks.len(),
    );

    let mut translated_parts = Vec::with_capacity(chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        eprintln!(
            "[translator] chunk {}/{} ({} chars)",
            i + 1,
            chunks.len(),
            chunk.len()
        );
        let translated = translate_chunk(chunk, lang_name, &chosen_model).await?;
        translated_parts.push(translated);
    }

    Ok(translated_parts.join("\n\n"))
}

fn chunk_by_headings(text: &str, max_chars: usize) -> Vec<String> {
    if text.len() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks: Vec<String> = Vec::new();
    let mut current = String::new();
    for line in text.lines() {
        let is_heading = line.starts_with("# ")
            || line.starts_with("## ")
            || line.starts_with("### ")
            || line.starts_with("#### ");
        let is_blank = line.trim().is_empty();
        let len = current.len();

        // 1. Hard split si on dépasse max_chars sur une ligne vide (paragraphe)
        if is_blank && len >= max_chars {
            chunks.push(std::mem::take(&mut current));
            continue;
        }
        // 2. Split préférentiel sur heading au-delà de la moitié
        if is_heading && len >= max_chars / 2 {
            chunks.push(std::mem::take(&mut current));
        }
        // 3. Split sur paragraphe vide au-delà de 75%
        else if is_blank && len >= max_chars * 3 / 4 {
            chunks.push(std::mem::take(&mut current));
            continue;
        }
        // 4. Hard split forcé si le buffer dépasse 1.5× max_chars (sécurité)
        else if len >= max_chars * 3 / 2 {
            chunks.push(std::mem::take(&mut current));
        }

        current.push_str(line);
        current.push('\n');
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    if chunks.is_empty() {
        chunks.push(text.to_string());
    }
    chunks
}

async fn translate_chunk(text: &str, lang_name: &str, model: &str) -> Result<String, String> {
    let system = format!(
        "You are a translator. Your ONLY task is to translate the Markdown fragment below into {lang_name}.\n\
         \n\
         DO NOT SUMMARIZE. DO NOT ANALYZE. DO NOT EXPLAIN. JUST TRANSLATE EVERY SENTENCE.\n\
         \n\
         STRICT RULES:\n\
         1. Translate ALL prose, bullet points, headings, and table cells.\n\
         2. Keep ALL Markdown formatting intact (# headings, - lists, | tables, ``` code blocks).\n\
         3. Keep ALL HTML tags EXACTLY as-is: <img src=\"...\" />, <br>, <a href=\"...\">, <hr>. Never remove, modify, or translate URL paths, src=, href=, width=, height=.\n\
         4. Keep ALL URLs, file paths, code snippets, variable names, command names, numeric values unchanged.\n\
         5. The output length must be similar to the input length.\n\
         6. Output ONLY the translated Markdown. No preamble. No explanation. No wrapping."
    );

    let start = std::time::Instant::now();
    let client = Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client
        .post(format!("{}/api/chat", ollama_base_url()))
        .json(&serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": text}
            ],
            "stream": false,
            "keep_alive": "0",
            "truncate": false,
            "options": {"temperature": 0.2, "num_ctx": 16384, "num_predict": 12288}
        }))
        .send()
        .await
        .map_err(|e| format!("Erreur appel Ollama : {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        eprintln!(
            "[translator] HTTP error status={status} body={}",
            crate::services::llm::sanitize_log_body(&body)
        );
        if status.as_u16() == 404 {
            return Err(format!(
                "Modèle '{model}' non installé. Installe-le via l'onglet Models."
            ));
        }
        return Err(format!("Échec traduction (HTTP {status})"));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    let elapsed = start.elapsed().as_secs();
    let done_reason = json["done_reason"].as_str().unwrap_or("?").to_string();
    eprintln!(
        "[translator]   chunk done in {elapsed}s eval={} reason={done_reason}",
        json["eval_count"].as_u64().unwrap_or(0),
    );
    let content = json["message"]["content"]
        .as_str()
        .ok_or("Réponse Ollama invalide")?
        .trim()
        .to_string();
    if content.is_empty() {
        return Err("Traduction vide (modèle trop petit ou contexte dépassé)".into());
    }
    Ok(content)
}
