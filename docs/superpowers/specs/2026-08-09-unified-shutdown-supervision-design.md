# Conception — supervision unifiée de la fermeture et des transactions Ollama

## Statut et autorité

Cette conception remplace les décisions incompatibles ou incomplètes des trois documents précédents relatifs à la fermeture :

- `2026-08-08-app-shutdown-lifecycle-design.md` ;
- `2026-08-08-app-shutdown-review-hardening-design.md` ;
- `2026-08-09-shutdown-recovery-hardening-design.md`.

Les mécanismes déjà corrects restent réutilisés, mais le présent document devient la source de vérité lorsqu'un détail diffère. Il constitue le contrat transverse : les invariants, décisions produit et frontières qu'il contient ne peuvent pas être modifiés par un jalon d'implémentation sans un nouvel amendement approuvé.

L'implémentation est décomposée en cinq spécifications et cinq PR reliées par un graphe de dépendances :

1. [Jalon 1 — socle de fermeture](./2026-08-09-shutdown-milestone-1-core-design.md) ;
2. [Jalon 1B — supervision native de CEF](./2026-08-09-shutdown-milestone-1b-cef-design.md) ;
3. [Jalon 2 — processus et services](./2026-08-09-shutdown-milestone-2-services-design.md) ;
4. [Jalon 3 — transaction Ollama](./2026-08-09-shutdown-milestone-3-ollama-design.md) ;
5. [Jalon 4 — convergence multi-OS](./2026-08-09-shutdown-milestone-4-convergence-design.md).

L'[inventaire de reprise de la branche de référence](./2026-08-09-shutdown-reference-branch-inventory.md) rattache séparément les 22 commits de code existants à ces jalons. Il fait partie du contrat : une correction de la grande branche ne peut pas disparaître simplement parce que cette branche n'est pas fusionnée.

Le jalon 1 part de `main`. Après sa fusion, les jalons 1B et 2 peuvent avancer indépendamment depuis ce même socle ; le jalon 3 dépend du jalon 2, puis le jalon 4 réunit les jalons 1B et 3 déjà fusionnés. La branche `codex/fix-app-shutdown-lifecycle`, devenue trop large, reste une sauvegarde et une source de référence ; elle n'est pas fusionnée telle quelle.

## Objectif utilisateur

Une vraie fermeture de Beaver doit terminer l'application et tout ce qu'elle possède, sans processus fantôme, sans installation perdue et sans mise à jour faussement validée.

Le comportement visible est figé ainsi :

- Windows et Linux : la croix de la fenêtre principale lance une vraie fermeture ;
- macOS : la croix rouge masque la fenêtre et conserve l'application active ;
- macOS : `Cmd+Q` et Quitter lancent une vraie fermeture ;
- les trois systèmes : Quitter depuis le tray lance une vraie fermeture ;
- une vraie fermeture arrête le gateway et ses canaux Telegram, Discord et Slack ;
- si sa configuration le demande, le gateway redémarre normalement au prochain lancement ;
- le champ historique `run_when_window_closed` est retiré des modèles Rust et TypeScript, des réglages visibles et des nouvelles écritures ; les anciens fichiers qui le contiennent restent lisibles grâce à la tolérance aux champs inconnus, mais sa valeur n'est plus utilisée ;
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
17. Chaque jalon reste fusionnable sans activer un mécanisme incomplet et conserve les protections existantes tant que leur remplacement n'est pas totalement adopté.
18. Les 22 lignes de l'inventaire de reprise sont toutes fermées par une correction testée ou un abandon explicitement justifié avant la fusion du jalon 4.
19. Toute version livrée sur Windows ou macOS active CEF avec une supervision native prouvée et le sandbox réel ; un échec local de prérequis peut rendre le navigateur indisponible avant tout lancement de helper sur la machine concernée, mais cet état de défense n'est jamais un état de livraison accepté.
20. L'intégration de CEF sous Linux reste désactivée et hors de ce chantier ; l'activer et la terminer fera l'objet d'une conception et d'une implémentation séparées.

## Vocabulaire de propriété

### Processus possédé

Un processus est possédé lorsqu'il est créé pour fournir une fonction interne de Beaver et doit cesser tout travail avec lui : Ollama intégré, SearXNG, hôte d'extensions, terminaux, MCP, Forecast, commandes shell d'agent et sous-processus du navigateur CEF intégré sur Windows et macOS. La preuve immédiate porte sur l'absence de processus possédé encore runnable ; un objet noyau déjà en terminaison ou zombie doit ensuite disparaître dans la fenêtre de constat native de 5 secondes. Le build Linux actuel n'active pas `native_browser` et ne crée donc aucun helper CEF. Les sous-processus de WebView créés indirectement par Tauri sont classés par le jalon 2 à partir des builds natifs : un descendant dédié à Beaver est possédé, tandis qu'un service système partagé reste externe.

Le sous-système qui détient le handle enfant reste responsable de son arrêt normal et de son moissonnage. Le superviseur global conserve seulement une identité bornée pour le filet de sécurité final.

### Processus externe

Un démon Ollama système déjà présent, un navigateur externe, un éditeur, un service WebView réellement partagé par le système ou une application ouverte par l'utilisateur ne sont pas possédés. Beaver peut les utiliser ou les ouvrir, mais ne les inscrit jamais dans son nettoyage.

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

La première demande enregistre aussi un intent immuable `Exit` ou `Restart`. Une nouvelle demande pendant `Closing` ne redémarre rien et ne peut pas remplacer cet intent. Une demande après `ReadyToExit` est laissée passer. L'intent `Restart` suit exactement le même nettoyage que `Exit`, puis demande la relance Tauri seulement après `ReadyToExit`. Si le tueur ultime force la sortie, la sécurité gagne et aucune relance n'est promise.

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

Le jalon 1 expose déjà le nombre actif nécessaire à sa preuve de fermeture. Pendant l'adoption du jalon 2, avant de brancher les producteurs, le registre global et chaque registre métier ajoutent uniquement des compteurs atomiques de taille fixe : admissions actives, maximum atteint depuis le démarrage, refus pour saturation et refus après `Closing`. Ces compteurs alimentent les diagnostics locaux sans identifiant, contenu utilisateur ni télémétrie distante. Les tests de chaque producteur prouvent la libération du slot sur succès, erreur, annulation, panique et abandon, puis répètent assez de cycles pour dépasser largement la capacité cumulée sans provoquer de fausse saturation.

### Inventaire d'adoption obligatoire

Doivent passer par l'admission suivie :

- installation, mise à jour et récupération Ollama ;
- téléchargement et préparation d'une mise à jour Beaver ;
- installation et redémarrage de l'hôte d'extensions ;
- démarrage et traitements supervisés du gateway ;
- installation, démarrage et arrêt de SearXNG ;
- serveurs MCP stdio, processus d'installation MCP et leurs superviseurs ;
- installation du runtime, sidecar, évaluations et commandes longues de Forecast ;
- création des terminaux PTY, processus shell et threads lecteurs associés ;
- téléchargements de modèles ;
- réveils du scheduler ;
- flux agentiques, sous-agents, commandes shell et travaux longs déjà suivis par un registre métier ;
- initialisation et cycle de vie du CEF intégré sur Windows et macOS ; ses sous-processus sont enregistrés dans l'inventaire même s'ils sont créés par la bibliothèque native et non par un `Command` Rust de Beaver.

Les processus créés indirectement par Tauri, WebView2, WebKit ou une autre bibliothèque native n'échappent pas à l'inventaire. Le jalon 2 les observe sur chaque OS, les classe avec des preuves de parenté et de partage, puis adopte uniquement les descendants dédiés à Beaver.

Les commandes courtes attendues jusqu'à leur sortie et les applications externes ouvertes pour l'utilisateur sont documentées comme exemptions et ne sont pas artificiellement transformées en services possédés.

### Budgets centralisés

Les constantes vivent dans un seul module de politique de fermeture :

- budget gracieux global : 8 secondes ;
- déclenchement indépendant de sortie Tauri : 10 secondes ;
- balayage final après la boucle Tauri : au maximum 3 secondes ;
- signal d'urgence indépendant aux processus possédés : 13 secondes ;
- barrière d'admission CEF au passage à `Closing` : au maximum 50 millisecondes ;
- cadence de revérification CEF pendant la phase forcée : constante centrale de 10 millisecondes ;
- auto-terminaison de secours d'un helper CEF macOS : 14 secondes, soit une marge centrale de 1 seconde avant la sortie brute ;
- ultime sortie processus indépendante : 15 secondes après le début de la fermeture.

Le fonctionnement normal doit rester proche de la mesure actuelle, autour d'une demi-seconde. Ces valeurs sont des plafonds de panne, pas des pauses obligatoires.

Les tests natifs disposent ensuite d'une fenêtre de constat séparée de 5 secondes pour vérifier que les objets `terminating` ou zombies ont disparu. Cette fenêtre appartient au processus de test : elle ne retarde jamais la fermeture de Beaver et n'autorise aucun travail applicatif après 15 secondes.

Le budget de chaque phase est calculé à partir d'une échéance absolue partagée. Ollama setup ne reçoit jamais plus de 3 secondes de grâce lors d'une fermeture. Une phase terminée tôt rend immédiatement son temps aux phases suivantes. Les relations `8 < 10 < 13 < 14 < 15` sont testées et aucune commande ne redéfinit localement ces nombres.

À 8 secondes, le nettoyage coopératif cesse d'attendre et passe au chemin forcé. À 10 secondes, la boucle Tauri reçoit sa sortie même si le nettoyage n'a pas répondu. La fenêtre 10–13 secondes est réservée à CEF et au balayage post-boucle ; la fenêtre 13–15 secondes est une phase forcée continue, pas un passage unique. Le watchdog y traite les slots généraux qui apparaissent tardivement et revérifie les terminaisons déjà demandées ; une publication CEF tardive reste invalidée par la barrière. Aucun de ces plafonds n'ajoute d'attente lorsque la fermeture normale est déjà terminée.

Un arrêt qui dépasse sa grâce peut être interrompu une seule fois, mais ne reçoit jamais une seconde enveloppe du même délai. Toutes les attentes utilisent `deadline - now`, ce qui supprime le comportement actuel où une interruption peut doubler le budget d'un sous-système.

### Filets indépendants

Le délai asynchrone reste utile pour le chemin normal, mais il n'est pas l'autorité ultime. Le tueur ultime est créé et validé au démarrage, avant d'afficher une fenêtre ou de lancer un service ; un échec de création empêche Beaver de démarrer. Il reste ensuite garé sur un état atomique préalloué, sans canal ni allocation tardive. Sa fonction ne contient aucune opération susceptible de paniquer ; une garde de dernier recours transforme malgré tout toute panique en arrêt brut immédiat. Au passage à `Closing`, Beaver arme ce thread déjà vivant et démarre séparément le watchdog de processus :

1. le watchdog de processus observe l'état monotone sans dépendre de Tokio ; à 10 secondes, il passe atomiquement à `ReadyToExit` si nécessaire et demande la sortie Tauri ; de 13 secondes jusqu'à l'échéance, il ferme le confinement général et draine les slots d'urgence ;
2. le tueur ultime précréé ne parcourt aucun slot et n'appelle aucun nettoyage : une fois armé, il attend uniquement l'échéance monotone de 15 secondes, puis appelle `TerminateProcess(GetCurrentProcess(), code)` sous Windows ou `libc::_exit(code)` sous macOS/Linux si Beaver existe encore.

Le tueur ultime ne dépend jamais du retour ni même de la création du watchdog de processus. Le watchdog vérifie le temps restant avant chaque slot et cesse d'initier un appel dès que l'échéance est atteinte ; un appel OS déjà bloqué ne peut donc pas retarder la sortie brute, exécutée par l'autre thread. Un test d'injection couvre l'échec de création du watchdog ; un autre simule l'échec initial du tueur ultime et exige que le démarrage soit refusé avant tout effet de bord.

Sur macOS avec CEF activé, un troisième thread système, distinct du traqueur normal, est créé et validé avant `cef::initialize` et constitue une condition de cette initialisation. Il possède son propre contrôle atomique et lit directement une table d'urgence parent privée à capacité fixe ; la fonction d'urgence ne se contente jamais de poser un drapeau destiné au traqueur normal. Après la barrière de fermeture, il rescane jusqu'à 15 secondes les 64 slots privés dont les générations admises sont désormais stables. Avant chaque `SIGKILL`, ce reaper compare avec `proc_pidinfo`, `proc_pidpath` et `getpgid` le PID leader, son parent, son heure de démarrage, son exécutable canonique et son groupe ; une identité absente, réutilisée ou ambiguë n'est jamais signalée. Les buffers sont fixes, aucun mutex ou tas n'est utilisé et `kill(2)` ne reçoit qu'un groupe dont le leader correspond encore à l'identité admise. Ce chemin reste actif si le traqueur normal panique ou se bloque, et demeure indépendant du watchdog général et du tueur ultime. Si le reaper ne peut pas être créé ou s'arrête de façon inattendue, CEF n'est pas initialisé ou sa porte est fermée et Beaver déclenche immédiatement la sortie coordonnée. Le tueur ultime reste séparé et n'attend jamais ce reaper.

Le watchdog de processus ne prend aucun verrou asynchrone, ne parcourt aucun dossier, ne supprime aucun fichier et ne décide aucun rollback. Il lit seulement l'état atomique et un inventaire d'urgence à capacité fixe. Sous Linux, la revérification lit `/proc/<pid>/stat` avec `open/read` dans un buffer de pile fixe ; sous macOS, elle utilise `proc_pidinfo` dans une structure de pile fixe. Les deux chemins comparent PID, groupe et heure de démarrage sans `sysinfo`, allocation de tas ou mutex. Si l'identité ne peut pas être revérifiée, le PID n'est pas signalé au risque de tuer une application externe.

CEF constitue un cas particulier sur Windows et macOS : sa bibliothèque crée elle-même les processus qui exécutent le helper canonique configuré par Beaver. Sous Windows, ce helper partage l'exécutable de Beaver avec d'autres rôles, notamment le bac à sable shell ; le nom de l'exécutable ne suffit donc jamais à l'identifier.

Avant `cef::initialize`, Beaver crée une table d'autorité parent privée à 64 slots et une porte atomique `CefLaunchGate`. Seul le processus parent peut écrire les états `Reserved`, `Published`, `Admitted`, `Terminating` ou `Exited`, les générations, les handles, les Jobs et les groupes. Aucune de ces autorités n'est placée dans une zone modifiable par un helper. Cette capacité dédiée ne concurrence pas les 128 slots des autres processus et reste cohérente avec les dix onglets maximum du navigateur.

Chaque réservation reçoit une boîte de publication distincte et une page de contrôle distincte, toutes deux de taille fixe et nommées par des nonces CSPRNG de 256 bits jamais loggés. La boîte, seule zone modifiable par le helper, ne contient que la génération publiée, le PID, l'heure de démarrage et, sur macOS, le groupe proposé. La page de contrôle est modifiable seulement par le parent et expose en lecture la génération ainsi que l'échéance de fermeture en ticks `u64` de la même horloge monotone native dans les deux processus ; schéma, unité et bornes sont validés avant usage. Deux événements distincts, eux aussi modifiables seulement par le parent, permettent uniquement d'attendre l'admission et la fermeture. Aucun de ces objets ne contient un handle ou un état d'autorité, et le parent traite chaque octet de la boîte comme non fiable. Un helper reçoit seulement les identifiants de son slot ; les nonces d'un autre slot ne lui sont jamais transmis et une corruption de sa boîte ne peut pas modifier l'autorité parent ni l'admettre. Tous les noms et handles sont invalidés à la libération de la génération, puis les buffers des nonces sont zéroïsés.

Sous Windows, ces objets utilisent des handles non héritables et des descripteurs construits pour le profil de jeton initial réellement configuré par CEF. La DACL accorde seulement les droits minimaux nécessaires aux SIDs activés et aux SIDs de restriction de ce jeton, et la SACL porte le niveau d'intégrité explicite requis, jusqu'au niveau `Untrusted` pour les types qui l'utilisent. Le parent garde seul `EVENT_MODIFY_STATE` et l'écriture de la page de contrôle ; le helper reçoit uniquement `SYNCHRONIZE` sur les événements, la lecture de la page de contrôle et l'écriture bornée de sa boîte. La conception tient donc compte des deux contrôles d'accès d'un jeton restreint et du contrôle d'intégrité obligatoire, au lieu de supposer qu'une DACL utilisateur suffit. Si un type CEF sandboxé ne peut pas publier avec ces droits minimaux, le sandbox n'est jamais désactivé et les permissions ne sont jamais élargies par repli.

Le repli CEF est fermé et pré-approuvé uniquement comme défense d'exécution sur une machine défaillante. Avant `cef::initialize`, tout échec local de création de la supervision sélectionne `BrowserCapability::Unavailable` et Beaver continue sans lancer de helper. Une erreur déterministe échoue immédiatement. Une erreur transitoire reconnue par son code OS reçoit exactement une seconde prévalidation propre après une attente centrale de 200 millisecondes ; aucune décision ne dépend d'un texte d'erreur et aucun objet de la tentative précédente n'est réutilisé. Une release Windows ou macOS n'est toutefois acceptée que si le chemin normal `Ready supervisé` est prouvé nativement avec le sandbox réel ; une plateforme entière livrée en indisponibilité n'est pas un troisième résultat acceptable. Une simple identification par nom, PID ou callback parent n'est pas un mode de secours. Après qu'un helper a potentiellement été créé, une ambiguïté déclenche l'arrêt du bootstrap avant CEF ou, si la porte était déjà ouverte, une vraie fermeture coordonnée ; elle ne peut pas être masquée en indisponibilité silencieuse.

`cef::initialize` reste un appel unique par processus Beaver. S'il retourne faux, aucune autre fonction CEF n'est appelée, y compris `cef::shutdown`, et Beaver lance une fermeture coordonnée. La seule nouvelle tentative sûre est un redémarrage complet du processus.

Quand la prévalidation sélectionne `BrowserCapability::Unavailable`, le backend émet un code public stable sans détail système. L'interface réutilise le composant de notification existant et l'étend avec une action optionnelle générique. La notification traduite dans les sept langues est non modale, fermable, n'empêche aucune autre action et disparaît entièrement après une constante centrale de 10 secondes ; son bouton `Redémarrer` disparaît avec elle et aucun contrôle permanent n'est ajouté au navigateur. Cette action demande l'intent `Restart` au coordinateur au lieu d'appeler directement le redémarrage Tauri.

Sous Windows, un routeur de rôle unique précède toute branche CEF et classe le processus en parent Beaver, helper CEF réservé, helper d'isolation du shell ou entrée invalide. Les rôles sont mutuellement exclusifs ; `--beaver-shell-sandbox` atteint le helper shell réel en développement comme dans le build empaqueté, tandis qu'une combinaison shell + marqueur CEF est refusée. Le classificateur parcourt la ligne de commande sans la recopier et borne uniquement les arguments privés Beaver qu'il décode. Il ne rejette pas une ligne Chromium à cause d'un plafond arbitraire de 64 arguments ; le transfert du bootstrap de développement reste borné par les 32 767 unités UTF-16 autorisées par `CreateProcess`.

Tant que la porte est ouverte, `on_before_child_process_launch` réserve atomiquement un slot avant le lancement et ajoute un marqueur privé de rôle CEF, de format et longueur bornés, contenant seulement le slot, sa génération et les identifiants aléatoires propres à cette réservation. Si le callback ne reçoit pas de ligne de commande exploitable, si l'ajout du marqueur échoue ou si la réservation ne peut pas être publiée, il invalide sa génération, ferme la porte et fait échouer l'initialisation CEF ; aucun lancement n'est admis par défaut. Le bootstrap du helper valide strictement le format, la réservation, la génération, le parent et l'exécutable, puis écrit son identité dans sa seule boîte de publication. Il n'appelle pas encore `cef::execute_process` : il attend de façon bornée l'événement d'admission que seul le parent peut signaler. Un processus portant `--beaver-shell-sandbox`, un marqueur absent, une réservation inconnue ou une génération périmée ne peut jamais être adopté comme CEF ni appeler CEF par un chemin de repli.

Le traqueur revendique une seule fois la publication de la génération, la copie dans sa table privée, la valide puis scelle le slot : toute réécriture ultérieure de la boîte est ignorée et ne peut jamais remplacer l'identité copiée. Il prend ensuite un permis dans un compteur atomique associé au bit de fermeture, acquiert l'identité stable et l'inscrit dans la vue d'urgence avant de passer son slot privé à `Admitted` et de signaler l'événement d'admission. La prise du permis est le point d'admission linéarisé et échoue si la porte est déjà fermée. Le passage `Running -> Closing` pose d'abord le bit fermé, écrit l'échéance absolue dans chaque page de contrôle et signale l'événement de fermeture, puis attend les permis déjà pris jusqu'à l'échéance absolue centrale de 50 millisecondes. Il invalide ensuite toute génération qui n'est pas déjà `Admitted` avant de masquer les fenêtres. Aucun nouveau permis ne peut commencer après ce bit et aucun ancien permis ne peut publier `Admitted` après cette invalidation : il nettoie ses objets puis termine le bootstrap qu'il venait de vérifier. La liste des admissions est donc stable avant le début du nettoyage. Un dépassement de la barrière est journalisé sans détail sensible et le nettoyage coordonné continue ; il ne devient jamais une violation d'invariant ni une sortie brute anticipée. Seule une corruption réelle de l'autorité monotone emprunte ce filet fatal. Un traqueur interrompu avant `Admitted` ne laisse pas entrer le helper dans CEF.

Un callback ultérieur reçoit un marqueur de refus et ne réserve aucun slot. Un helper issu d'une réservation antérieure sort immédiatement avant `cef::execute_process` si l'événement de fermeture est signalé, si l'admission n'arrive pas dans sa borne, si ses objets ont disparu ou si sa génération n'est plus valide. Il relit l'événement de fermeture après avoir observé l'admission et juste avant l'appel CEF. Ainsi, un lancement retardé peut tout au plus créer un processus de bootstrap voué à sortir ; toute course déjà admise possède sa preuve d'arrêt avant de pouvoir entrer dans CEF et reste traitée par la phase forcée. Aucun thread supplémentaire n'est créé dans le helper avant l'initialisation du sandbox CEF.

Ce bootstrap pré-admission est un candidat transitoire, pas un service CEF admis. Si le système ne lui accorde du temps CPU qu'après la sortie du parent, il peut être brièvement runnable pour ouvrir le registre, constater son absence et sortir ; il ne charge pas CEF, n'ouvre aucun profil, ne consomme pas la VRAM et ne lance aucun descendant. Le critère immédiat porte donc sur les services et helpers `Admitted`. Le test place aussi volontairement un candidat dans cette fenêtre et exige sa disparition dans les 5 secondes de constat, avec preuve que `cef::execute_process` n'a jamais été appelé.

Un traqueur natif démarre avant `cef::initialize`, adopte uniquement les publications validées par sa table privée et les rafraîchit jusqu'à la sortie effective de Beaver. Sous Windows, il ouvre un handle avec `SYNCHRONIZE`, `PROCESS_TERMINATE`, `PROCESS_QUERY_LIMITED_INFORMATION` et `PROCESS_SET_QUOTA`, vérifie l'identité, crée pour ce slot un Job Object vide sans restriction d'interface avec `KILL_ON_JOB_CLOSE`, puis y affecte uniquement ce helper avant `Admitted`. Un Job distinct par helper respecte la contrainte des jobs imbriqués lorsque le sandbox Chromium a déjà placé le processus dans son propre job ; ce point reste obligatoirement validé avec le sandbox actif dans le smoke test natif. Les handles du processus et du Job ne sont ni héritables ni dupliqués dans le helper : le slot parent en possède les seules copies, afin que la sortie brute ferme réellement le dernier handle du Job. L'échec d'ouverture, de vérification, d'affectation ou de publication du slot ferme les guards locaux, tue le bootstrap et refuse l'admission.

Sous macOS, le point d'entrée valide d'abord le marqueur, ouvre et mappe uniquement sa boîte, sa page de contrôle et ses événements, puis crée son groupe dédié, le tout avant `sandbox.initialize`. Aucun thread n'est encore créé et aucun appel CEF n'est effectué. Après l'application réussie du sandbox, le helper publie son PID, son parent, son heure de démarrage et ce groupe ; le parent les revérifie avant `Admitted` et les inscrit dans le slot privé du reaper macOS. Le helper démarre alors, toujours avant `cef::execute_process`, un moniteur minimal. Il compare de façon bornée `getppid()` au parent validé et appelle `_exit` dès que ce parent disparaît ou change. Quand l'événement de fermeture est réellement signalé par le chemin de production, il lit l'échéance absolue réellement écrite sur la page de contrôle en lecture seule et s'auto-termine au plus tard une seconde avant la sortie brute ; une page absente, invalide ou déjà expirée provoque une sortie immédiate. Il ne tente pas de signaler le groupe depuis le sandbox, dont la politique peut limiter le signal à soi-même. Ce moniteur n'utilise ni Tokio, ni tas dans sa boucle, ni objet partagé modifiable, et un échec de création fait sortir le bootstrap avant CEF. Le reaper parent précréé reste l'autorité indépendante qui signale, après revérification, le groupe si le watchdog général se bloque ; le moniteur enfant apporte une seconde coupure par helper. Le smoke test natif prouve cet ordre pré-sandbox/post-sandbox, l'échéance d'auto-terminaison, l'écriture de fermeture en production et l'absence de descendants CEF échappant au groupe ; son échec bloque la fusion du jalon 1B et toute release concernée au lieu de désactiver la fonctionnalité ou d'affaiblir le sandbox.

Une saturation, une publication invalide ou une identité ambiguë fait échouer l'initialisation si elle est encore en cours, sinon elle ferme la porte et déclenche immédiatement une vraie fermeture coordonnée. Beaver ne continue jamais avec un enfant CEF connu mais non supervisé.

Les handles Windows du processus et du Job Object du slot sont dédiés ; le reste de l'application ne les utilise jamais. Le retrait normal et la terminaison forcée revendiquent atomiquement le slot avec sa génération ; seul le gagnant les emploie et les ferme. Après `TerminateProcess`, le slot reste en état `Terminating` et le watchdog appelle `WaitForSingleObject(process_handle, 0)` à chaque passage. Le handle processus n'est fermé qu'après le signal de fin ; sur le chemin forcé, le handle du Job est fermé pour déclencher `KILL_ON_JOB_CLOSE`. Cette propriété empêche à la fois de considérer trop tôt l'arrêt comme terminé et qu'une fermeture concurrente transforme une valeur de handle réutilisée en un autre objet.

Le thread du traqueur est lui-même possédé. Sur le chemin normal, il effectue un dernier rafraîchissement après le retour de CEF, retire les helpers terminés, ferme ses handles puis est joint avant le balayage final. Si CEF bloque ou si le traqueur tombe, le watchdog lit directement la table parent privée et peut revalider une proposition encore présente dans la boîte du slot sans lui faire confiance. Sous Windows, un slot publié sans handle lui permet d'appeler `OpenProcess` avec `SYNCHRONIZE`, `PROCESS_TERMINATE`, `PROCESS_QUERY_LIMITED_INFORMATION` et `PROCESS_SET_QUOTA`, puis de vérifier le parent, l'heure de création, l'exécutable et la réservation avant tout signal. Un slot non admis n'entre jamais dans CEF ; un slot admis possède déjà sa preuve d'arrêt et son confinement. Le watchdog ne dépend donc pas du traqueur pour fermer un helper existant.

`on_before_child_process_launch` réveille aussi le traqueur sans lui transmettre de donnée non validée. Une réconciliation native peu fréquente et un dernier instantané borné par la limite centrale des processus couvrent une notification manquée ; le suivi ne reparcourt pas en boucle non bornée toute la machine. Une réservation sans publication expire de façon bornée : sa génération est invalidée avant réutilisation, et un helper retardé qui la présente sort avant CEF.

Si `cef::shutdown()` bloque le thread principal, le balayage placé après cet appel ne peut pas servir de garantie. De 13 secondes jusqu'à l'échéance, le watchdog de processus relit les slots CEF fixes déjà `Admitted`, ferme une fois leur Job Object et appelle une fois `TerminateProcess` sous Windows puis observe le signal par sondages non bloquants, ou envoie `SIGKILL` après revérification sous macOS puis confirme que l'identité a disparu ou est devenue zombie. Toute publication arrivée après la barrière porte une génération invalidée et ne peut que sortir dans le bootstrap à la vue de la fermeture ; elle n'est jamais promue ni signalée comme un service possédé. Si ce watchdog se bloque, la sortie du processus parent ferme tous les handles Job non héritables encore ouverts sous Windows ; sous macOS, le reaper CEF séparé signale les groupes admis avant la sortie brute, et le moniteur de chaque helper respecte aussi son échéance d'auto-terminaison.

La limite de 15 secondes concerne l'exécution, pas la disparition instantanée de l'objet processus dans les tables du noyau. Après que `TerminateProcess`, la fermeture du Job Object ou `SIGKILL` a réussi, le helper ne peut plus poursuivre le travail Beaver, mais Windows peut le laisser brièvement en terminaison et Unix comme zombie jusqu'au moissonnage. Le test natif distingue ces états non exécutables d'un processus encore runnable, puis attend leur disparition pendant une fenêtre de constat séparée et bornée. La conception ne prétend pas rendre synchrone une terminaison que le système d'exploitation définit comme asynchrone.

La sortie de 15 secondes est volontairement brute : elle n'exécute aucun destructeur, callback `atexit`, flush de log ou nettoyage supplémentaire. Tout travail récupérable repose donc sur un journal durable écrit avant cette limite.

### Confinement établi au lancement

Le filet d'urgence est préparé au moment du spawn, pas improvisé pendant la fermeture :

- Windows : un enfant possédé est affecté au Job Object Beaver immédiatement après son spawn et avant que son handle soit rendu à l'appelant ; `portable-pty` fournit le handle natif nécessaire sans fork de la dépendance ; si l'affectation échoue, l'enfant et les descendants détectés dans cette courte fenêtre sont arrêtés et moissonnés, puis l'opération échoue ;
- Linux : chaque enfant direct possédé reçoit un signal de mort du parent et un groupe de processus dédié avant `exec` ;
- macOS : chaque enfant possédé reçoit un groupe de processus dédié, enregistré dans les slots atomiques du watchdog ;
- toutes les plateformes : le parent vérifie immédiatement que l'identité enregistrée correspond au processus créé ; un échec tue et moissonne cet enfant avant de rendre le spawn visible.

Les helpers CEF sont l'exception explicite à l'enregistrement d'un handle avant le spawn, car la bibliothèque native contrôle leur création. Ils ne sont toutefois jamais admis sans réservation : le callback réserve le slot, le bootstrap propose son identité dans sa boîte isolée et le parent acquiert la preuve d'arrêt dans sa table privée avant d'autoriser l'entrée dans CEF. Un helper qui dépasse cette capacité ou ne peut pas publier sort immédiatement et déclenche la fermeture coordonnée. Le handshake rend désormais sûre l'affectation Windows au Job Object avant l'entrée dans CEF ; la table d'autorité privée, les objets sandboxés minimaux, le handle stable Windows ou l'identité revérifiée macOS constituent leur chemin forcé dédié.

Pour les lanceurs Windows ordinaires qui ne disposent pas du handshake CEF, la fenêtre entre spawn et affectation est acceptée comme compromis explicite : le Job Object, l'inventaire d'identité, le balayage final et le nettoyage borné au lancement suivant forment quatre défenses complémentaires, sans réécrire tous les lanceurs avec `CreateProcessW` brut.

Le helper de mise à jour Beaver conserve le mécanisme `UpdateHandoff` déjà présent et suit un lanceur distinct qui ne l'inscrit jamais dans le Job Object destructeur. Son exécutable, l'asset et les arguments sont validés avant le spawn ; l'opération suivie capture puis publie immédiatement son identité complète. Tant que cette publication n'a pas réussi, le guard local l'arrête et le moissonne lors d'une fermeture normale. Après publication, le balayage l'exclut uniquement si l'identité complète correspond encore. Un crash brutal dans la courte fenêtre spawn–publication peut laisser continuer ce helper déjà validé : c'est cohérent avec la mise à jour explicitement déclenchée et son protocole de santé au redémarrage. Aucun Job Object de handoff ni transfert de handle supplémentaire n'est introduit.

### Phases de fermeture

1. Fermer l'admission et annuler le jeton global.
2. Masquer toutes les fenêtres. Lors d'un vrai Quitter macOS, masquer aussi l'icône du Dock.
3. Annuler et attendre les opérations enregistrées qui peuvent encore écrire ou lancer un processus.
4. En parallèle, arrêter et attendre scheduler, gateway, extensions, flux, téléchargements, OAuth, shells et MCP.
5. En parallèle, arrêter Forecast, SearXNG et terminaux, puis libérer la VRAM.
6. Arrêter et moissonner Ollama en dernier.
7. Passer à `ReadyToExit` et sortir de la boucle Tauri.
8. Arrêter CEF dans l'ordre natif déjà validé.
9. Si l'appel CEF rend la main, signaler les processus possédés encore enregistrés, puis balayer les enfants directs non transférés ; s'il bloque, le watchdog draine directement et à répétition les slots CEF et les autres confinements de 13 à 15 secondes.
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

Le journal `ollama-update-state.json` vit sous `services::paths::data_dir()`. Il est borné à 4 Kio, versionné et écrit atomiquement par temporaire puis renommage. Avant chaque mutation de dossiers, le fichier temporaire est synchronisé, renommé, puis le dossier parent est synchronisé. Après chaque renommage de dossier réussi, le même parent est synchronisé avant la transition de journal suivante. Après la suppression complète d'un rebut, le parent est de nouveau synchronisé avant le retrait du journal ; le retrait du journal est lui-même suivi d'une dernière synchronisation du parent. Sous Windows, l'abstraction utilise les primitives natives de renommage durable et un handle de dossier ; sous Unix, elle utilise `fsync` sur le dossier. Si le système de fichiers refuse cette garantie, la phase durable précédente et son code récupérable sont conservés : Beaver ne prétend pas la transaction terminée. Un journal symlinké, non régulier, surdimensionné ou de schéma inconnu bloque la transaction sans suppression.

Les noms sont centralisés dans `services::paths` et désignent uniquement des enfants directs du dossier de données canonique :

- destination active : `ollama-bundle` ;
- staging de première installation : `ollama-bundle-install-staging` ;
- staging de mise à jour : `ollama-bundle-update-staging` ;
- sauvegarde : `ollama-bundle-backup` ;
- cible rejetée pendant un rollback : `ollama-bundle-failed` ;
- rebut de nettoyage de la sauvegarde : `ollama-bundle-backup-delete` ;
- rebut de nettoyage de la cible rejetée : `ollama-bundle-failed-delete`.

Les deux stagings distincts empêchent une première installation et une mise à jour interrompues de se confondre. Le verrou unique interdit néanmoins leur exécution concurrente.

### Isolement du stockage des modèles

Le sidecar hérite actuellement de l'environnement Beaver. Le gestionnaire calcule donc avant toute mutation le chemin de modèles réellement transmis : `OLLAMA_MODELS` hérité lorsqu'il existe, sinon le chemin par défaut de l'environnement d'exécution. Une valeur relative est résolue depuis le vrai dossier de travail du processus Ollama. La sonde possédée reçoit au contraire un dossier temporaire explicitement isolé et ne peut jamais réutiliser ce stockage réel.

Le chemin effectif et tous les dossiers transactionnels modernes ou hérités de la table des layouts sont comparés après résolution canonique de leurs ancêtres existants et rejet de tout symlink, junction ou reparse point ambigu. La comparaison emploie l'identité et les règles de casse du système de fichiers, jamais un préfixe textuel. Tout chevauchement dans un sens ou dans l'autre — égalité, stockage parent d'un bundle ou stockage enfant d'un bundle — bloque l'installation, la mise à jour et la récupération avant l'écriture du journal. Une résolution impossible échoue fermée. Le code public `ollama-model-store-conflict` est traduit dans les sept langues et ne révèle aucun chemin ; les logs conservent seulement la catégorie nettoyée.

Une sauvegarde ou une cible rejetée n'est jamais supprimée récursivement sous son nom d'autorité. Le gestionnaire vérifie d'abord son empreinte, exige l'absence du rebut correspondant, la renomme atomiquement vers ce rebut direct, synchronise le parent, puis supprime le rebut sans suivre de symlink, junction ou reparse point. Une interruption peut laisser un rebut partiel dont l'empreinte a changé : la phase durable autorise explicitement la reprise de cette seule suppression. La présence simultanée de la source et du rebut, un rebut hors de la phase attendue ou une racine non régulière reste ambigu et ne déclenche aucune suppression.

Les layouts réellement publiés sont recensés, pas déduits :

| Versions sources | Destination | Staging partagé | Sauvegarde | Cible rejetée |
|---|---|---|---|---|
| baseline `1.0.2`, `1.1.0`, `1.1.1`, `1.1.2` | `ollama-bundle` | `ollama-bundle-staging` | `ollama-bundle-old` | `ollama-bundle-failed` |
| branche de référence actuelle | `ollama-bundle` | `ollama-bundle-staging` | `ollama-bundle-old` | `ollama-bundle-failed` |
| format moderne | `ollama-bundle` | deux stagings distincts | `ollama-bundle-backup` | `ollama-bundle-failed` |

Toute future disposition publiée doit être ajoutée à cette table avant de modifier la migration.

États durables :

```text
Prepared
PendingValidation
CleanupPending
RollbackPending
RollbackCleanupPending
```

Le schéma conceptuel est un enum typé et refuse les champs inconnus :

```text
TransactionJournal(schema_version = 1) {
  Prepared { target, previous }
  PendingValidation { target, previous }
  CleanupPending { target, previous }
  RollbackPending { previous, rejected_target? }
  RollbackCleanupPending { previous, rejected_target? }
}

BundleFingerprint { version, executable_sha256 }
```

Une mise à jour moderne exige une destination existante et identifiable ; sans version précédente valide, la commande est redirigée vers la réparation ou la première installation et ne crée pas ce journal. `rejected_target` n'est absent que pendant une restauration où la cible a déjà disparu : migration d'un layout hérité ou reprise durable d'un `PendingValidation` dont la destination manque. Son absence interdit donc la présence d'un dossier `ollama-bundle-failed` ou de son rebut `ollama-bundle-failed-delete` : si l'un existe malgré `rejected_target: None`, l'état est ambigu, aucun fichier n'est déplacé ou supprimé et le code stable `ollama-update-recovery-required` est renvoyé. Cette règle vaut pour `RollbackPending` et `RollbackCleanupPending`.

Chaque version est une chaîne semver normalisée d'au plus 64 octets et chaque empreinte contient exactement 64 caractères hexadécimaux ASCII. Les empreintes sont comparées en temps constant, octet par octet. Aucun chemin absolu, message d'erreur ou texte extérieur n'est sérialisé. Aucun identifiant aléatoire n'est nécessaire, car le verrou et l'unique journal imposent une seule transaction.

- `Prepared` : staging complet et validé, échange pas encore confirmé ;
- `PendingValidation` : nouvelle installation en place, ancienne sauvegarde conservée ;
- `CleanupPending` : nouvelle version validée, suppression de la sauvegarde à reprendre ;
- `RollbackPending` : restauration de l'ancienne version à reprendre.
- `RollbackCleanupPending` : ancienne version restaurée, suppression de la cible rejetée à reprendre.

L'absence du journal signifie qu'aucune transaction moderne n'est active. Une migration unique interprète prudemment les dossiers des versions publiées et de la branche de référence listés ci-dessus. Toute ambiguïté ferme l'opération sans supprimer de dossier et produit un code récupérable.

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
| `PendingValidation` | destination absente, sauvegarde présente, sans dossier rejeté | écrire et synchroniser `RollbackPending { rejected_target: None }`, puis appliquer la reprise « sauvegarde précédente seule » |
| `CleanupPending` | destination cible + sauvegarde précédente exacte, sans rebut de sauvegarde | renommer la sauvegarde vers `ollama-bundle-backup-delete`, synchroniser le parent, puis supprimer ce rebut ; un échec garde `CleanupPending` |
| `CleanupPending` | destination cible + rebut de sauvegarde régulier, éventuellement partiellement supprimé, sans sauvegarde | reprendre uniquement la suppression sûre du rebut, synchroniser le parent après sa disparition, puis retirer et synchroniser le journal |
| `CleanupPending` | destination cible seule, sans sauvegarde ni rebut | le nettoyage est déjà terminé ; retirer le journal |
| `CleanupPending` | destination absente ou non validable | ne pas supprimer la sauvegarde ; passer à `RollbackPending` si elle existe, sinon exposer une récupération requise |
| `RollbackPending` | destination cible correspondant à `rejected_target: Some` + sauvegarde précédente, sans dossier rejeté | l'identité rejetée est déjà durable ; reprendre le déplacement vers le dossier rejeté, restaurer la sauvegarde, puis écrire `RollbackCleanupPending` |
| `RollbackPending` | sauvegarde précédente seule, `rejected_target: None`, sans destination ni dossier rejeté | restaurer la sauvegarde en destination, synchroniser le parent, puis retirer le journal ; une coupure après le renommage est reprise par la ligne « destination précédente seule » |
| `RollbackPending` | sauvegarde précédente + cible rejetée correspondant à `rejected_target: Some`, sans destination | terminer la restauration, puis écrire `RollbackCleanupPending` |
| `RollbackPending` | destination précédente + cible rejetée correspondant à `rejected_target: Some`, sans sauvegarde | le rollback est déjà restauré ; écrire `RollbackCleanupPending` |
| `RollbackPending` | destination précédente seule, sans sauvegarde ni dossier rejeté | le rollback et son nettoyage sont déjà terminés ; retirer le journal, que `rejected_target` soit encore présent ou non |
| `RollbackCleanupPending` | destination précédente + cible rejetée correspondant à `rejected_target: Some`, sans sauvegarde ni rebut rejeté | renommer la cible rejetée vers `ollama-bundle-failed-delete`, synchroniser le parent, puis supprimer ce rebut ; un échec conserve cet état |
| `RollbackCleanupPending` | destination précédente + rebut rejeté régulier, éventuellement partiellement supprimé, `rejected_target: Some`, sans sauvegarde ni cible rejetée | reprendre uniquement la suppression sûre du rebut, synchroniser le parent après sa disparition, puis retirer et synchroniser le journal |
| `RollbackCleanupPending` | destination précédente seule, `rejected_target: Some`, sans sauvegarde, cible rejetée ni rebut | le nettoyage est déjà terminé ; retirer le journal |
| `RollbackCleanupPending` | destination précédente seule, `rejected_target: None`, sans sauvegarde, cible rejetée ni rebut | le nettoyage hérité est déjà terminé ; retirer le journal |

Chaque qualification « cible » ou « précédente » exige l'empreinte attendue et la version normalisée inscrites dans le journal. Toute combinaison non listée ou empreinte différente est ambiguë : aucune suppression n'a lieu, l'état reste durable et un code public de récupération requise est renvoyé. Les tests coupent artificiellement l'opération avant et après chaque renommage, écriture du journal et synchronisation du parent, puis injectent l'échec de chacune de ces opérations.

La migration des dossiers hérités est elle-même unique et testée :

- destination seule : installation normale, aucun journal créé ;
- destination + `ollama-bundle-staging`, sans sauvegarde : staging pré-échange abandonné ; la destination est conservée et le staging exact est supprimé après validation de son type ;
- destination + `ollama-bundle-old`, sans staging : création de `PendingValidation` après calcul borné des deux empreintes ;
- `ollama-bundle-old` sans destination : calcul de l'empreinte précédente, création durable de `RollbackPending { previous, rejected_target: None }`, puis restauration par la ligne « sauvegarde précédente seule » ;
- destination + `ollama-bundle-failed`, sans sauvegarde ni rebut : calcul borné des deux empreintes, création de `RollbackCleanupPending` avec `rejected_target: Some(empreinte rejetée)` ;
- toute présence d'un rebut moderne sans journal moderne correspondant : état ambigu conservé intact ;
- toute autre combinaison : aucun déplacement ni suppression, code de récupération requise.

Le marqueur de migration n'est écrit qu'après la création durable du journal moderne, la fin sûre du nettoyage pré-échange ou la confirmation qu'aucun dossier hérité n'existe. Les tests construisent chaque layout publié et chaque combinaison ambiguë.

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
12. Si la suppression réussit, synchroniser son parent, retirer le journal puis synchroniser encore le parent. Si une suppression ou une synchronisation échoue, annoncer tout de même la mise à jour comme réussie, conserver `CleanupPending` et réessayer plus tard.

Si la validation échoue de façon certaine, le gestionnaire écrit et synchronise directement `RollbackPending { previous, rejected_target: Some(empreinte cible) }` avant le premier renommage de rollback. Il n'existe aucun état moderne intermédiaire avec une cible encore présente et `rejected_target: None`. Le gestionnaire n'efface jamais la destination cible avant que cette identité soit durable, que la sauvegarde ait été confirmée comme dossier interne régulier et que le renommage vers le dossier rejeté réussisse.

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

Une vue d'urgence générale utilise 128 slots atomiques préalloués contenant PID, groupe ou Job, heure de démarrage, génération et état. La seconde vue d'urgence CEF est la table parent privée de 64 slots décrite plus haut ; elle seule contient réservation, état et handles natifs. Les boîtes sandboxées par slot ne servent qu'à proposer une publication non fiable et ne sont jamais consultées comme une autorité de terminaison. Ces vues n'allouent pas et ne prennent pas le mutex du registre normal dans le watchdog. Sous Unix, les appels natifs décrits plus haut revérifient PID, groupe et heure de démarrage directement contre les slots privés. Sous Windows, un handle CEF revendiqué reste dans son slot privé jusqu'à ce que le processus soit signalé comme terminé. Les métadonnées riches restent dans le registre normal et servent au balayage post-boucle.

Le balayage final traite d'abord cet inventaire, puis les enfants directs découverts par le système. Il revérifie l'identité avant tout signal. Le helper transféré est exclu uniquement si son identité complète correspond encore.

Sur Unix, le signal du groupe possédé part avant les signaux individuels aux descendants et à la racine. Cela ferme d'abord la fenêtre où un parent superviseur pourrait recréer un enfant absent de l'instantané. Les zombies et processus morts sont ensuite ignorés et moissonnés par leur propriétaire lorsque celui-ci est encore disponible.

Cet inventaire n'adopte pas les applications externes ouvertes par Beaver et ne remplace pas l'arrêt normal des services.

## Matrice de scénarios obligatoire

### Fermeture générale

- fermeture normale sans service ;
- fermeture avec chaque service en état `starting`, `running` et `stopping` ;
- opération synchrone factice bloquée au-delà du délai asynchrone ;
- watchdog indépendant qui rend la sortie Tauri possible malgré ce blocage ;
- watchdog de processus lui-même bloqué dans un appel OS : le tueur ultime distinct déclenche tout de même la sortie brute à l'échéance ;
- échec injecté de création du tueur ultime au démarrage : aucune fenêtre, aucun service et aucun helper n'est lancé ; échec de création du watchdog à `Closing` : le tueur déjà armé conserve l'échéance ;
- fermeture du Job Object Windows et signal des groupes Unix avant la sortie ultime ;
- seconde demande de fermeture pendant le nettoyage ;
- aucune nouvelle admission après `Closing` ;
- arrêt CEF puis balayage final dans l'ordre prévu ;
- `cef::shutdown()` volontairement bloqué, helper CEF réel encore actif puis rendu non exécutable par le filet d'urgence avant la sortie forcée, avec disparition noyau contrôlée ensuite ;
- helper CEF réservé avant `Closing` mais publié après 13 secondes : sa génération déjà invalidée interdit `Admitted`, le bootstrap constate la fermeture et sort sans appel CEF ;
- demande de lancement CEF après `Closing` et réservation périmée : le bootstrap sort avant `cef::execute_process` ;
- callback CEF sans ligne de commande exploitable ou ajout du marqueur impossible : génération invalidée, porte fermée, initialisation refusée et aucun chemin de repli vers `cef::execute_process` ;
- permis CEF pris avant `Closing` mais encore inachevé après 50 millisecondes : génération invalidée, preuve locale nettoyée, admission tardive impossible et nettoyage des autres services poursuivi sans sortie brute anticipée ;
- erreur transitoire de prévalidation : une seule seconde tentative propre après 200 millisecondes ; erreur déterministe : aucun retry ; `cef::initialize` faux : aucun second appel ni `cef::shutdown`, puis fermeture coordonnée ;
- notification d'indisponibilité : design existant, sept langues, action générique de redémarrage coordonné, fermeture manuelle et disparition complète à 10 secondes sans bloquer l'interface ;
- sous Windows, `TerminateProcess` réussi mais handle pas encore signalé : le slot reste possédé et est revérifié sans fermeture prématurée ;
- chaque type de helper CEF Windows réel avec sandbox actif publie en état `Ready supervisé` à travers la DACL, les SIDs de restriction et le niveau MIC attendus, sans droit de signaler lui-même l'admission ou la fermeture ; un échec local injecté avant initialisation prouve séparément qu'aucun helper n'est créé ;
- build empaqueté exercé avec protections Microsoft actives, puis dans un environnement Windows renforcé représentatif par antivirus tiers ou politique d'entreprise ; macOS exercé avec Gatekeeper et quarantaine actifs ; une incompatibilité reproductible sur une configuration supportée bloque la fusion ;
- corruption volontaire d'une boîte CEF avant validation, réécriture après scellement, tentative d'écriture inter-slot et fausse valeur de handle : aucune autorité parent n'est modifiée, aucun PID externe n'est adopté ou signalé et l'initialisation échoue fermée ;
- appel OS du watchdog général bloqué ou panique du traqueur macOS normal : le Job Object Windows ou le reaper CEF macOS distinct rend tout helper admis non exécutable avant la sortie brute ;
- reaper macOS absent, arrêté ou signal refusé dans le smoke test : CEF n'est pas initialisé ou la fermeture coordonnée démarre ; le sandbox n'est jamais relâché ;
- PID ou PGID macOS réutilisé après la fin du helper : le reaper refuse le signal ; avec le watchdog général bloqué, chaque helper réel respecte tout de même son échéance d'auto-terminaison ;
- helpers CEF et shell actifs simultanément dans le build empaqueté : le routeur unique atteint le vrai helper shell, et seul le rôle CEF muni de sa réservation valide occupe les 64 slots dédiés ;
- ligne de commande Chromium de plus de 64 arguments, argument opaque long et argument opaque non Unicode : les champs privés restent bornés et valides sans plafond artificiel sur les arguments étrangers ;
- le contrat Node lancé directement avec une fixture invalide échoue sur les trois OS, et chaque filtre CI énumère un nombre non nul de tests avant exécution ;
- un build empaqueté lance un vrai enfant CEF avec sa ligne de commande réelle, ouvre une page puis ferme Beaver pendant son activité ; les processus factices restent une preuve complémentaire du protocole ;
- helper validé seul survivant à une mise à jour Beaver.

### Gateway et extensions

- fermeture avec Telegram, Discord et Slack simulés ;
- file gateway fermée et toutes les tâches attendues ;
- message en cours annulé sans nouveau traitement après `Closing` ;
- redémarrage automatique du gateway au lancement suivant si configuré ;
- redémarrage manuel et automatique de l'hôte d'extensions refusé après `Closing` ;
- aucun host recréé entre `extensions.stop_and_wait` et le balayage final.

### Scheduler et flux

- réveil ponctuel inactif sans appel provider ni fausse erreur ;
- annulation après revendication enregistrée avec une issue typée et traduite ;
- erreur de `claim_once` pendant la réconciliation d'un réveil manqué journalisée de façon bornée, sans détail interne visible ;
- notification du scheduler après chaque mutation de réveil ;
- saturation et remplacement de flux exposés par des codes stables traduits dans les sept langues ; erreur inconnue masquée par le fallback générique.
- compteurs d'admission locaux exacts et bornés ; cycles successifs au-delà de la capacité cumulée sans fuite de slot ni fausse saturation.

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
- migration `ollama-bundle-old` seule : coupure juste après `RollbackPending { rejected_target: None }`, puis restauration automatique ; seconde coupure après le renommage mais avant le retrait du journal, puis reprise idempotente ;
- `PendingValidation` sans destination avec sauvegarde seule suit le même chemin automatique ;
- `RollbackPending` avec cible et sauvegarde écrit durablement `rejected_target: Some` avant tout déplacement vers `ollama-bundle-failed` ;
- coupure après cette écriture mais avant le déplacement : reprise du renommage à partir de `rejected_target: Some` ;
- coupure après la suppression de `ollama-bundle-failed-delete` mais avant le retrait de `RollbackCleanupPending { rejected_target: Some }` : retrait idempotent du journal ;
- coupure ou erreur au milieu de la suppression de `ollama-bundle-backup-delete` ou `ollama-bundle-failed-delete` : reprise de ce rebut partiel sans exiger son empreinte d'origine ;
- coupure et erreur injectées entre la disparition du rebut, la synchronisation du parent, le retrait du journal et sa synchronisation finale : la phase reste récupérable et aucun rebut ne réapparaît sans autorité ;
- source et rebut simultanés, rebut symlinké/reparse ou rebut présent hors phase : état ambigu, aucune suppression ;
- `RollbackCleanupPending { rejected_target: None }` avec destination précédente seule retire le journal sans toucher à la destination ;
- `RollbackPending` et `RollbackCleanupPending` avec `rejected_target: None` mais `ollama-bundle-failed` ou son rebut présent : état ambigu, aucun renommage ni suppression, journal et octets des dossiers inchangés ;
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
- chemin de modèles par défaut ou `OLLAMA_MODELS` externe accepté ; valeur relative résolue depuis le vrai dossier de travail ; égalité ou chevauchement parent/enfant avec chaque dossier transactionnel moderne ou hérité refusé avant toute mutation ; alias symlink, junction et reparse point refusés ;
- migration réelle depuis chaque release `1.0.2`, `1.1.0`, `1.1.1` et `1.1.2` sur chaque OS où elle a été publiée, avec modèle préexistant toujours listé, vérifié et utilisable sans nouveau téléchargement.

### Intégrité des données utilisateur

- profil éphémère contenant configuration, sessions, vault avec keystore factice isolé, données Forecast, skills, mémoire, métadonnées de modèles et historiques ;
- fermeture normale puis fermeture forcée à chaque frontière durable pendant une écriture de chaque famille ;
- après redémarrage, chaque fichier final est lisible et cohérent, et chaque mutation est entièrement absente ou entièrement commitée ;
- seuls des temporaires explicitement documentés peuvent rester, sans secret, avec reprise ou nettoyage borné ;
- aucune valeur sensible ni aucun chemin complet n'apparaît dans les diagnostics de reprise.

### Processus et plateformes

- terminal tué puis moissonné ;
- zombie Linux ignoré sans boucle de trois secondes ;
- ordre groupe, descendants, racine sous Unix ;
- `taskkill` Windows conserve ses arguments séparés et son chemin système validé ;
- clic tray et item Afficher restaurent une fenêtre minimisée ;
- croix macOS masque sans nettoyage ;
- vrai Quitter macOS masque le Dock avant le nettoyage.

## Stratégie de mise en œuvre et de fusion

La branche actuelle contient 31 commits dans la photographie `main..42823ba`, dont 22 commits de code ou de CI et 9 commits documentaires. Elle reste intacte comme sauvegarde et source de comparaison. Aucun nouveau code d'implémentation n'y est ajouté et elle n'est pas proposée à la fusion. L'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md) est l'autorité exhaustive pour ces 22 commits.

Le travail passe par cinq jalons :

1. **Socle de fermeture** : états, admission suivie, budgets, frontière bloquante et watchdog.
2. **Supervision CEF** : admission native, confinement Windows, reaper macOS, sandbox, disponibilité obligatoire à la livraison et repli local avant lancement.
3. **Processus et services** : inventaire, confinement, gateway, extensions, MCP, Forecast, terminaux, SearXNG et mise à jour Beaver.
4. **Transaction Ollama** : journal, migration, sonde isolée, première installation, mise à jour, récupération et polling.
5. **Convergence multi-OS** : dettes structurelles dans le périmètre, inventaires finaux, tests natifs et validation manuelle.

Le graphe de fusion est `J1 -> {J1B, J2}`, `J2 -> J3`, puis `{J1B, J3} -> J4`. Cette indépendance permet au travail sur les services et Ollama d'avancer pendant la preuve native CEF, sans autoriser une release incomplète.

Pour chaque jalon :

- créer une nouvelle branche depuis le `main` qui contient toutes ses dépendances déjà fusionnées ;
- écrire un plan TDD propre au jalon ;
- commencer chaque comportement par un test qui échoue pour la bonne raison ;
- conserver le mécanisme existant tant que son remplacement n'est pas totalement adopté ;
- produire des commits petits et révisables, puis une PR dédiée ;
- exécuter la review globale du jalon et les tests natifs avant fusion ;
- fusionner la PR avant de créer une branche qui en dépend.

La branche du premier jalon reprend uniquement les commits documentaires du contrat, des cinq spécifications et de l'inventaire, puis implémente son propre périmètre. Aucun commit de code de la grande branche n'est repris en bloc ; une correction existante n'est réutilisée qu'après comparaison avec `main`, test isolé et rattachement au jalon qui la possède. Chaque PR ferme ses lignes d'inventaire avec le commit de remplacement et ses preuves ; une ligne partagée reste ouverte jusqu'au dernier jalon indiqué. Aucune release publique n'est créée entre le début du jalon 1 et la fusion du jalon 4.

Juste avant la fusion du jalon 1, une branche de maintenance protégée est créée depuis le dernier `main` publiable. Elle ne reçoit que les correctifs critiques ou de sécurité nécessaires à une release d'urgence. Chaque correctif est immédiatement reporté dans les branches de jalon concernées et revalidé. Cette branche reste disponible jusqu'à la fusion et à la validation native du jalon 4, puis elle peut être retirée ; le `main` transitoire ne sert jamais à produire une release.

Chaque commit final de jalon reçoit une Git note expliquant objectif, causes racines, décisions, alternatives rejetées, compatibilité multi-OS et validations. Le reviewer peut ainsi vérifier le raisonnement sans dépendre de l'historique de conversation.

## Journal des décisions pour le reviewer

| Sujet | Décision | Justification |
|---|---|---|
| Gateway | Une vraie fermeture arrête Telegram, Discord et Slack ; seule la croix rouge macOS masque l'app | comportement demandé explicitement, prévisible et identique pour Quitter sur les trois OS |
| `run_when_window_closed` | retirer le champ des modèles et nouvelles écritures, tolérer sa présence dans les anciens JSON | éviter un réglage mort tout en conservant la compatibilité des données |
| Taille du chantier | cinq PR reliées par `J1 -> {J1B, J2}`, `J2 -> J3`, `{J1B, J3} -> J4` ; branche actuelle conservée mais non fusionnée | éviter une nouvelle méga-branche, isoler CEF du socle et laisser avancer les travaux indépendants sans publier un état incomplet |
| Acquis de la grande branche | inventorier les 22 commits de code et fermer chaque ligne dans son jalon | empêcher la perte silencieuse de corrections déjà trouvées, notamment les réveils ponctuels et les erreurs de flux traduites |
| Windows | affecter immédiatement les enfants au Job Object après spawn, sans suspension généralisée | `portable-pty` expose le handle ; le gain d'une suspension partout ne justifie pas une réécriture de tous les lanceurs |
| Helper Beaver | conserver `UpdateHandoff`, sans Job Object de transfert supplémentaire | mécanisme existant borné à une identité ; complexité supplémentaire sans bénéfice proportionné |
| Watchdog Unix | slots avec heure de démarrage et revérification native dans des buffers fixes | garantir l'identité sans dépendre de Tokio, `sysinfo`, du tas ou d'un mutex |
| Sortie ultime | `TerminateProcess` sous Windows, `_exit` sous macOS/Linux | chemin brut qui ne peut pas attendre un destructeur ou un callback |
| Migration Ollama | couvrir explicitement baseline `1.0.2` et releases `1.1.0` à `1.1.2` | ce sont les layouts réellement présents chez les utilisateurs |
| Stockage des modèles | résoudre le chemin effectif, y compris `OLLAMA_MODELS` hérité, et refuser tout chevauchement avec les dossiers transactionnels avant mutation | une configuration personnalisée peut placer des modèles hors du chemin par défaut et ne doit jamais les faire entrer dans un renommage ou une suppression de bundle |
| Validation des migrations | exécuter une vraie mise à niveau depuis chaque release publiée avec modèle préexistant | les fixtures unitaires prouvent la table d'états, pas l'ensemble du cycle produit par les anciens binaires |
| Adoption | nommer MCP, Forecast, PTY, SearXNG et tous les autres producteurs de processus | empêcher qu'un service ne contourne l'autorité globale |
| CEF bloqué | fermer une porte de lancement partagée, réserver et marquer chaque rôle CEF avant le spawn, puis drainer ses slots en continu de 13 à 15 s | le balayage post-CEF n'est jamais atteint si l'appel natif bloque ; un passage unique à 13 s laisserait une fenêtre aux helpers publiés tardivement |
| Échéance CEF | séparer le watchdog de processus du tueur ultime précréé, qui ne scanne rien et force seul la sortie brute à 15 secondes | une création de thread tardive, un appel OS bloqué ou une dernière passe ne doit jamais repousser la seule garantie empêchant Beaver de devenir invisible et impossible à quitter |
| Preuve CEF | exiger l'absence de helper admis runnable avant la sortie, puis observer pendant 5 secondes la disparition des objets en terminaison, zombies et bootstraps refusés | les systèmes rendent la terminaison asynchrone ; un candidat jamais admis peut seulement exécuter le bootstrap fail-closed et ne doit jamais appeler CEF |
| Portée CEF | traqueur natif sur Windows et macOS ; sous Linux, vérifier l'absence de `native_browser` et de helper | `build.rs` active actuellement CEF uniquement sur Windows hors `windows-tests` et sur macOS ; ne pas prétendre tester un runtime Linux inexistant |
| Rôle CEF Windows | exiger une réservation et une génération injectées par le callback, pas seulement `current_exe()` | Beaver utilise aussi son propre exécutable pour le helper shell ; filtrer seulement le chemin mélangerait les rôles et pourrait saturer les slots CEF |
| Registre CEF Windows | garder états et handles dans le parent, isoler chaque publication sandboxée et adapter DACL, SIDs de restriction et MIC au jeton CEF réel | une zone enfant-écrivable ne peut porter aucune autorité ; une simple DACL utilisateur ne suffit pas pour les jetons restreints ou `Untrusted` |
| Secours CEF macOS | reaper parent précréé sur les groupes admis, plus auto-terminaison du helper à la mort du parent | `_exit` du parent ne tue pas ses groupes et la politique Seatbelt ne garantit pas qu'un helper sandboxé puisse signaler tout son groupe |
| Barrière CEF | conserver 50 millisecondes comme échéance centrale d'invalidation, sans sortie brute en cas de dépassement | une machine chargée peut dépasser cette borne ; la porte fermée suffit à empêcher l'admission tardive tandis que le reste du nettoyage doit continuer |
| Retry CEF | une seule nouvelle tentative typée pendant la prévalidation ; aucune réinitialisation de CEF dans le même processus | les ressources natives transitoirement indisponibles peuvent être recréées proprement avant CEF, alors que le contrat CEF impose la sortie après un échec d'initialisation |
| Indisponibilité CEF | notification existante enrichie d'une action `Redémarrer`, entièrement retirée après 10 secondes | expliquer l'échec local sans bloquer l'utilisateur ni installer un contrôle permanent ; le redémarrage complet est le seul retry sûr après la frontière CEF |
| Routage Windows | une seule autorité distingue parent, CEF, shell isolé et entrée invalide ; aucun plafond global de 64 arguments | l'exécutable empaqueté porte plusieurs rôles et Chromium contrôle sa propre ligne de commande, qui ne doit pas être rejetée pour des arguments étrangers |
| Bundle Windows | construire un seul type par invocation, NSIS par défaut et MSI explicitement isolé ; injecter le type depuis cette même demande dans le bootstrap et la copie empaquetée de la DLL | Tauri ne corrige que l'exécutable principal et une DLL préparée une seule fois ne peut pas représenter simultanément deux types sans valeur inventée ni course entre paquets |
| Découpage CEF | déplacer toute la supervision native dans un jalon 1B parallèle au jalon 2 après le socle | le socle anti-fantôme reste petit et fusionnable ; les services peuvent avancer sans réduire les garanties finales de CEF |
| Livraison CEF | Windows et macOS exigent `Ready supervisé` avant fusion du jalon 1B et avant release ; `Unavailable avant lancement` reste seulement une défense locale injectée ou liée à une machine défaillante | livrer un navigateur intégré désactivé serait une régression produit ; face à un blocage technique, le calendrier cède, jamais la fonctionnalité ni le sandbox |
| Compatibilité CEF réelle | tester le build empaqueté avec protections système actives et au moins un environnement Windows renforcé, sans télémétrie distante | les permissions, antivirus et politiques d'entreprise ne sont pas reproduits intégralement par un runner CI propre |
| Portée Linux CEF | conserver `native_browser` désactivé sous Linux et traiter son intégration complète dans un chantier séparé | l'intégration Linux est plus large que la supervision de fermeture et ne doit pas être simulée dans ce chantier |
| Reprise rollback héritée | décrire `RollbackPending + sauvegarde seule + rejected_target: None` comme une restauration automatique | ce layout est créé par la migration et par une cible absente ; le classer ambigu contredirait la récupération promise |
| Suppression Ollama | renommer une source validée vers un rebut propre à la phase avant toute suppression récursive, synchroniser sa disparition puis retirer le journal | une coupure au milieu de `remove_dir_all` détruit l'empreinte initiale ; le nom durable et la phase donnent une autorité de reprise sans deviner ni abandonner un rebut ressuscité |
| Empreintes | conserver la comparaison en temps constant | règle de sécurité obligatoire du projet pour les hash, même si l'empreinte du binaire est publique |
| Continuité des releases | conserver une branche de maintenance depuis le dernier `main` publiable jusqu'au jalon 4 | un correctif critique doit rester publiable sans expédier l'architecture transitoire |
| Intégrité des données | redémarrer un profil complet après fermetures normales et forcées à chaque frontière | le chantier ne déplace pas ces données, mais l'annulation et la sortie ultime peuvent interrompre une écriture en cours |

## Revue globale obligatoire avant chaque fusion

La review de chaque jalon met à jour les six inventaires explicitement vérifiés :

1. Tous les chemins qui créent un processus, y compris indirectement dans CEF, Tauri ou une bibliothèque native, classés en possédé, externe, court ou transféré.
2. Tous les travaux asynchrones longs ou mutateurs, avec leur admission, annulation et preuve de fin.
3. Tous les appels synchrones atteignables pendant une fermeture, avec leur frontière bloquante et leur borne.
4. Toutes les transitions Ollama, avec l'état durable avant et après chaque mutation de fichiers.
5. Tous les accès au journal ou aux dossiers Ollama, qui doivent aboutir au gestionnaire unique et au même verrou.
6. Tous les stockages persistants, avec leur écriture atomique, leur comportement sous annulation et leur preuve de réouverture après fermeture normale ou forcée.

Elle met aussi à jour les lignes du [registre de reprise des 22 commits](./2026-08-09-shutdown-reference-branch-inventory.md) attribuées au jalon. Le reviewer vérifie le scénario repris et son test, pas uniquement la présence d'un hash ou d'un fichier similaire.

Elle vérifie également :

- le diff complet du jalon contre le `main` dont il est issu ;
- les comportements existants de CEF, gateway, scheduler, SearXNG, MCP, Forecast et mise à jour ;
- si CEF est actif, l'identification bornée de ses helpers et leur terminaison forcée quand l'arrêt natif ne rend pas la main ; sinon, la preuve qu'aucun helper n'est lancé ;
- l'appel à `scheduler.notify_config_changed()` après chaque mutation de réveil ;
- les textes visibles dans les sept langues ;
- les fichiers de production sous 230 lignes ;
- l'absence de collection externe non bornée ;
- l'absence de `tokio::spawn`, `std::thread::spawn` ou spawn de processus longue durée sans propriétaire, limite et chemin d'arrêt documentés ;
- l'absence de chemin, secret ou erreur brute dans l'interface et les journaux ;
- la mise à jour Graphify après le code et la documentation.

Après le cinquième jalon, une dernière review compare le `main` obtenu au `main` antérieur au premier jalon et vérifie les six inventaires cumulés. Cette review finale ne remplace pas les reviews de chaque PR.

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
- sur Windows et macOS, preuve obligatoire du chemin normal `Ready supervisé` avec helpers `Admitted` non runnable après fermeture ; test séparé d'un échec local avant initialisation, qui rend le navigateur indisponible sans créer de helper ; candidat non admis limité au bootstrap fail-closed, puis contrôle sous 5 secondes de la disparition de tous les objets associés ;
- build empaqueté CEF validé avec protections système actives et environnement Windows renforcé représentatif, résultats consignés sans télémétrie distante ;
- sous Linux, preuve que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé dans ce chantier ;
- tests manuels d'une mise à jour Beaver et d'une mise à jour Ollama interrompue ;
- mises à niveau réelles depuis chaque release publiée, avec stockage de modèles par défaut et personnalisé hors bundle ;
- profil complet rouvert avec succès après fermeture normale et forcée, sans fichier final tronqué ni secret dans les diagnostics.

## Critères d'acceptation

- Beaver ne peut pas rester invisible en état `Closing` au-delà du délai absolu.
- Sur Windows et macOS, CEF est actif et sa supervision native est prouvée avant livraison ; un échec local de prérequis reste fermé avant tout lancement. Un blocage de l'arrêt ne laisse aucun helper `Admitted` capable de poursuivre du travail après la sortie forcée de Beaver, un candidat non admis ne peut qu'exécuter le bootstrap fail-closed et tous les états résiduels disparaissent dans les 5 secondes de constat.
- Sous Linux, CEF reste désactivé et son intégration complète demeure hors périmètre.
- Une vraie fermeture arrête le gateway et tous les processus possédés.
- La croix macOS ne ferme pas Beaver.
- Aucun service ne redémarre après la fermeture de l'admission.
- Aucun gros téléchargement partiel Beaver ne survit à une annulation normale.
- Une mise à jour Ollama ne peut être validée que par le binaire cible possédé et la version attendue.
- Une transaction Ollama ne commence pas tant que l'absence de chevauchement avec le stockage effectif des modèles n'est pas prouvée.
- Un démon Ollama externe n'est jamais arrêté ni utilisé comme preuve de validation.
- Une suppression de sauvegarde échouée n'annule pas une mise à jour déjà validée.
- Toute transaction Ollama interrompue possède un chemin automatique de reprise.
- Une sauvegarde précédente seule avec `RollbackPending { rejected_target: None }` est restaurée automatiquement et de manière idempotente.
- Un journal sans `rejected_target` accompagné d'un dossier rejeté reste intact et échoue fermé.
- Les 22 lignes de reprise de la branche de référence sont closes avec une preuve testée ou une justification approuvée.
- Aucun test de délai ne confond une future coopérative avec un appel bloquant.
- Les stockages persistants restent lisibles et cohérents après redémarrage, y compris après la sortie ultime.
- Une branche de maintenance du dernier `main` publiable reste disponible et synchronisée jusqu'à la validation du jalon 4.
- Les suites et builds des trois systèmes réussissent avant fusion.
