#[cfg(test)]
mod tests {
    use crate::services::agent_local::circuit_breaker::CircuitBreaker;
    use serde_json::json;

    const OWNER: &str = "00000000-0000-4000-8000-000000000001";

    fn call(name: &str, arg: i64) -> Vec<(String, serde_json::Value)> {
        vec![(name.to_string(), json!({ "n": arg }))]
    }

    #[test]
    fn allows_different_calls() {
        let mut breaker = CircuitBreaker::new();
        for i in 0..10 {
            let calls = call("bash", i);
            assert!(
                breaker.check(&calls, OWNER).is_ok(),
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
                breaker.check(&calls, OWNER).is_ok(),
                "l'appel #{i} identique devrait être autorisé"
            );
        }
        // 6ème appel identique : ERREUR (count atteint MAX_CONSECUTIVE_IDENTICAL)
        let result = breaker.check(&calls, OWNER);
        assert_eq!(result, Err("circuit_breaker".to_string()));
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
                breaker.check(c, OWNER).is_ok(),
                "l'appel #{i} (clés inversées) devrait être autorisé"
            );
        }
        // 6ème appel : compteur = 6 → ERREUR circuit breaker
        let result = breaker.check(&call_ab, OWNER);
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
            assert!(breaker.check(&same, OWNER).is_ok());
        }
        // Appel différent : reset → OK
        assert!(breaker.check(&different, OWNER).is_ok());
        // Reprendre les appels identiques au début (compteur remis à 1)
        for i in 1..=5 {
            assert!(
                breaker.check(&same, OWNER).is_ok(),
                "l'appel #{i} identique après reset devrait être autorisé"
            );
        }
        // Maintenant le 6ème identique consécutif => ERREUR
        assert!(breaker.check(&same, OWNER).is_err());
    }

    #[test]
    fn missing_shell_session_controls_are_bounded() {
        let calls = vec![(
            "bash_control".to_string(),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099" }),
        )];
        assert_sixth_call_trips(&calls);
    }

    #[test]
    fn active_or_malformed_shell_controls_are_bounded() {
        let cases = [
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "chars": "x" }),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "eof": true }),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "stop": true }),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "chars": 7 }),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "stop": "false" }),
            json!({ "session_id": "00000000-0000-4000-8000-000000000099", "unknown": true }),
        ];

        for args in cases {
            assert_sixth_call_trips(&[("bash_control".to_string(), args)]);
        }
    }

    fn assert_sixth_call_trips(calls: &[(String, serde_json::Value)]) {
        let mut breaker = CircuitBreaker::new();
        for _ in 0..5 {
            assert!(breaker.check(calls, OWNER).is_ok());
        }
        assert_eq!(
            breaker.check(calls, OWNER),
            Err("circuit_breaker".to_string())
        );
    }
}
