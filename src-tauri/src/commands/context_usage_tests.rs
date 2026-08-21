#[test]
fn hidden_usage_resolves_context_before_selecting_the_prompt() {
    let source = include_str!("context_usage.rs");
    let context = source
        .find("context_usage_memory::usage")
        .expect("la jauge doit résoudre les métadonnées du contexte");
    let prompt = source
        .find("system_prompt_defaults::beaver_prompt")
        .expect("la jauge doit mesurer le prompt Beaver");

    assert!(
        context < prompt,
        "la taille Ollama doit choisir le prompt mesuré"
    );
    assert!(
        !source.contains("system_prompt_defaults::tier_for_model"),
        "la commande ne doit pas réintroduire un choix fondé sur le nom"
    );
}
