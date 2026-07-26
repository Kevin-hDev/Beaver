use super::*;

#[test]
fn manual_mode_requires_an_explicit_request() {
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = begin(&session, MemoryMode::Manual, false, 3_000, 500);
    assert!(read_allowed(&session));
    assert!(!write_allowed(&session));

    let session_allowed = uuid::Uuid::new_v4().to_string();
    let _allowed = begin(
        &session_allowed,
        MemoryMode::Manual,
        true,
        3_000,
        500,
    );
    assert!(write_allowed(&session_allowed));
}

#[test]
fn natural_requests_are_detected_in_supported_languages() {
    for message in [
        "Souviens-toi que je préfère ceci",
        "Ajoute dans ta memoire que je préfère ceci",
        "Remember this preference",
        "Recuerda esta decisión",
        "Erinnere dich daran",
        "Ricorda questa preferenza",
        "请记住这个偏好",
        "これを覚えてください",
    ] {
        assert!(has_explicit_request(message), "{message}");
    }
    assert!(!has_explicit_request("Explique-moi simplement ce fichier"));
}

#[test]
fn dynamic_results_never_exceed_the_turn_budget() {
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = begin(&session, MemoryMode::Automatic, true, 100, 80);
    let (output, truncated) = consume_result(&session, &"mémoire ".repeat(500));

    assert!(truncated);
    assert!(crate::services::token_counting::estimate_text_tokens(&output) <= 20);
    assert_eq!(
        consume_result(&session, "encore").0,
        "[résultat mémoire omis : budget épuisé]"
    );
}

#[test]
fn parallel_results_share_one_atomic_budget() {
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = begin(&session, MemoryMode::Automatic, true, 120, 80);
    let barrier = std::sync::Arc::new(std::sync::Barrier::new(8));
    let handles = (0..8)
        .map(|_| {
            let session = session.clone();
            let barrier = barrier.clone();
            std::thread::spawn(move || {
                barrier.wait();
                consume_result(&session, &"mémoire parallèle ".repeat(200)).0
            })
        })
        .collect::<Vec<_>>();
    let consumed = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .filter(|output| !output.contains("budget épuisé"))
        .map(|output| crate::services::token_counting::estimate_text_tokens(&output))
        .sum::<usize>();

    assert!(consumed <= 40);
    assert_eq!(
        consume_result(&session, "encore").0,
        "[résultat mémoire omis : budget épuisé]"
    );
}

#[test]
fn final_partial_result_cannot_exceed_a_tiny_remaining_budget() {
    let session = uuid::Uuid::new_v4().to_string();
    let _guard = begin(&session, MemoryMode::Automatic, true, 81, 80);

    let output = consume_result(&session, &"mémoire ".repeat(200)).0;

    assert!(crate::services::token_counting::estimate_text_tokens(&output) <= 1);
}
