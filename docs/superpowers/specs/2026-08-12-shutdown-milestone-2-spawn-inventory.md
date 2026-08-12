# Jalon 2 — inventaire exhaustif des processus et tâches

## Photographie et méthode

Photographie auditée : `b104cc3`, après le confinement natif `1f8fd38`.

Le contrat de marque passe honnêtement de 231 à 234 occurrences de l'identifiant interne historique : les trois références nouvelles sont les noms de processus de test utilisés pour classer les WebViews sur Windows, Linux et macOS. Le nombre de références inconnues reste nul ; aucune exclusion n'est ajoutée.

Le balayage porte sur tout `src-tauri/src` de production avec :

- `Command::new`, `tokio::process::Command` et chaque appel `.spawn()` associé ;
- `portable-pty` et `spawn_command` ;
- `tokio::spawn`, `tauri::async_runtime::spawn`, `std::thread::spawn` et `std::thread::Builder::spawn` ;
- les créations indirectes de WebView observées dans le vrai arbre de processus.

Les blocs `#[cfg(test)]` sont exclus du décompte de production, mais leurs preuves sont indiquées. Les `spawn_blocking` dont le handle est immédiatement attendu par la même requête sont regroupés comme opérations synchrones déportées : ils ne possèdent ni service durable ni processus survivant.

## Autorités uniques

| Ressource | Autorité | Borne | Arrêt |
|---|---|---:|---|
| Admission de l'application | `AppExitCoordinator::AdmissionRegistry` | 128 | Fermée et annulée par la première transition vers `Closing` |
| Tâches d'un domaine | `ServiceWorkSupervisor` du domaine | Tableau fixe propre au domaine | `stop_and_wait(deadline)` |
| Boucles générales du runtime | `RuntimeBackgroundServices` | 4 boucles + 8 tâches | Annulation, attente puis abandon à l'échéance absolue |
| Processus enfant possédé | `OwnedProcess` | Job Object ou groupe natif ; 64 identités macOS | Signal du groupe, terminaison, moissonnage |
| Terminal | `PtyManager` + `OwnedProcess::adopt_existing` | 16 sessions | Lecteur annulé, PTY fermé, processus et thread attendus |
| Helper de mise à jour transféré | `UpdateHandoff` | 1 identité validée | Arrêt avant transfert ; unique survivant après transfert |

La raison de `RuntimeBackgroundServices` est de donner un propriétaire aux trois boucles permanentes trouvées par le balayage final : surveillance de fichiers, polling Ollama et nettoyage d'activité de la mascotte. Le même propriétaire suit les réparations de démarrage, l'initialisation Ollama/LiteLLM, le message différé du vault et les commandes de démarrage du gateway.

## Processus enfants possédés

| Résultat de recherche | Propriétaire et admission | Annulation, attente et confinement | Preuve |
|---|---|---|---|
| Shells agent : `tool_bash_process`, `tool_bash_profile`, `subagent_explorer_process`, `shell_environment_capture` ; les constructeurs sous `shell_sandbox/*` et `shell_environment_unix` aboutissent à ces chemins | `AgentWorkServices`, 64 shells ; lancement par `OwnedProcess` | Jeton propagé, lecteurs bornés, groupe/Job natif, terminaison et attente | `54aa453`, `1f8fd38`; tests `tool_bash_supervision`, `owned_process` et environnement shell |
| Hôte et installateurs d'extensions : `extensions/host_process.rs`, `extensions/process_runner.rs` | `ExtensionWorkServices`, registres fixes lecteurs/opérations/appels | Admission fermée avant l'hôte, lecteurs annulés, enfant confiné et moissonné | `8066a3c`, `1f8fd38`; tests `work_supervision` et processus réel |
| MCP stdio : `mcp_bridge/process_spawn.rs` | `McpWorkServices` et pool de processus borné | stdin fermé, lecteurs annulés, `OwnedProcess`, attente du processus | `847c4e1`, `1f8fd38`; fixture stdio réelle |
| SearXNG : `searxng/process.rs`, `searxng/runtime.rs` | `SearxngSidecar`, registres fixes démarrage/serveur | Annulation pendant installation et serveur, `OwnedProcess`, `stop_and_wait` | `2204923`, `1f8fd38`; tests de cycle réel |
| Forecast : `sidecar_spawn`, `sidecar_runtime_command`, `sidecar_runtime_install`, `model_manager/smoke` | `ChronosSidecar` et `ForecastWorkServices` | Annulation des opérations, installateurs et sidecar ; `OwnedProcess`, moissonnage | `0bbed33`, `1f8fd38`; tests sidecar/runtime/smoke |
| Ollama : `ollama_lifecycle.rs` | `OllamaSidecar`; démarrage et polling suivis par `RuntimeBackgroundServices` | `OwnedProcess`; polling annulable ; `stop_sidecar` après les autres services | `1f8fd38`, `b104cc3`; tests de possession voisins. La transaction d'installation reste J3 |
| PTY : `terminal/pty_session.rs::spawn_command` | `PtyManager`, 16 sessions | PID portable-pty adopté avant restitution, lecteur et PTY attendus hors verrou | `ad90f5e`, `1f8fd38`; vrai shell lancé puis absent |

`OwnedProcess` est le seul appel direct à `Command::spawn` pour ces enfants. Sur Windows il affecte le Job Object global avant de rendre le handle. Sur Linux il crée un groupe et arme `PR_SET_PDEATHSIG`. Sur macOS il enregistre une identité dans 64 slots et la revalide avant signal. Un refus d'admission tue et moissonne l'enfant avant de retourner l'erreur.

## Exceptions de processus documentées

| Résultat | Classement et raison | Sécurité et attente |
|---|---|---|
| `commands/app_update_helper_process.rs` | Exception possédée puis transférée, volontairement hors Job destructeur | Admission `AppUpdateRuntime`; identité PID/parent/démarrage/exécutable ; `Drop` tue avant transfert ; `UpdateHandoff` conserve au plus un helper validé (`d66f3e5`) |
| `updater_worker/command.rs` | Processus lancés par le helper après le transfert, donc hors runtime Beaver | Arguments validés et bornés ; commandes attendues avec délai ; lecteur de sortie joint ; seul l'installateur final explicitement lancé en arrière-plan survit au helper |
| `personality.rs`, `projects.rs`, `file_preview.rs`, `forecast/notes.rs`, `mcp_oauth/flow_auth.rs` | Applications externes demandées par l'utilisateur (`open`, `xdg-open`, `explorer`, éditeur, navigateur OAuth) | Chemins/URL validés ; aucun handle adopté ; Beaver ne les annule et ne les tue jamais |
| `windows_entry.rs` | Bootstrap de développement avant le runtime Tauri | Commande fixe et statut attendu ; absent de la version publiée |
| `shell_sandbox/linux.rs::exec` | Remplacement du processus shell déjà possédé, pas création d'un descendant détaché | Le PID/groupe reste celui admis par `OwnedProcess` |
| `background_command.rs` et constructeurs `shell_sandbox/*` | Constructeurs seulement | L'appelant final appartient à une ligne possédée ou externe ci-dessus |

## Commandes courtes sans processus durable

Les appels suivants utilisent `output` ou `status` et rendent leur résultat avant de rendre la main. Ils ne créent aucun service, n'exposent aucun handle et sont donc exemptés d'admission longue :

- détection OS/GPU : `env_detect.rs`, `gpu_detect.rs`, `gpu_vram/{macos,linux}.rs` ;
- inspection de processus : `ollama_kill.rs`, `searxng/process.rs`, `forecast/sidecar_process.rs` ;
- association d'éditeur Linux : `file_preview_editors/linux.rs` ;
- suppression de quarantaine macOS : `ollama_setup_install.rs` ;
- sonde Ollama : `ollama_port.rs`, sur un thread nommé immédiatement joint avec délai HTTP de deux secondes.

Ces commandes ne sont pas adoptées par `OwnedProcess`, précisément pour ne pas transformer une sonde finie ou une application externe en enfant durable de Beaver.

## Tâches asynchrones et threads

| Résultat de recherche | Propriétaire | Borne et fin | Preuve ou exemption |
|---|---|---|---|
| `work_registry/task.rs::tokio::spawn` | Chaque `ServiceWorkSupervisor` | Tableau fixe ; handle stocké avant restitution ; annulation, attente ou abort | `9594582` et suites `work_registry` |
| Flux agent, sous-agents, eager tools et lecteurs bash | `AgentWorkServices`; les handles eager/lecteurs sont attendus par le flux parent | 32 flux, 8 sous-agents, 64 shells, eager limité à 10, canal bash 64 | `54aa453`; tests succès/erreur/annulation/panique |
| Gateway, extensions, MCP, OAuth, SearXNG, Forecast, téléchargements, scheduler, update et terminal | Superviseur local de chaque domaine | Bornes fixes documentées dans leurs modules ; `stop_and_wait` idempotent | `696fb39` à `ad90f5e` et notes Git correspondantes |
| Surveillance de fichiers, polling Ollama, mascotte, récupérations et initialisations de démarrage, démarrage/toggle gateway | `RuntimeBackgroundServices` | 4 boucles + 8 tâches ; le watcher ne conserve que 256 chemins par fenêtre de debounce | `b104cc3`; tests d'annulation/attente/refus après fermeture |
| Lecteur PTY et sampler mémoire Forecast | Objet parent `OwnedSession` / `MemorySampler` | Un thread par objet admis ; drapeau d'arrêt ; handle joint par `close` / `finish` | Tests terminal réel et évaluation Forecast |
| Lecteur de capture shell | Shell agent possédé | Une lecture bornée à `MAX_CAPTURE_BYTES`, attente 250 ms ; la fermeture du pipe après moissonnage force sa fin | Tests de capture et `OwnedProcess` |
| `agent_chat_task/session_events`, `tool_executor_delegate_batch`, focus mascotte, pont d'état navigateur | Tâche parente déjà admise ou callback natif | Opérations finies, sans processus ni ressource durable ; résultats attendus par canal lorsqu'ils influencent le tour | Exemption courte ; fermeture des extensions refuse tout nouvel événement |
| `spawn_blocking` des commandes Git, stockage, preview et scans | Future Tauri appelante | Handle immédiatement attendu ; aucune boucle, enfant ou ressource durable | Exemption synchrone déportée ; la sortie du processus termine le calcul si la requête est interrompue |
| Watchdogs `app_exit/{watchdog,ultimate}` | `AppExitCoordinator` | Threads précréés et déclenchement unique | Propriété J1, hors duplication J2 |

Les occurrences `tokio::spawn` dans `codex_oauth/login.rs`, `llm_oauth/lifecycle.rs` et `gateway/conversation_locks.rs` appartiennent uniquement à des blocs `#[cfg(test)]`.

## WebView et CEF

- Windows : seul `msedgewebview2.exe` descendant de Beaver est classé `Dedicated`.
- Linux : seuls les processus WebKit web/réseau/GPU descendants de Beaver sont `Dedicated`.
- macOS : les services `com.apple.WebKit.*` sont `SharedSystem`, même s'ils apparaissent pendant le test ; ils ne sont jamais signalés.
- L'inventaire natif est borné à 4 096 processus, 64 PID dédiés et 32 niveaux d'ascendance.
- Le parcours E2E ouvre une vraie WebView et vérifie la disparition des PID dédiés sur Windows/Linux ; macOS vérifie la classification partagée.

Les helpers CEF, leurs traqueurs, le cookie gate et leurs threads restent exclusivement couverts par le jalon 1B. Le présent jalon ne les réimplémente pas ; il renvoie aux commits J1B et étend seulement le parcours natif avec l'observation WebView dans `1f8fd38`.

## Nettoyage central final

`app_exit/cleanup.rs` appelle désormais uniquement les `stop_and_wait` des propriétaires, en parallèle et avec la même échéance absolue. Les anciens arrêts directs des flux et shells ont été retirés. Chaque résultat faux est tracé sans empêcher l'attente des autres services. Après cette phase, le nombre d'admissions globales est contrôlé et tracé ; zéro est la seule situation normale.

Preuves : `cleanup_tests::one_service_timeout_does_not_skip_the_other_services`, `cleanup_tests::the_global_registry_must_be_empty_after_service_cleanup` et `runtime_background_tests` dans `b104cc3`.

## Conclusion de l'inventaire

Chaque résultat de production du balayage a maintenant soit :

1. un propriétaire borné relié à l'admission globale, une annulation, une attente et un test ;
2. une propriété native spéciale et vérifiée (`UpdateHandoff`, WebView partagée, CEF J1B) ;
3. une exemption explicite parce que l'opération est courte et attendue, remplace le processus déjà possédé, ou ouvre volontairement une application externe.

Aucun processus externe ni service système partagé n'est adopté par Beaver.
