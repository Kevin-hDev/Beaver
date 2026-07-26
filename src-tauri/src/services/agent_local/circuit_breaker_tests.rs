#[cfg(test)]
mod tests {
    use crate::services::agent_local::circuit_breaker::CircuitBreaker;
    use serde_json::json;

    fn call(name: &str, arg: i64) -> Vec<(String, serde_json::Value)> {
        vec![(name.to_string(), json!({ "n": arg }))]
    }

    #[test]
    fn allows_different_calls() {
        let mut breaker = CircuitBreaker::new();
        for i in 0..10 {
            let calls = call("bash", i);
            assert!(
                breaker.check(&calls).is_ok(),
                "appel #{i} devrait être autorisé"
            );
        }
    }

    #[test]
    fn trips_on_identical_calls() {
        let mut breaker = CircuitBreaker::new();
        let calls = call("bash", 42);

        // 1 à 5 appels identiques : OK (seuil = 6)
        for i in 1..=5 {
            assert!(
                breaker.check(&calls).is_ok(),
                "l'appel #{i} identique devrait être autorisé"
            );
        }
        // 6ème appel identique : ERREUR (count atteint MAX_CONSECUTIVE_IDENTICAL)
        let result = breaker.check(&calls);
        assert!(
            result.is_err(),
            "le 6ème appel identique consécutif devrait déclencher le circuit breaker"
        );
        let msg = result.unwrap_err();
        assert!(msg.contains("Circuit breaker"), "message inattendu : {msg}");
    }

    #[test]
    fn trips_on_reordered_json_keys() {
        // Deux tool_calls identiques en valeur mais avec clés JSON dans un ordre différent
        // doivent produire la même signature → détecter la boucle
        let mut breaker = CircuitBreaker::new();

        let call_ab = vec![(
            "write_file".to_string(),
            json!({ "path": "x", "content": "y" }),
        )];
        let call_ba = vec![(
            "write_file".to_string(),
            json!({ "content": "y", "path": "x" }),
        )];

        // 1 à 5 alternances (ab/ba = même signature normalisée) : OK
        for i in 1..=5 {
            let c = if i % 2 == 1 { &call_ab } else { &call_ba };
            assert!(
                breaker.check(c).is_ok(),
                "l'appel #{i} (clés inversées) devrait être autorisé"
            );
        }
        // 6ème appel : compteur = 6 → ERREUR circuit breaker
        let result = breaker.check(&call_ab);
        assert!(
            result.is_err(),
            "les clés inversées doivent être détectées comme identiques au 6ème appel"
        );
    }

    #[test]
    fn resets_on_different_call() {
        let mut breaker = CircuitBreaker::new();
        let same = call("bash", 99);
        let different = call("bash", 100);

        // 5 appels identiques : OK
        for _ in 0..5 {
            assert!(breaker.check(&same).is_ok());
        }
        // Appel différent : reset → OK
        assert!(breaker.check(&different).is_ok());
        // Reprendre les appels identiques au début (compteur remis à 1)
        for i in 1..=5 {
            assert!(
                breaker.check(&same).is_ok(),
                "l'appel #{i} identique après reset devrait être autorisé"
            );
        }
        // Maintenant le 6ème identique consécutif => ERREUR
        assert!(breaker.check(&same).is_err());
    }
}
