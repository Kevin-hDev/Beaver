# Conception — supervision unifiée de la fermeture et des transactions Ollama

## Statut et autorité

Cette conception remplace les décisions incompatibles ou incomplètes des trois documents précédents relatifs à la fermeture :

- `2026-08-08-app-shutdown-lifecycle-design.md` ;
- `2026-08-08-app-shutdown-review-hardening-design.md` ;
- `2026-08-09-shutdown-recovery-hardening-design.md`.

Les mécanismes déjà corrects restent réutilisés, mais le présent document devient la source de vérité lorsqu'un détail diffère. L'objectif est de terminer la série de corrections locales en imposant une autorité unique pour le cycle de vie de Beaver, une autorité unique pour Ollama et des scénarios de validation complets.

## Objectif utilisateur

Une vraie fermeture de Beaver doit terminer l'application et tout ce qu'elle possède, sans processus fantôme, sans installation perdue et sans mise à jour faussement validée.

Le comportement visible est figé ainsi :

- Windows et Linux : la croix de la fenêtre principale lance une vraie fermeture ;
- macOS : la croix rouge masque la fenêtre et conserve l'application active ;
- macOS : `Cmd+Q` et Quitter lancent une vraie fermeture ;
- les trois systèmes : Quitter depuis le tray lance une vraie fermeture ;
- une vraie fermeture arrête le gateway et ses canaux Telegram, Discord et Slack ;
- si sa configuration le demande, le gateway redémarre normalement au prochain lancement ;
- le champ historique `run_when_window_closed` reste accepté lors de la lecture des anciennes configurations, mais ne transforme pas une vraie fermeture Windows ou Linux en exécution cachée ;
- une mise à jour Beaver autorise uniquement le helper validé à survivre temporairement au processus courant.

## Problèmes à résoudre

### Fermeture

- Le délai asynchrone actuel ne peut pas interrompre un appel synchrone bloqué dans sa propre future.
- Les terminaux et Ollama sont arrêtés directement dans le flux asynchrone du nettoyage.
- Le budget Ollama de dix secondes absorbe presque tout le budget global de douze secondes.
- L'admission globale distribue un jeton d'annulation, mais ne garantit pas que le travail admis est enregistré puis attendu.
- Certains redémarrages d'extensions et certaines opérations de mise à jour contournent encore l'admission.
- Le gateway annule ses canaux sans attendre toutes ses tâches supervisées et tous les traitements de messages.
- Le téléchargement d'une mise à jour Beaver n'écoute pas la fermeture.
- Le balayage Unix traite les zombies comme des processus vivants.
- Le menu Afficher du tray ne restaure pas toujours une fenêtre minimisée.
- Le Dock macOS reste visible pendant un vrai Quitter.

### Ollama

- La présence de deux dossiers sert à deviner plusieurs états différents : validation en attente, validation réussie avec nettoyage restant ou récupération interrompue.
- Le résultat booléen du démarrage confond un sidecar Beaver déjà actif avec un démon externe.
- Un démon externe peut répondre à `/api/version` et faire valider par erreur une nouvelle version intégrée qui n'a jamais été lancée.
- La récupération de démarrage n'utilise ni le verrou d'installation ni le registre de travail annulable.
- La suppression impossible d'une ancienne sauvegarde transforme une mise à jour réussie en échec visible.
- L'annulation d'une première installation pendant le premier démarrage laisse le sidecar actif alors que l'écran d'installation reste affiché.
- Les renommages Windows ne réessaient pas les violations de partage temporaires et l'erreur d'origine est perdue.

### Qualité et vérification

- Le test du délai global utilise une future qui cède la main, donc il ne couvre pas un vrai appel bloquant.
- Plusieurs tests Ollama vérifient seulement une classification ou l'état final de dossiers, pas le cycle téléchargement, échange, lancement, validation et récupération.
- Les fonctions de suppression sûre de dossiers internes sont dupliquées.
- `scheduler/mod.rs` dépasse la limite de 230 lignes et porte plusieurs responsabilités.

## Invariants non négociables

1. Le passage de `Running` à `Closing` ferme atomiquement l'admission de nouveau travail.
2. Tout travail long, mutateur ou capable de lancer un processus est soit enregistré et attendu, soit explicitement classé comme opération externe non possédée.
3. Un jeton annulable sans preuve de terminaison ne compte pas comme suivi complet.
4. Aucun appel synchrone potentiellement bloquant n'est exécuté directement dans la future qui porte le délai global.
5. Les budgets internes sont dérivés du budget global restant ; aucun sous-système ne possède un délai supérieur ou presque égal au délai global.
6. Une fermeture normale et un échec métier sont des résultats typés différents.
7. Un abandon forcé ne déclenche jamais une suppression ou un rollback simplement à partir d'un texte d'erreur.
8. Un démon externe n'est jamais arrêté, modifié ni utilisé pour valider le binaire intégré de Beaver.
9. Une mise à jour Ollama validée reste validée même si le nettoyage de l'ancienne sauvegarde échoue.
10. Toute étape Ollama destructive est précédée d'un état durable permettant de reprendre après un arrêt brutal.
11. Le helper de mise à jour Beaver est la seule exception de processus autorisée, après validation de son identité complète.
12. Les collections de tâches, processus et états restent bornées.
13. Les erreurs visibles utilisent des codes traduits ; les détails techniques nettoyés restent dans les journaux.
14. Les chemins macOS, Linux et Windows disposent de tests natifs dédiés ; une validation Windows ne vaut pas validation multi-OS.
15. Une sortie forcée ne doit jamais dépendre du runtime asynchrone, d'un verrou de service ou d'une allocation non bornée.
16. En cas de conflit entre « ne pas tuer un processus externe » et « ne rien laisser tourner », l'identité non vérifiable n'est jamais tuée ; le confinement établi au spawn et le nettoyage du lancement suivant servent de défenses complémentaires.

## Vocabulaire de propriété

### Processus possédé

Un processus est possédé lorsqu'il est créé pour fournir une fonction interne de Beaver et doit disparaître avec lui : Ollama intégré, SearXNG, hôte d'extensions, terminaux, MCP, Forecast et commandes shell d'agent.

Le sous-système qui détient le handle enfant reste responsable de son arrêt normal et de son moissonnage. Le superviseur global conserve seulement une identité bornée pour le filet de sécurité final.

### Processus externe

Un démon Ollama système déjà présent, un navigateur, un éditeur ou une application ouverte par l'utilisateur ne sont pas possédés. Beaver peut les utiliser ou les ouvrir, mais ne les inscrit jamais dans son nettoyage.

### Processus transféré

Le helper de mise à jour Beaver commence comme processus possédé. Après validation de son PID, de son heure de démarrage, de son parent et de son exécutable canonique, il est transféré à `UpdateHandoff`. Il devient alors l'unique exception au balayage final.

## Autorité 1 — superviseur global de cycle de vie

### États

Le superviseur expose un état monotone :

```text
Running -> Closing -> ReadyToExit
```

- `Running` : admission ouverte ;
- `Closing` : admission fermée, jeton global annulé, nettoyage en cours ;
- `ReadyToExit` : Tauri peut quitter sans relancer un nettoyage.

Une nouvelle demande de fermeture pendant `Closing` ne redémarre rien. Une demande après `ReadyToExit` est laissée passer.

### Admission suivie

Le superviseur remplace le simple permis clonable par une admission non clonable associée à une preuve de fin :

- le registre global contient au maximum 128 opérations ;
- les limites métier plus basses continuent de s'appliquer par sous-système ;
- une admission retourne un jeton enfant et un runner/guard de complétion ;
- le runner marque toujours la tâche terminée, y compris après panique ou abandon ;
- le registre refuse toute inscription après `Closing` ;
- le nettoyage ferme le registre, annule les entrées restantes et attend leur preuve de fin dans le budget disponible.

Le registre n'invente aucun identifiant public : une entrée est référencée par son slot borné et sa génération interne. La libération compare le slot et la génération afin qu'un ancien guard ne puisse pas libérer une opération plus récente.

Le registre global suit les opérations de haut niveau. Les services composés, comme le gateway ou le scheduler, conservent un registre interne borné pour leurs propres tâches.

### Inventaire d'adoption obligatoire

Doivent passer par l'admission suivie :

- installation, mise à jour et récupération Ollama ;
- téléchargement et préparation d'une mise à jour Beaver ;
- installation et redémarrage de l'hôte d'extensions ;
- démarrage et traitements supervisés du gateway ;
- téléchargements de modèles ;
- réveils du scheduler ;
- flux agentiques et travaux longs déjà suivis par un registre métier.

Les commandes courtes attendues jusqu'à leur sortie et les applications externes ouvertes pour l'utilisateur sont documentées comme exemptions et ne sont pas artificiellement transformées en services possédés.

### Budgets centralisés

Les constantes vivent dans un seul module de politique de fermeture :

- budget gracieux global : 8 secondes ;
- déclenchement indépendant de sortie Tauri : 10 secondes ;
- balayage final après la boucle Tauri : au maximum 3 secondes ;
- signal d'urgence indépendant aux processus possédés : 13 secondes ;
- ultime sortie processus indépendante : 15 secondes après le début de la fermeture.

Le fonctionnement normal doit rester proche de la mesure actuelle, autour d'une demi-seconde. Ces valeurs sont des plafonds de panne, pas des pauses obligatoires.

Le budget de chaque phase est calculé à partir d'une échéance absolue partagée. Ollama setup ne reçoit jamais plus de 3 secondes de grâce lors d'une fermeture. Une phase terminée tôt rend immédiatement son temps aux phases suivantes. Les relations `8 < 10 < 13 < 15` sont testées et aucune commande ne redéfinit localement ces nombres.

À 8 secondes, le nettoyage coopératif cesse d'attendre et passe au chemin forcé. À 10 secondes, la boucle Tauri reçoit sa sortie même si le nettoyage n'a pas répondu. La fenêtre 10–13 secondes est réservée à CEF et au balayage post-boucle ; la fenêtre 13–15 secondes laisse aux signaux d'urgence le temps d'agir. Aucun de ces plafonds n'ajoute d'attente lorsque la fermeture normale est déjà terminée.

Un arrêt qui dépasse sa grâce peut être interrompu une seule fois, mais ne reçoit jamais une seconde enveloppe du même délai. Toutes les attentes utilisent `deadline - now`, ce qui supprime le comportement actuel où une interruption peut doubler le budget d'un sous-système.

### Deux filets indépendants

Le délai asynchrone reste utile pour le chemin normal, mais il n'est pas l'autorité ultime.

Un watchdog basé sur un thread système indépendant :

1. observe l'état monotone sans dépendre de Tokio ;
2. à 10 secondes, passe atomiquement à `ReadyToExit` si nécessaire et demande la sortie Tauri ;
3. laisse ensuite CEF et le balayage final s'exécuter ;
4. à 13 secondes, ferme le confinement Windows ou envoie un dernier signal borné aux groupes Unix possédés dont l'identité est encore vérifiée ;
5. à 15 secondes, termine le processus Beaver si celui-ci existe encore.

Le watchdog ne prend aucun verrou asynchrone, ne parcourt aucun dossier, ne supprime aucun fichier et ne décide aucun rollback. Il lit seulement l'état atomique et un inventaire d'urgence à capacité fixe. Si l'identité d'un PID ne peut pas être revérifiée, il ne le signale pas au risque de tuer une application externe. Il garantit avant tout que Beaver ne reste jamais invisible et impossible à quitter.

### Confinement établi au lancement

Le filet d'urgence est préparé au moment du spawn, pas improvisé pendant la fermeture :

- Windows : tout enfant possédé créé par Beaver est lancé suspendu, placé dans un Job Object Beaver configuré pour le terminer à la fermeture de son dernier handle, enregistré, puis repris ; CEF conserve son arrêt natif et le balayage direct déjà prévu ;
- Linux : chaque enfant direct possédé reçoit un signal de mort du parent et un groupe de processus dédié avant `exec` ;
- macOS : chaque enfant possédé reçoit un groupe de processus dédié, enregistré dans les slots atomiques du watchdog ;
- toutes les plateformes : le parent vérifie immédiatement que l'identité enregistrée correspond au processus créé ; un échec tue et moissonne cet enfant avant de rendre le spawn visible.

Le helper de mise à jour Beaver est créé selon un chemin distinct. Sous Windows, il est d'abord suspendu dans un Job Object de handoff dédié. La validation et la publication de `UpdateHandoff` lui transfèrent un handle de ce job avant sa reprise : avant ce transfert, une mort de Beaver le termine ; après, le handle conservé par le helper lui permet de finir la mise à jour. Sur Unix, son groupe reste possédé jusqu'au transfert validé. Si le transfert échoue, le helper est arrêté et moissonné comme tout autre enfant.

### Phases de fermeture

1. Fermer l'admission et annuler le jeton global.
2. Masquer toutes les fenêtres. Lors d'un vrai Quitter macOS, masquer aussi l'icône du Dock.
3. Annuler et attendre les opérations enregistrées qui peuvent encore écrire ou lancer un processus.
4. En parallèle, arrêter et attendre scheduler, gateway, extensions, flux, téléchargements, OAuth, shells et MCP.
5. En parallèle, arrêter Forecast, SearXNG et terminaux, puis libérer la VRAM.
6. Arrêter et moissonner Ollama en dernier.
7. Passer à `ReadyToExit` et sortir de la boucle Tauri.
8. Arrêter CEF dans l'ordre natif déjà validé.
9. Signaler les processus possédés encore enregistrés, puis balayer les enfants directs non transférés.
10. Terminer le processus Beaver.

### Frontière synchrone/asynchrone

Les arrêts synchrones de terminaux, Ollama et tout helper basé sur `std::process` s'exécutent sur un exécuteur bloquant dédié. Leur handle est attendu dans le budget commun.

Un timeout ne prétend pas annuler un thread bloquant. Si l'opération dépasse le budget, son processus est ciblé par l'inventaire d'urgence et le watchdog garantit la sortie. Les opérations de fichiers interrompables reposent sur un journal durable plutôt que sur l'espoir que leur future termine.

Les copies, suppressions, synchronisations et renommages de dossiers pouvant durer passent aussi par l'exécuteur bloquant. Une annulation n'est observée qu'avant ou après une frontière durable documentée ; elle ne coupe jamais une mutation atomique au milieu.

## Arrêt attendu des services

### Gateway

Le gateway conserve les handles de ses superviseurs de canaux, de son consommateur de messages et des traitements de messages bornés.

La file d'entrée reste bornée à 256 messages et le registre de traitements simultanés à 64 tâches. Une tâche terminée libère son slot avant que le consommateur n'en admette une autre. La saturation applique une contre-pression ou un refus audité ; elle ne crée jamais un `tokio::spawn` non suivi.

`stop_and_wait` :

1. refuse les nouveaux messages ;
2. annule le jeton de chaque canal ;
3. ferme la file d'entrée ;
4. attend les tâches déjà lancées dans le budget restant ;
5. abandonne les tâches restantes avant de rendre la main ;
6. n'écrit pas de faux échec utilisateur pour une fermeture normale.

Une nouvelle instance de Beaver relit la configuration et redémarre le gateway seulement si `enabled` et `start_with_app` le demandent.

### Extensions

Le runtime d'extensions reçoit le jeton global à son initialisation. `start_and_sync`, `restart` et `ensure_running` vérifient ce jeton sous le même verrou que le slot du processus, avant et après tout spawn.

Toutes les commandes de redémarrage utilisent l'admission suivie. Une exécution d'outil déjà annulée ne peut pas recréer l'hôte après `Closing`.

### Téléchargement de mise à jour Beaver

Le téléchargement :

- obtient une admission suivie avant toute requête ;
- écoute l'annulation pendant le manifeste, la réponse, chaque morceau et les écritures ;
- ferme le fichier avant de supprimer le temporaire ;
- supprime le fichier partiel avant de signaler sa complétion ;
- n'autorise le helper à survivre qu'après le transfert atomique dans `UpdateHandoff`.

La préparation, le spawn, l'attente et le nettoyage du helper qui utilisent `std::process` ou des copies synchrones passent par l'exécuteur bloquant et restent suivis jusqu'au handoff. Le chemin de fermeture n'appelle jamais directement une attente bloquante du helper sur un worker Tokio.

Un nettoyage borné au démarrage inspecte au maximum 256 entrées et supprime au maximum 16 anciens temporaires Beaver par lancement. Il ne cible que le préfixe exact, les fichiers réguliers non symlinkés et âgés de plus de 24 heures. Il s'arrête à la borne, journalise un résumé sans chemin et reprend au lancement suivant ; il ne parcourt jamais tout le dossier temporaire sans borne.

### Terminaux et zombies Unix

`PtySession::kill_and_wait` tue puis moissonne l'enfant. `PtyManager::kill_all_and_wait` retire les sessions sous verrou, relâche le verrou, puis attend chaque enfant hors verrou avec une collection bornée à 16.

Le balayage final ignore les statuts `Zombie` et `Dead`, car ils ne peuvent plus exécuter de code. Il ne boucle pas pour les signaler à nouveau. Les tests Linux vérifient un vrai enfant terminé mais non encore moissonné.

### Tray et macOS

Le clic direct et l'item de menu Afficher partagent une seule fonction : `show`, `unminimize`, puis `focus`.

La croix rouge macOS masque uniquement la fenêtre et ne déclenche ni nettoyage, ni masquage du Dock. Un vrai Quitter masque les fenêtres et le Dock avant le premier travail potentiellement long.

## Autorité 2 — transaction Ollama durable

### Gestionnaire unique

Un seul gestionnaire possède :

- le verrou asynchrone d'installation ;
- le registre de l'opération active ;
- la lecture et l'écriture du journal durable ;
- la préparation, l'échange, la validation, le nettoyage et le rollback ;
- la décision de démarrer ou non le sidecar intégré.

Les commandes d'installation, la commande de mise à jour, le démarrage de l'application et le watchdog Ollama appellent ce gestionnaire. Aucun de ces chemins n'inspecte ou ne supprime directement les dossiers de transaction.

### Journal atomique

Le journal `ollama-update-state.json` vit sous `services::paths::data_dir()`. Il est borné à 4 Kio, versionné et écrit atomiquement par temporaire puis renommage. Avant chaque mutation de dossiers, le fichier temporaire est synchronisé, renommé, puis le dossier parent est synchronisé lorsque la plateforme le permet. Un journal symlinké, non régulier, surdimensionné ou de schéma inconnu bloque la transaction sans suppression.

Les noms sont centralisés dans `services::paths` et désignent uniquement des enfants directs du dossier de données canonique :

- destination active : `ollama-bundle` ;
- staging de première installation : `ollama-bundle-install-staging` ;
- staging de mise à jour : `ollama-bundle-update-staging` ;
- sauvegarde : `ollama-bundle-backup` ;
- cible rejetée pendant un rollback : `ollama-bundle-failed`.

Les deux stagings distincts empêchent une première installation et une mise à jour interrompues de se confondre. Le verrou unique interdit néanmoins leur exécution concurrente.

États durables :

```text
Prepared
PendingValidation
CleanupPending
RollbackPending
RollbackCleanupPending
```

Le schéma conceptuel est exact et refuse les champs inconnus :

```text
TransactionJournal {
  schema_version: 1,
  phase: Prepared | PendingValidation | CleanupPending
       | RollbackPending | RollbackCleanupPending,
  target:   BundleFingerprint { version, executable_sha256 },
  previous: BundleFingerprint { version, executable_sha256 }
}
```

Une mise à jour exige une destination existante et identifiable ; sans version précédente valide, la commande est redirigée vers la réparation ou la première installation et ne crée pas ce journal. Chaque version est une chaîne semver normalisée d'au plus 64 octets et chaque empreinte contient exactement 64 caractères hexadécimaux ASCII. Les empreintes sont comparées en temps constant, octet par octet. Aucun chemin absolu, message d'erreur ou texte extérieur n'est sérialisé. Aucun identifiant aléatoire n'est nécessaire, car le verrou et l'unique journal imposent une seule transaction.

- `Prepared` : staging complet et validé, échange pas encore confirmé ;
- `PendingValidation` : nouvelle installation en place, ancienne sauvegarde conservée ;
- `CleanupPending` : nouvelle version validée, suppression de la sauvegarde à reprendre ;
- `RollbackPending` : restauration de l'ancienne version à reprendre.
- `RollbackCleanupPending` : ancienne version restaurée, suppression de la cible rejetée à reprendre.

L'absence du journal signifie qu'aucune transaction moderne n'est active. Une migration unique interprète prudemment les dossiers hérités produits par la branche actuelle. Toute ambiguïté ferme l'opération sans supprimer de dossier et produit un code récupérable.

### Table de reprise après interruption

La reprise ne devine pas une réussite à partir d'un simple dossier. Elle applique cette table sous le verrou unique :

| Phase durable | État des dossiers attendu ou interrompu | Action de reprise |
|---|---|---|
| Aucun journal | destination seule | état normal ; supprimer plus tard uniquement un staging moderne incomplet reconnu |
| Aucun journal | sauvegarde ou cible rejetée présente | état hérité ambigu ; ne rien supprimer et passer par la migration prudente |
| `Prepared` | destination précédente + staging cible, sans sauvegarde | reprendre l'échange : destination vers sauvegarde, puis staging vers destination |
| `Prepared` | sauvegarde précédente + staging cible, sans destination | terminer staging vers destination |
| `Prepared` | destination cible + sauvegarde précédente, sans staging | écrire `PendingValidation` et valider la destination |
| `Prepared` | destination précédente seule | staging perdu avant l'échange : retirer le journal, conserver l'installation précédente et signaler l'annulation de la mise à jour |
| `Prepared` | sauvegarde précédente seule | staging perdu au milieu de l'échange : restaurer la sauvegarde en destination, retirer le journal et signaler l'annulation |
| `PendingValidation` | destination présente | lancer uniquement la sonde du binaire cible ; succès vers `CleanupPending`, échec certain vers `RollbackPending`, indisponibilité temporaire sans changement |
| `PendingValidation` | destination absente, sauvegarde présente | écrire `RollbackPending` puis restaurer |
| `CleanupPending` | destination cible présente | supprimer la sauvegarde si possible, puis retirer le journal ; un échec garde `CleanupPending` |
| `CleanupPending` | destination absente ou non validable | ne pas supprimer la sauvegarde ; passer à `RollbackPending` si elle existe, sinon exposer une récupération requise |
| `RollbackPending` | destination cible + sauvegarde précédente, sans cible rejetée | déplacer la cible vers le dossier rejeté, restaurer la sauvegarde, puis écrire `RollbackCleanupPending` |
| `RollbackPending` | sauvegarde précédente + cible rejetée, sans destination | terminer la restauration, puis écrire `RollbackCleanupPending` |
| `RollbackPending` | destination précédente + cible rejetée, sans sauvegarde | le rollback est déjà restauré ; écrire `RollbackCleanupPending` |
| `RollbackPending` | destination précédente seule | le rollback et son nettoyage sont déjà terminés ; retirer le journal |
| `RollbackCleanupPending` | destination précédente présente, sans sauvegarde | supprimer la cible rejetée si possible, puis retirer le journal ; un échec conserve cet état |

Chaque qualification « cible » ou « précédente » exige l'empreinte attendue et la version normalisée inscrites dans le journal. Toute combinaison non listée ou empreinte différente est ambiguë : aucune suppression n'a lieu, l'état reste durable et un code public de récupération requise est renvoyé. Les tests coupent artificiellement l'opération avant et après chaque renommage et chaque écriture du journal.

La migration des dossiers hérités est elle-même unique et testée. Destination + sauvegarde devient `PendingValidation` après calcul borné de leurs empreintes ; sauvegarde sans destination devient `RollbackPending` ; destination + cible rejetée sans sauvegarde devient `RollbackCleanupPending`. Toute autre combinaison héritée reste intacte et produit une récupération requise. Le marqueur de migration n'est écrit qu'après la création durable du journal moderne ou la confirmation qu'aucun dossier hérité n'existe.

### Disponibilité selon la phase

- `Prepared`, `PendingValidation` et `RollbackPending` bloquent le démarrage du sidecar intégré hors sonde et exposent l'état précis ;
- `CleanupPending` autorise immédiatement l'utilisation de la cible déjà validée, même si l'ancienne sauvegarde est encore verrouillée ;
- `RollbackCleanupPending` autorise l'utilisation de la version précédente restaurée ;
- une nouvelle mise à jour commence seulement après résolution du nettoyage précédent ; si le système maintient un handle, l'utilisateur reçoit `ollama-update-cleanup-pending`, jamais un faux échec de la mise à jour déjà installée.

Les reprises temporaires suivent une politique centralisée : essais après 5 secondes, 15 secondes, 60 secondes, puis toutes les 5 minutes, avec un seul essai actif et une action manuelle « Réessayer » qui réveille le gestionnaire. La séquence se réinitialise après un progrès de phase. Le statut et le dernier code nettoyé sont visibles, sans chemin ni erreur brute.

### Ordre d'une mise à jour

1. Prendre l'admission suivie puis le verrou unique.
2. Résoudre une transaction précédente avant tout nouveau téléchargement.
3. Télécharger et valider le staging, puis calculer les versions et empreintes de la cible et de l'installation précédente.
4. Écrire `Prepared` atomiquement.
5. Arrêter et moissonner uniquement le sidecar Beaver possédé.
6. Sous Windows, renommer avec une nouvelle tentative bornée uniquement pour les violations de partage temporaires ; journaliser le code système nettoyé sans chemin.
7. Placer la nouvelle installation et écrire `PendingValidation`.
8. Lancer une sonde de validation possédée qui cible explicitement le nouveau binaire et un port isolé.
9. Vérifier que le processus enfant est toujours vivant et que `/api/version` correspond à la version cible attendue.
10. Arrêter et moissonner la sonde.
11. Écrire `CleanupPending` avant de supprimer l'ancienne sauvegarde.
12. Si la suppression réussit, retirer le journal. Si elle échoue, annoncer tout de même la mise à jour comme réussie et réessayer plus tard.

Si la validation échoue de façon certaine, le gestionnaire écrit `RollbackPending` avant le premier renommage de rollback. Il n'efface jamais la destination cible avant que celle-ci ait été déplacée vers le dossier rejeté et que la sauvegarde ait été confirmée comme dossier interne régulier.

Une fermeture entre deux étapes laisse toujours un état et des dossiers suffisants pour reprendre.

### Démon Ollama externe

Le résultat de démarrage n'est plus un booléen. Il distingue au minimum :

```text
OwnedStarted
OwnedAlreadyRunning
ExternalAvailable
RejectedDuringShutdown
Failed
```

La sonde de validation ne lit jamais le port global actuellement utilisé par Beaver. Elle lance le binaire exact situé dans le bundle cible sur un port local isolé, avec un dossier de modèles de sonde isolé, `OLLAMA_NO_CLOUD=1` et une configuration minimale centralisée. Elle conserve son propre handle enfant et ne charge aucun modèle.

La version attendue et la version retournée passent par le même parseur semver strict ; seule une lettre `v` initiale documentée peut être normalisée. Une réponse HTTP n'est acceptée que si l'enfant ciblé est encore vivant avant et après la requête. Un port déjà pris fait sortir cet enfant et provoque au maximum trois essais sur de nouveaux ports, jamais l'utilisation de la réponse du processus occupant.

La présence d'un démon externe ne reporte pas, à elle seule, la validation : la sonde utilise son propre port et son propre dossier. Si la plateforme ou Ollama refuse temporairement la sonde isolée, la transaction reste `PendingValidation`, l'ancienne version reste disponible et l'utilisateur reçoit un code clair de validation différée. Le gestionnaire réessaie au prochain lancement et sur un prochain tick borné. Il ne renvoie pas un verrou générique sans action possible.

### Récupération au démarrage

La récupération :

- obtient une admission suivie ;
- prend le même verrou que les commandes ;
- s'arrête proprement si la fermeture commence ;
- ne supprime jamais la sauvegarde tant que la version cible exacte n'est pas validée ;
- reprend directement le nettoyage si l'état est `CleanupPending` ;
- reprend le rollback si l'état est `RollbackPending` ;
- termine avant que le polling puisse tenter un redémarrage concurrent.

Le polling ne lit plus directement les dossiers avec `unwrap_or(true)`. Il consulte l'état typé du gestionnaire. Une erreur d'inspection est fermée par défaut, journalisée une seule fois par changement d'état et réessayée avec un délai borné.

### Première installation

La première installation utilise exclusivement `ollama-bundle-install-staging`, tandis qu'une mise à jour utilise `ollama-bundle-update-staging`. Après le commit atomique du bundle :

- une fermeture conserve le bundle validé ;
- une annulation utilisateur pendant le premier démarrage arrête et moissonne le sidecar possédé ;
- l'écran relit ensuite `is_ollama_installed` ;
- si le bundle est installé, l'interface quitte l'écran de téléchargement au lieu de prétendre que rien n'existe ;
- une annulation avant le commit supprime uniquement le staging incomplet.

### Fichiers internes partagés

La validation d'un dossier interne, son inspection sans suivi de symlink, sa suppression et les renommages récupérables vivent dans un module commun. Les appelants fournissent leur code public générique, mais n'implémentent plus leur propre variante de `remove_internal_dir`.

Les nouveaux résultats visibles (`validation différée`, `récupération requise`, `fermeture en cours`, `opération annulée`) utilisent des codes stables présents dans le contrat d'erreurs et traduits dans les sept langues. Les logs conservent uniquement la phase, la catégorie d'erreur système nettoyée et les versions non sensibles ; aucun chemin complet ni réponse HTTP brute n'est écrit.

## Autorité 3 — inventaire de processus de secours

Les sous-systèmes restent propriétaires de leurs handles enfants. En complément, chaque processus interne de longue durée enregistre après son spawn :

- son PID ;
- son heure de démarrage ;
- son parent ;
- son type de service ;
- son exécutable canonique lorsque la plateforme le permet.

L'inventaire normal est borné à 128 entrées et retire une entrée quand l'enfant est moissonné. Un échec d'enregistrement tue immédiatement le nouvel enfant et ferme l'opération.

Une vue d'urgence séparée utilise 128 slots atomiques préalloués contenant seulement PID, groupe ou Job, génération et état. Elle n'alloue pas et ne prend pas le mutex du registre normal dans le watchdog. Les métadonnées riches restent dans le registre normal et servent à la revérification d'identité pendant le balayage post-boucle.

Le balayage final traite d'abord cet inventaire, puis les enfants directs découverts par le système. Il revérifie l'identité avant tout signal. Le helper transféré est exclu uniquement si son identité complète correspond encore.

Sur Unix, le signal du groupe possédé part avant les signaux individuels aux descendants et à la racine. Cela ferme d'abord la fenêtre où un parent superviseur pourrait recréer un enfant absent de l'instantané. Les zombies et processus morts sont ensuite ignorés et moissonnés par leur propriétaire lorsque celui-ci est encore disponible.

Cet inventaire n'adopte pas les applications externes ouvertes par Beaver et ne remplace pas l'arrêt normal des services.

## Matrice de scénarios obligatoire

### Fermeture générale

- fermeture normale sans service ;
- fermeture avec chaque service en état `starting`, `running` et `stopping` ;
- opération synchrone factice bloquée au-delà du délai asynchrone ;
- watchdog indépendant qui rend la sortie Tauri possible malgré ce blocage ;
- fermeture du Job Object Windows et signal des groupes Unix avant la sortie ultime ;
- seconde demande de fermeture pendant le nettoyage ;
- aucune nouvelle admission après `Closing` ;
- arrêt CEF puis balayage final dans l'ordre prévu ;
- helper validé seul survivant à une mise à jour Beaver.

### Gateway et extensions

- fermeture avec Telegram, Discord et Slack simulés ;
- file gateway fermée et toutes les tâches attendues ;
- message en cours annulé sans nouveau traitement après `Closing` ;
- redémarrage automatique du gateway au lancement suivant si configuré ;
- redémarrage manuel et automatique de l'hôte d'extensions refusé après `Closing` ;
- aucun host recréé entre `extensions.stop_and_wait` et le balayage final.

### Mise à jour Beaver

- fermeture pendant le manifeste ;
- fermeture au milieu d'un gros téléchargement ;
- fichier partiel supprimé ;
- nettoyage de temporaires limité à 256 inspections et 16 suppressions ;
- fermeture pendant la copie du helper ;
- helper non transféré arrêté et supprimé ;
- helper transféré préservé et nouvelle application relancée.

### Ollama

- première installation annulée avant commit ;
- première installation annulée après commit pendant le démarrage ;
- sidecar arrêté mais bundle conservé et interface actualisée ;
- fermeture à chaque frontière durable de la mise à jour ;
- reprise depuis chacun des cinq états du journal et chaque combinaison de dossiers autorisée ;
- échec de suppression de la sauvegarde après validation ;
- rollback interrompu puis repris ;
- démon externe actif pendant une mise à jour ;
- version externe différente qui ne peut pas valider le bundle ;
- sonde possédée dont la version correspond ;
- sonde possédée dont la version diffère ;
- récupération et commande de mise à jour concurrentes, sérialisées par le même verrou ;
- erreur d'inspection journalisée puis réessayée ;
- renommage Windows temporairement verrouillé puis réussi ;
- renommage Windows définitivement verrouillé, sans perte des deux versions.

### Processus et plateformes

- terminal tué puis moissonné ;
- zombie Linux ignoré sans boucle de trois secondes ;
- ordre groupe, descendants, racine sous Unix ;
- `taskkill` Windows conserve ses arguments séparés et son chemin système validé ;
- clic tray et item Afficher restaurent une fenêtre minimisée ;
- croix macOS masque sans nettoyage ;
- vrai Quitter macOS masque le Dock avant le nettoyage.

## Stratégie de mise en œuvre

Le travail est découpé par autorité, avec un test rouge et un commit révisable par comportement :

1. Tests de contrat et superviseur global suivi.
2. Frontière bloquante, budgets et watchdog indépendant.
3. Adoption gateway et extensions.
4. Téléchargement/helper de mise à jour Beaver.
5. Journal et gestionnaire unique Ollama.
6. Sonde Ollama possédée et migration des chemins de démarrage/polling.
7. Première installation, annulation et interface.
8. Inventaire de processus, terminaux, zombies et détails plateforme.
9. Nettoyage structurel du scheduler et des utilitaires de fichiers.
10. Validation globale et tests manuels.

Après chaque lot, la relecture porte aussi sur le diff complet depuis `main`, pas seulement sur le dernier commit.

## Revue globale obligatoire avant fusion

La revue finale doit produire cinq inventaires explicitement vérifiés :

1. Tous les chemins qui créent un processus, classés en possédé, externe, court ou transféré.
2. Tous les travaux asynchrones longs ou mutateurs, avec leur admission, annulation et preuve de fin.
3. Tous les appels synchrones atteignables pendant une fermeture, avec leur frontière bloquante et leur borne.
4. Toutes les transitions Ollama, avec l'état durable avant et après chaque mutation de fichiers.
5. Tous les accès au journal ou aux dossiers Ollama, qui doivent aboutir au gestionnaire unique et au même verrou.

Elle vérifie également :

- le diff complet de la branche contre `main` ;
- les comportements existants de CEF, gateway, scheduler, SearXNG, MCP, Forecast et mise à jour ;
- l'appel à `scheduler.notify_config_changed()` après chaque mutation de réveil ;
- les textes visibles dans les sept langues ;
- les fichiers de production sous 230 lignes ;
- l'absence de collection externe non bornée ;
- l'absence de `tokio::spawn`, `std::thread::spawn` ou spawn de processus longue durée sans propriétaire, limite et chemin d'arrêt documentés ;
- l'absence de chemin, secret ou erreur brute dans l'interface et les journaux ;
- la mise à jour Graphify après le code et la documentation.

## Validation finale

- tests unitaires ciblés après chaque étape ;
- suite Rust complète séquentielle avec `windows-tests` ;
- suite Rust parallèle pour détecter les états globaux ou tests instables ;
- `cargo fmt --check` ;
- Clippy strict sur tous les targets ;
- TypeScript, lint et totalité des tests frontend ;
- tests des scripts de build, du runner E2E, de CEF et de l'hôte d'extensions ;
- CI native Windows, Ubuntu et macOS ;
- test manuel d'une vraie fermeture sur chaque système à partir du build natif ;
- contrôle après fermeture de l'absence de processus Beaver possédé ;
- tests manuels d'une mise à jour Beaver et d'une mise à jour Ollama interrompue.

## Critères d'acceptation

- Beaver ne peut pas rester invisible en état `Closing` au-delà du délai absolu.
- Une vraie fermeture arrête le gateway et tous les processus possédés.
- La croix macOS ne ferme pas Beaver.
- Aucun service ne redémarre après la fermeture de l'admission.
- Aucun gros téléchargement partiel Beaver ne survit à une annulation normale.
- Une mise à jour Ollama ne peut être validée que par le binaire cible possédé et la version attendue.
- Un démon Ollama externe n'est jamais arrêté ni utilisé comme preuve de validation.
- Une suppression de sauvegarde échouée n'annule pas une mise à jour déjà validée.
- Toute transaction Ollama interrompue possède un chemin automatique de reprise.
- Aucun test de délai ne confond une future coopérative avec un appel bloquant.
- Les suites et builds des trois systèmes réussissent avant fusion.
