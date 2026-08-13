# Conception — corrections mineures du jalon 2 de fermeture

Date : 2026-08-12
Branche : `codex/shutdown-milestone-2-minors`
Base : `origin/main` à `b863903`

## But

Traiter tous les constats exploitables des sections **Mineurs** et **Nouveaux mineurs et remarques** du rapport du jalon 2, sans rouvrir les points classés **Infos consignées**.

La raison de ce périmètre est simple : l'utilisateur a demandé les corrections mineures après la fusion et a explicitement exclu les informations consignées, qui décrivent des choix ou des dettes sans action immédiate.

## État de référence observé

- Suite frontend complète : 417 fichiers et 1 916 tests réussis, code de sortie 0.
- Suite Rust : 3 198 tests, dont 3 196 réussis et 2 ignorés, aucun échec, code de sortie 0.
- Le premier lancement Rust a été interrompu uniquement par la limite de cinq minutes pendant la compilation ; le second, avec le cache prêt et une limite adaptée, est vert.

Ces résultats constituent la base de comparaison. Une correction n'est déclarée terminée que si son test ciblé puis les contrôles proportionnés sont réellement exécutés et lus.

## Périmètre exact

Chaque ligne ci-dessous reçoit un statut vérifiable dans le plan d'exécution puis dans les commits : `confirmé et corrigé`, `déjà corrigé sur main`, ou `non applicable` avec une preuve. Aucun constat ne peut disparaître silencieusement.

### Lot 1 — sécurité et propriété des processus

1. Remplacer la comparaison XOR manuelle du jeton terminal par `subtle::ConstantTimeEq`.
2. Revalider l'identité des descendants Unix avant tout signal individuel.
3. Rattacher la tâche de focus de la mascotte à un propriétaire annulable.
4. Rattacher les dispatches empressés au flux parent au lieu d'un `tokio::spawn` nu.
5. Prendre l'admission du lecteur d'extension avant de lancer l'enfant Node.
6. Rendre la désinstallation d'extension annulable.
7. Fermer la fenêtre Forecast entre lancement et publication du handle, notamment sur macOS.
8. Supprimer le double contrôle `is_forecast_process`.
9. Résoudre PowerShell par une autorité absolue partagée, jamais par le `PATH`.
10. Effacer réellement le message Discord qui contient le jeton, pas seulement une copie JSON.
11. Ne pas recopier le jeton Forecast dans un `String` ordinaire non effaçable.
12. Remplacer un flux agent sans consommer temporairement deux places d'admission.

### Lot 2 — scheduler et gateway

13. Tracer les refus d'admission aujourd'hui avalés au démarrage et dans le file watcher.
14. Consommer `EnqueueOutcome`, imposer son usage et arrêter une boucle de canal sur `Closed`.
15. Conserver une décision durable lorsque l'écriture d'un refus scheduler échoue.
16. Compter localement les messages perdus même quand l'audit gateway est désactivé.
17. Donner une décision durable aux occurrences situées dans la grâce de cinq minutes après redémarrage.
18. Garder une raison d'audit utile mais bornée au lieu de tout remplacer par `operation_failed`.
19. Borner les reconnexions internes Discord et Slack avec une politique partagée.
20. Éviter la relecture complète du journal scheduler à chaque ajout.
21. Préserver une présentation compréhensible des anciennes entrées de réveil sans `error_code` et documenter la compatibilité.

### Lot 3 — extensions, MCP et OAuth

22. Continuer le serveur de callback MCP après un mauvais `state`, jusqu'à un callback valide ou l'échéance.
23. Remplacer les deux scrutations OAuth dupliquées par l'attente du vrai travail possédé.
24. Remplacer l'attente MCP fixe de 500 ms par un signal réel de disponibilité ou de terminaison.
25. Transformer le `include!` de MCP stdio en modules à responsabilités explicites.
26. Faire exercer aux tests la remise à zéro réellement utilisée en production par les téléchargements de modèles.
27. Ajouter les tests de séquence complète de fermeture MCP et extensions.
28. Retirer les coutures de test d'extension du binaire publié.

### Lot 4 — Forecast, SearXNG, GPU et runtime

29. Déplacer les lectures sysfs GPU bloquantes hors des workers Tokio.
30. Borner l'attente du lecteur après annulation d'une sonde GPU.
31. Aligner le délai de réutilisation du sidecar Forecast sur le coût maximal de sa sonde de santé.
32. Découper le cycle de vie SearXNG par responsabilité et sortir `save_pid` du verrou de processus.

### Lot 5 — contrats, interface et tests anti-régression

33. Distinguer en japonais les états `missed` et `never`.
34. Renforcer les tests de codes d'erreur de réveil : noms exacts des clés et rendu réel du composant.
35. Supprimer les constantes ou comportements de test qui remplacent silencieusement la production, notamment dans le registre de sous-agents et Codex OAuth.

Certains points du rapport regroupent plusieurs fichiers ou symptômes ; c'est pourquoi cette liste compte 35 obligations vérifiables pour 32 puces de revue.

## Hors périmètre

Les sept éléments de **Infos consignées** ne sont pas modifiés dans cette branche : fenêtre de crash du helper transféré, rôle diagnostique de `ProcessIdentity`, émetteur scheduler non vidé, classes d'admission gateway/réveils, registre du callback Codex, scrutation ConPTY et délai de grâce générique, ainsi que `tool_document_write_xml.rs`.

Une correction nécessaire à un élément inclus peut toucher du code voisin, mais elle ne doit pas transformer l'un de ces choix sans une nouvelle décision explicite.

## Autorités et sens des dépendances

- Le propriétaire d'une tâche ou d'un processus fournit son annulation, son échéance et sa moisson ; les appelants ne créent pas un second mécanisme parallèle.
- La comparaison des secrets dépend de la primitive cryptographique commune ; aucune comparaison locale n'en réimplémente le contrat.
- La résolution des exécutables système dépend d'une seule autorité validée par plateforme.
- Les politiques de reconnexion, d'attente et de grâce vivent dans une autorité nommée et testée ; les services les consomment.
- Le scheduler est l'autorité de décision durable pour chaque occurrence ; un échec de journalisation ne devient jamais une absence silencieuse de décision.
- Le compteur gateway est l'autorité locale de perte ; l'audit persistant est un consommateur optionnel, pas la seule preuve qu'un message a été refusé.
- Les coutures de test observent le chemin de production ou sont absentes du binaire publié ; elles ne remplacent jamais le comportement testé.

La raison est d'empêcher le problème déjà observé sur ce chantier : un mécanisme adopté à moitié laisse deux comportements qui divergent au prochain changement.

## Méthode d'exécution

1. Revalider le constat dans le code fusionné avant toute modification.
2. Écrire un test qui échoue pour la raison attendue.
3. Appliquer le correctif minimal en réutilisant l'autorité existante ou en en créant une seule si elle manque.
4. Exécuter le test ciblé et lire sa sortie verte.
5. Vérifier les fichiers touchés : sécurité, erreurs propagées, collections bornées, absence de secret dans les traces et limite de 230 lignes.
6. Committer un lot cohérent et ajouter une git note contenant la raison, les tests exécutés et leur résultat exact.

Les cinq lots restent séparés afin que la review puisse attribuer chaque changement à un risque précis. Si la revalidation montre qu'un constat est déjà corrigé, aucun changement artificiel n'est produit : la preuve est inscrite dans le registre d'exécution.

## Vérification finale

- Tests ciblés rouges puis verts pour chaque comportement modifié.
- `npm test` complet.
- `npx tsc --noEmit`.
- `npm run lint`.
- `cargo test --lib --features windows-tests -- --test-threads=1`.
- `cargo check`.
- `cargo clippy --all-targets --features windows-tests -- -D warnings`.
- Contrôle structurel des fichiers de code touchés sous 230 lignes.
- `graphify update .` après les changements de code pour maintenir le graphe AST local.
- Push de la branche et CI verte avant la nouvelle review.

## Critères d'acceptation

- Les 35 obligations ci-dessus ont chacune un statut et une preuve.
- Aucun élément de **Infos consignées** n'est modifié sans décision nouvelle.
- Aucun processus ou travail asynchrone ajouté n'est détaché de son propriétaire.
- Aucun secret n'est comparé avec `==`, copié dans une chaîne ordinaire durable ou écrit dans une trace.
- Toute file alimentée de l'extérieur reste bornée et rend sa perte observable.
- Toute attente de fermeture ou de disponibilité est bornée par une échéance ou un signal possédé.
- Les sept langues restent cohérentes lorsqu'un texte visible change.
- Chaque commit de correction possède une git note fidèle au code et aux tests réellement exécutés.
- La branche est poussée pour review, mais aucune fusion dans `main` n'est faite avant validation de l'utilisateur.
