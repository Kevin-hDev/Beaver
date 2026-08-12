# Jalon 2 — corrections de la revue complète

## Autorité et objectif

Ce document complète le [jalon 2 — processus et services possédés](./2026-08-09-shutdown-milestone-2-services-design.md). Il ferme les deux bloquants et les vingt défauts à corriger relevés par la revue du 12 août 2026. Les remarques classées mineures restent hors périmètre.

La correction conserve les autorités existantes :

- `AppWorkSupervisor` décide si un nouveau travail global peut commencer ;
- chaque `ServiceWorkSupervisor` possède l'admission et l'annulation de son domaine ;
- chaque service possède son état et ses processus ;
- l'échéance absolue reçue par `stop_and_wait` borne toutes les attentes de fermeture en aval ;
- le frontend est seul responsable de traduire les codes publics stables.

Aucun lot ne crée un second registre global, une seconde chronologie de fermeture ou une seconde liste de codes publics.

## Approche retenue

Les corrections sont réalisées en sept lots indépendamment vérifiables. Chaque défaut reçoit d'abord un test qui échoue pour la raison attendue. Le correctif minimal est ensuite appliqué autour de l'autorité existante, puis les tests ciblés et les tests de régression du domaine sont exécutés.

Les lots restent séparés dans l'historique afin que la relecture puisse comparer chaque preuve rouge et verte au diff correspondant.

## Lot 1 — transfert du helper de mise à jour

Le helper validé est l'unique enfant autorisé à survivre à Beaver. Il ne doit donc pas recevoir le signal Linux de mort du parent destiné aux processus possédés.

`process_tree` expose une configuration dédiée au transfert :

- Windows conserve le chemin `UpdateHandoff` hors du Job Object destructeur ;
- Linux et macOS créent un groupe de processus dédié afin que le helper n'hérite pas d'un groupe utilisateur ;
- Linux n'arme pas `PR_SET_PDEATHSIG` pour ce seul chemin ;
- tous les autres enfants continuent d'utiliser la configuration possédée actuelle.

La raison de l'exception est écrite au point d'appel. Le helper attend la disparition de Beaver, applique la mise à jour puis se termine ; il ne devient jamais un service général détaché.

La copie bornée du helper, `sync_all`, son lancement et la validation d'identité quittent le worker Tokio au moyen de `spawn_blocking`. L'API d'installation devient asynchrone et propage toute erreur sans poursuivre le handoff.

### Preuves

- un test Linux lance un processus parent intermédiaire réel, lui fait créer le helper, termine ce parent et prouve que le helper reste vivant assez longtemps pour accomplir une action témoin ;
- le même test échoue avec la configuration possédée qui arme le signal de mort ;
- les tests existants prouvent toujours qu'un helper non transféré est arrêté et qu'un helper transféré validé est préservé ;
- un test prouve que le travail de copie et de validation ne bloque pas le worker asynchrone.

## Lot 2 — refus d'admission traduits

Les six codes publics existants forment une liste fermée :

- `app-shutting-down` ;
- `app-work-capacity-reached` ;
- `service-shutting-down` ;
- `service-work-capacity-reached` ;
- `gateway-shutting-down` ;
- `gateway-busy`.

Un module TypeScript unique associe ces codes à des clés i18n. Les sept catalogues fournissent les traductions. Un code inconnu retourne toujours le message générique existant et n'est jamais affiché brut.

Le démarrage et la mise en file d'un flux utilisent ce traducteur. Le rendu des erreurs d'outils le consulte avant le fallback générique. Les chemins backend concernés renvoient exclusivement un code stable ; les deux textes français du canal de sous-agents sont remplacés par des codes publics du domaine existant.

### Preuves

- test exhaustif de la liste fermée et du fallback inconnu ;
- test de présence d'une traduction non vide dans les sept langues pour chaque code ;
- tests du démarrage de flux, de la mise en file et du panneau d'outils avec un refus simulé ;
- recherche de régression prouvant qu'aucun des six codes ne peut être rendu directement.

## Lot 3 — scheduler durable

Un réveil arrivé à échéance produit toujours un résultat durable : exécution, absence, annulation ou refus d'admission. Une saturation ou une fermeture n'est plus seulement tracée techniquement.

`WakeupRun` stocke un `error_code` stable et optionnel. L'ancien champ `error` reste accepté en lecture pour les journaux existants, mais aucune nouvelle entrée n'y écrit un texte localisé. Le frontend traduit `error_code` et masque un ancien détail inconnu derrière un message générique.

Le journal des réveils possède une seule serrure pour les lectures, ajouts et rotations. La rotation écrit un fichier temporaire dans le même dossier, le synchronise, puis le renomme atomiquement. La collection reste limitée à 500 lignes et chaque ligne à la taille actuelle.

Le curseur `last_checked` n'avance qu'après la journalisation des décisions de la fenêtre. Si son écriture échoue, le scheduler journalise un code générique borné et conserve une frontière permettant de rejouer sans perdre un réveil. Les réveils ponctuels déjà revendiqués restent idempotents ; les réveils récurrents utilisent l'identité `wakeup_id + scheduled_for` afin qu'un rejeu ne crée pas de doublon.

### Preuves

- saturation et fermeture au moment exact de l'échéance produisent une entrée visible ;
- une panne injectée de `write_last_checked` ne perd ni ne duplique un réveil au passage suivant ;
- lecture et rotation concurrentes ne produisent jamais de JSON partiel ;
- une coupure entre écriture temporaire et renommage conserve l'ancien journal valide ;
- chaque code du journal est traduit dans les sept langues et aucun texte français n'est persisté.

## Lot 4 — gateway réactif et arrêté

Les boucles réseau ne doivent jamais attendre que le consommateur libère de la capacité. Discord, Slack et Telegram utilisent une tentative d'envoi bornée et non bloquante vers la file de 256 messages. Un message refusé pour saturation ou fermeture reçoit une trace d'audit générique bornée ; Discord continue donc à traiter ses heartbeats.

Le jeton d'annulation du run reste accessible sans obtenir le verrou qui protège son handle. `stop_and_wait` annule d'abord le run, puis tente d'acquérir le verrou jusqu'à l'échéance. Un échec d'acquisition n'empêche donc jamais l'annulation du travail actif.

Sous le verrou d'état, l'arrêt se limite à annuler les canaux, écrire `ChannelStatus::Stopping` et prendre un instantané des clés. Les écritures d'audit sont exécutées hors du verrou, sur un worker bloquant, dans le budget restant. Après l'attente des tâches, le service publie `ChannelStatus::Off` et émet l'état final, même si l'audit a échoué ; l'échec d'audit reste un diagnostic générique.

### Preuves

- une file saturée pendant plusieurs intervalles ne retarde pas le heartbeat Discord ;
- Slack et Telegram continuent à lire et à répondre à l'annulation quand la file est pleine ;
- une acquisition du verrou volontairement bloquée reçoit quand même l'annulation avant l'échéance ;
- une écriture d'audit lente ne détient pas le verrou d'état ;
- un arrêt manuel et une fermeture globale terminent tous deux dans l'état `Off`.

## Lot 5 — MCP et extensions bornés

Le pool MCP est l'autorité des processus, pas la serrure de démarrage. Au début de l'arrêt, le service ferme l'admission puis vide le pool sous sa serrure courte, même si le propriétaire d'un spawn est bloqué. Les handles extraits sont ensuite terminés sans garder la serrure.

La fermeture de stdin, le signal de terminaison, l'attente des processus et la réunion des tâches consomment tous la même échéance absolue. Une attente dépassée passe au chemin forcé existant ; aucune sous-opération ne recrée un délai local complet.

L'hôte d'extensions reçoit lui aussi l'échéance absolue. Son receiver `reader_done` n'est retiré qu'après réception réussie ; un timeout le laisse disponible pour la prochaine tentative. Le handle du processus est extrait sous verrou dans le budget, puis arrêté hors verrou. `stop_host` et le registre de travail sont arrêtés dans la même chronologie.

### Preuves

- un propriétaire de spawn MCP bloqué n'empêche pas le drainage et le moissonnage du pool ;
- un verrou stdin bloqué et plusieurs terminaisons lentes ne dépassent pas l'échéance commune ;
- deux appels d'arrêt d'extension après un premier timeout peuvent encore observer `reader_done` ;
- un verrou de processus d'extension bloqué ne dépasse pas le budget partagé ;
- les arrêts répétés restent idempotents et ne fuient aucune admission.

## Lot 6 — SearXNG et Forecast cohérents

Les mutex de processus ne couvrent plus les installations, lectures de fichiers, appels réseau ou boucles de disponibilité.

SearXNG utilise une porte de démarrage distincte pour éviter deux installations concurrentes. Le travail lent se déroule hors du mutex du processus avec un handle `kill_on_drop`. La publication finale du processus est courte et vérifie encore la génération et l'annulation. L'arrêt peut ainsi extraire le processus publié dans son échéance sans attendre toute l'installation.

Forecast photographie sous verrou les données nécessaires au contrôle de santé, relâche le verrou, exécute l'appel bloquant sur un worker borné, puis republie uniquement si l'identité du processus n'a pas changé. L'arrêt extrait le processus sous la même échéance absolue.

La désinstallation Forecast retire d'abord le modèle visible, puis nettoie le runtime familial devenu inutile. Une interruption peut laisser un runtime inutilisé, jamais un modèle visible privé de son moteur. Les écritures de métadonnées utilisent le mécanisme atomique existant.

SearXNG ne lit qu'une queue de journal bornée pour le diagnostic interne. Le frontend reçoit un code générique traduit, sans chemin, version ou fragment de log.

### Preuves

- fermeture pendant chaque phase lente de démarrage SearXNG et Forecast ;
- contrôle de santé Forecast bloqué sans bloquer l'arrêt ;
- changement de génération pendant le contrôle de santé refusé à la republication ;
- interruption injectée à chaque frontière de désinstallation : jamais de modèle visible sans runtime ;
- journal SearXNG très volumineux lu avec une borne fixe et détail absent de l'erreur publique.

## Lot 7 — sondes GPU et bilan Git annulables

Le polling GPU utilise un chemin asynchrone qui possède chaque commande externe via `OwnedProcess`. La sortie est bornée, l'annulation de fermeture termine puis moissonne le processus, et un timeout opérationnel empêche une sonde de vivre au-delà de l'intervalle prévu. Sous Windows, PowerShell et `nvidia-smi` passent par cette même autorité.

Les API synchrones utilisées hors polling peuvent rester en place si elles ne participent pas à la fermeture ; elles ne deviennent pas une seconde autorité pour les processus lancés par le polling.

Une commande shell annulée par la fermeture ne démarre plus un balayage Git complet après avoir moissonné son processus. Elle vide uniquement les événements déjà collectés par le watcher et marque le bilan comme incomplet. Une annulation utilisateur ou une fin normale peut encore demander le scan final existant.

### Preuves

- fermeture pendant une sonde GPU volontairement bloquée : enfant terminé et tâche rejointe ;
- sortie GPU excessive tronquée à la borne définie ;
- Windows prouve que PowerShell est adopté par `OwnedProcess` ;
- un hook de test prouve qu'aucun scan Git final ne démarre après l'annulation de fermeture ;
- fin normale et annulation utilisateur conservent le bilan Git attendu.

## Ordre, erreurs et limites

L'ordre d'exécution recommandé est : helper, admission UI, scheduler, gateway, MCP/extensions, SearXNG/Forecast, GPU/Git. Les deux bloquants sont ainsi fermés en premier et les lots de services partagent ensuite les mêmes conventions déjà testées.

Toute erreur visible utilise une clé i18n ou un code public fermé. Les détails techniques restent dans des traces bornées et filtrées. Toute collection alimentée par des processus, messages ou journaux conserve une capacité explicite. Toute nouvelle attente de fermeture reçoit l'échéance absolue de son appelant.

Les fichiers de code source restent sous 230 lignes. Lorsqu'une responsabilité supplémentaire ferait dépasser cette limite, elle est extraite dans un module du même domaine au lieu de compacter le fichier.

## Validation et critères de fusion

Pour chaque lot :

1. écrire le test de régression ;
2. exécuter le test et conserver la sortie rouge attendue ;
3. appliquer le correctif ;
4. exécuter les tests ciblés et conserver la sortie verte ;
5. créer un commit focalisé et une git note contenant les deux preuves.

La branche n'est proposée à la fusion que lorsque :

- les deux bloquants possèdent des tests non aveugles ;
- les vingt autres constats sont couverts par le code et un test pertinent ;
- les tests frontend, Rust et navigateur ciblés sont verts ;
- `npm run lint`, `npx tsc --noEmit`, `cargo fmt --check`, `cargo check`, Clippy strict et les suites complètes sont verts ;
- Graphify est actualisé après les modifications ;
- les contrôles CI Windows, macOS et Linux sont verts ;
- la plage exacte des commits et leurs git notes est fournie à la re-review.
