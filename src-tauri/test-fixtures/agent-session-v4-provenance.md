# Provenance de la fixture de session v4

La fixture est synthétique et ne contient aucune donnée utilisateur. Elle a été
produite le 5 septembre 2026 par le sérialiseur réel de Beaver `v1.2.1`, au tag
Git `v1.2.1` et au commit
`335c3959d5ca9a3c1b9a90e7aad50d75a2cf3a61`.

## Procédure vérifiée

1. Créer un worktree détaché sur `v1.2.1` dans un répertoire temporaire.
2. Construire une `AgentSession` v4 synthétique avec trois messages et une
   véritable entrée `ToolActivityRecord` nommée `write_file`.
3. Dans un test ignoré temporaire de `session_store_document.rs`, passer cette
   session à `prepare`, puis à `write_prepared_to_path` dans un `tempfile`.
4. Exécuter le test avec le CEF vérifié du dépôt :

   ```text
   cargo test --lib export_v121_tool_activity_fixture -- --ignored --nocapture
   ```

5. Copier exactement le document écrit entre les marqueurs de sortie, sans le
   modifier à la main. Le test a terminé avec `1 passed; 0 failed`.

Le harnais temporaire a ensuite été retiré avec son worktree. Le test permanent
`v4_tool_activity_fixture_migrates_with_empty_artifacts_and_exact_backup`
prouve que la fixture reste lisible, que la migration initialise une liste
`artifacts` vide et que la sauvegarde `.v4.bak` est octet pour octet identique.
