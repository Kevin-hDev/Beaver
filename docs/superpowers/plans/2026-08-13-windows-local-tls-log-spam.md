# Plan d'implémentation — bruit TLS du détecteur local Windows

> **Pour l'agent :** exécuter ce plan avec `superpowers:executing-plans`, en
> TDD strict et dans le worktree isolé `C:\Users\huynh\btls`.

**But :** supprimer la trace TLS répétitive du scanner local Windows sans
accepter le certificat et sans masquer les erreurs TLS des autres services.

**Architecture :** `services::app_log` possède une portée asynchrone qui filtre
une seule cible de journal. `local_site_probe` place seulement son envoi HTTPS
dans cette portée. Le client Reqwest et sa politique de confiance restent
inchangés.

**Technologies :** Rust, Tokio task-local, `tauri-plugin-log`, Reqwest,
`rustls-platform-verifier`, tests Cargo Windows.

---

## Tâche 1 — Verrouiller le contrat du journal

**Fichiers :**

- modifier `src-tauri/src/services/app_log_tests.rs` ;
- modifier `src-tauri/src/services/app_log.rs`.

1. Ajouter trois tests qui demandent l'API de portée souhaitée : cible TLS
   autorisée hors portée, refusée dans la portée, autre cible autorisée dans la
   portée.
2. Lancer `cargo test app_log_tests --lib --features windows-tests` et observer
   l'échec dû à l'API absente.
3. Ajouter la constante de cible, la portée Tokio task-local et la décision de
   filtrage minimale dans `app_log.rs`, avec la raison écrite sur place.
4. Brancher cette décision sur le filtre unique de `tauri-plugin-log`.
5. Relancer le test ciblé et obtenir le vert.

## Tâche 2 — Confinement au probe HTTPS local

**Fichiers :**

- modifier `src-tauri/src/services/browser/local_site_probe_tests.rs` ;
- modifier `src-tauri/src/services/browser/local_site_probe.rs` ;
- modifier les dépendances de test Windows dans `src-tauri/Cargo.toml` seulement
  si le vrai serveur TLS l'exige.

1. Ajouter un test Windows en sous-processus avec un vrai serveur TLS local
   auto-signé et un logger capturant les décisions du filtre.
2. Prouver avant correction que le certificat est refusé mais que la trace
   réelle est encore émise.
3. Entourer uniquement `send().await` du probe HTTPS avec la portée définie par
   `app_log`; laisser HTTP et tous les autres clients inchangés.
4. Relancer le test et prouver simultanément le refus du certificat, l'absence
   d'émission et le passage réel de la trace par le filtre.
5. Relancer les deux tests HTTP existants.

## Tâche 3 — Vérification complète et entretien

1. Lancer `cargo fmt --all -- --check`.
2. Lancer `cargo test local_site_probe --lib --features windows-tests -- --test-threads=1`.
3. Lancer `cargo test app_log_tests --lib --features windows-tests -- --test-threads=1`.
4. Lancer `cargo clippy --all-targets -- -D warnings`; le profil
   `windows-tests` remplace CEF par ses stubs documentaires et produit des
   avertissements de code mort qui ne représentent pas le build Windows réel.
5. Lancer le profil Windows complet prescrit par le projet si sa commande est
   disponible localement.
6. Lancer `graphify update .` depuis la racine du worktree.
7. Vérifier `git diff --check`, les tailles des fichiers de code et le statut
   Git. Ne déclarer le correctif terminé que si toutes les sorties sont vertes.
