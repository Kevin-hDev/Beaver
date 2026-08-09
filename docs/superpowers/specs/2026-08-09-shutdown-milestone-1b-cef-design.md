# Jalon 1B — supervision native de CEF

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md), de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md) et du [jalon 1](./2026-08-09-shutdown-milestone-1-core-design.md). Sa branche est créée depuis le `main` où le jalon 1 a été fusionné.

Le contrat principal décrit le protocole CEF complet et prévaut sur ce résumé. Le présent jalon isole son implémentation, ses tests natifs et sa décision de repli afin qu'un problème interne au sandbox Chromium ne bloque pas la livraison du socle anti-fantôme.

## Objectif fusionnable

Garantir qu'un arrêt CEF bloqué ne laisse aucun helper CEF admis capable de poursuivre du travail après la sortie forcée de Beaver, sans désactiver le sandbox, sans adopter un processus externe et sans dépendre du balayage placé après `cef::shutdown()`.

La fusion de ce jalon ferme l'exception transitoire du jalon 1. Le jalon 2 et toute nouvelle release publique dépendent de cette fusion.

## Inclus

- table d'autorité parent privée à 64 slots et porte atomique `CefLaunchGate` ;
- boîte de publication et page de contrôle isolées par réservation, nommées avec des nonces CSPRNG de 256 bits ;
- marqueur de rôle CEF borné ajouté avant le lancement ;
- admission parent unique avant tout appel à `cef::execute_process` ;
- invalidation des réservations tardives au passage à `Closing` ;
- traqueur natif démarré avant `cef::initialize` ;
- table d'urgence CEF indépendante du registre général ;
- Job Object Windows propre à chaque helper, vide et `KILL_ON_JOB_CLOSE` ;
- droits Windows minimaux compatibles avec les SIDs activés, les SIDs de restriction et le niveau MIC réel ;
- groupe, reaper parent et moniteur d'auto-terminaison propres à macOS ;
- terminaison et revérification continues de 13 à 15 secondes, indépendantes de `cef::shutdown()` ;
- disparition native des objets `terminating`, zombies et bootstraps refusés vérifiée pendant une fenêtre de constat de 5 secondes appartenant aux tests ;
- job CI macOS natif qui prépare CEF et exécute compilation et Clippy strict ;
- fermeture des sous-lignes J1B de l'inventaire de reprise.

## Repli sûr pré-approuvé

Le mode dégradé n'essaie jamais de deviner l'identité d'un helper à partir de son nom, de son PID seul ou du callback parent, qui ne fournit pas à lui seul une preuve de processus stable.

Chaque plateforme possède exactement deux états autorisés :

1. `Ready supervisé` : tous les prérequis natifs sont créés avant CEF et les smoke tests prouvent la publication, l'admission et la terminaison des types de helpers réellement utilisés ;
2. `Unavailable avant lancement` : le navigateur intégré reste indisponible et Beaver continue normalement, sans avoir lancé ni admis de helper CEF.

Une impossibilité détectée avant `cef::initialize` — création du traqueur, des objets sécurisés, du reaper ou échec d'un prérequis natif Windows — sélectionne `Unavailable avant lancement`. Le runtime marque la capacité navigateur indisponible, comme il sait déjà le faire, et l'interface utilise son message traduit existant. La politique de capacité est centralisée et choisie avant CEF ; elle ne peut pas basculer silencieusement après la création d'un helper.

Si les tests natifs montrent qu'un type sandboxé ne peut pas publier avec les droits minimaux, la plateforme ne peut pas activer `Ready supervisé`. Le sandbox n'est jamais désactivé, les droits ne sont jamais élargis et aucun suivi approximatif n'est ajouté. La Git note indique explicitement si la plateforme reste temporairement en `Unavailable avant lancement`.

Après qu'un processus a potentiellement été créé, une publication invalide, une identité ambiguë ou une admission impossible ne peut plus être convertie en simple indisponibilité silencieuse. Le bootstrap est arrêté avant CEF quand son identité est sûre ; si CEF est déjà actif ou si un enfant connu peut avoir échappé à l'admission, la porte ferme et Beaver déclenche immédiatement une vraie fermeture coordonnée.

Le comportement supposé d'auto-terminaison des processus Chromium n'est jamais utilisé comme preuve contractuelle. Le repli sûr est l'absence de lancement, pas l'espoir qu'un helper non supervisé finira par sortir.

## Contraintes Windows

L'exécutable Beaver sert aussi au bac à sable shell. Le filtre CEF exige donc réservation, génération, parent, heure de démarrage et exécutable canonique ; le chemin seul est insuffisant. L'autorité et les handles restent exclusivement dans le parent. La zone modifiable par le helper ne porte jamais un handle, un état d'admission ou les identifiants d'un autre slot.

Avant `Admitted`, le parent ouvre l'identité stable puis place le helper dans son Job Object propre, compatible avec le job du sandbox Chromium actif. Tout échec ferme les guards locaux et refuse l'admission. La sortie brute du parent ferme le dernier handle non héritable du Job.

## Contraintes macOS

Le helper ouvre sa boîte, sa page de contrôle et ses événements, puis crée son groupe avant `sandbox.initialize`. Il publie seulement après l'application du sandbox. Le parent revérifie PID, parent, heure de démarrage, exécutable et PGID avant `Admitted`.

Le reaper parent précréé rescane les générations admises et revalide l'identité avant chaque signal. Le moniteur du helper s'auto-termine si le parent disparaît ou au plus tard une seconde avant la sortie brute. Il ne suppose pas que Seatbelt l'autorise à signaler tout son groupe.

## Tests obligatoires

- arrêt natif CEF normal suivi du balayage final dans l'ordre existant ;
- `cef::shutdown()` bloqué avec helper réel admis, rendu non exécutable avant la sortie forcée ;
- callback sans ligne de commande exploitable, marqueur impossible, capacité saturée et identité ambiguë : aucun appel CEF de repli ;
- callback après `Closing`, réservation expirée et publication après 13 secondes : génération invalide et sortie avant CEF ;
- permis encore inachevé après la barrière de 50 millisecondes : admission tardive impossible ;
- candidat jamais publié juste avant 15 secondes : aucun appel CEF et disparition dans la fenêtre de constat ;
- helpers CEF et shell simultanés sous Windows : seul le rôle réservé est adopté ;
- en état `Ready supervisé`, chaque type CEF Windows réel publie avec le sandbox actif, les SIDs de restriction et le MIC minimal ; en état `Unavailable avant lancement`, aucun helper n'est créé ;
- corruption de boîte, réécriture après scellement, faux handle et tentative inter-slot : aucune autorité parent modifiée et aucun processus externe signalé ;
- Job Object imbriqué Windows réussi avec le sandbox actif ; échec d'affectation fermé ;
- `TerminateProcess` accepté mais handle pas encore signalé : slot conservé puis revérifié ;
- panne du traqueur et watchdog général bloqué : Job Object Windows ou reaper macOS reste efficace ;
- macOS : objets et groupe avant sandbox, publication et moniteur après sandbox, PGID réutilisé refusé ;
- reaper macOS absent ou arrêté : CEF indisponible avant lancement ou fermeture coordonnée si la porte était déjà ouverte ;
- échec de chaque prérequis avant initialisation : `BrowserCapability::Unavailable`, Beaver utilisable et aucun helper créé ;
- test Linux confirmant que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé ;
- smoke test natif de l'état retenu sur chaque plateforme ; l'échec injecté avant initialisation couvre toujours `Unavailable avant lancement`, et le chemin `Ready supervisé` est obligatoire partout où CEF reste activé.

## Critères de fusion

- aucune identification fondée seulement sur un nom ou un PID ;
- aucune permission de sandbox élargie pour faire passer la publication ;
- toute plateforme est soit `Ready supervisé` avec ses preuves natives, soit `Unavailable avant lancement` sans helper ;
- aucun helper CEF admis ne poursuit d'exécution dans le scénario d'arrêt natif bloqué ;
- tout objet noyau résiduel et tout bootstrap refusé disparaissent dans la fenêtre de constat ;
- aucune application externe ni aucun helper shell adopté ou signalé ;
- job CI macOS CEF et validations Windows natives réellement exécutés ;
- toutes les sous-lignes J1B de l'inventaire sont fermées et référencent leurs tests ;
- fichiers de production sous 230 lignes, suites complètes et CI native vertes ;
- Git note détaillant le mode retenu par plateforme, les alternatives rejetées et les preuves.
