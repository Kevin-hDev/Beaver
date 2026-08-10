# Jalon 1B — supervision native de CEF

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md), de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md) et du [jalon 1](./2026-08-09-shutdown-milestone-1-core-design.md). Sa branche est créée depuis le `main` où le jalon 1 a été fusionné.

Le contrat principal décrit le protocole CEF complet et prévaut sur ce résumé. Le présent jalon isole son implémentation, ses tests natifs et sa défense locale avant lancement afin que le socle anti-fantôme et le jalon 2 puissent avancer sans réduire les garanties de livraison du navigateur.

## Objectif fusionnable

Garantir qu'un arrêt CEF bloqué ne laisse aucun helper CEF admis capable de poursuivre du travail après la sortie forcée de Beaver, sans désactiver le sandbox, sans adopter un processus externe et sans dépendre du balayage placé après `cef::shutdown()`.

La fusion de ce jalon ferme l'exception transitoire du jalon 1. Elle exige le chemin normal `Ready supervisé` prouvé sur Windows et macOS. Le jalon 2 n'en dépend pas ; le jalon 4 et toute nouvelle release publique en dépendent.

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

## Défense locale avant lancement

Le mode dégradé n'essaie jamais de deviner l'identité d'un helper à partir de son nom, de son PID seul ou du callback parent, qui ne fournit pas à lui seul une preuve de processus stable.

Sur une machine Windows ou macOS, le runtime possède exactement deux résultats avant initialisation :

1. `Ready supervisé` : tous les prérequis natifs sont créés avant CEF et les smoke tests prouvent la publication, l'admission et la terminaison des types de helpers réellement utilisés ;
2. `Unavailable avant lancement` : un prérequis local a échoué, le navigateur intégré reste indisponible sur cette machine et Beaver continue normalement, sans avoir lancé ni admis de helper CEF.

Une impossibilité détectée avant `cef::initialize` — création du traqueur, des objets sécurisés, du reaper ou échec d'un prérequis natif Windows — sélectionne `Unavailable avant lancement`. Le runtime marque la capacité navigateur indisponible, comme il sait déjà le faire, et l'interface utilise son message traduit existant. La politique de capacité est centralisée et choisie avant CEF ; elle ne peut pas basculer silencieusement après la création d'un helper.

Si les tests natifs montrent qu'un type sandboxé ne peut pas publier avec les droits minimaux, la fusion du jalon et la release sont bloquées. Le sandbox n'est jamais désactivé, les droits ne sont jamais élargis et aucun suivi approximatif n'est ajouté. Le calendrier est ajusté ; le navigateur n'est pas retiré de la plateforme livrée.

Après qu'un processus a potentiellement été créé, une publication invalide, une identité ambiguë ou une admission impossible ne peut plus être convertie en simple indisponibilité silencieuse. Le bootstrap est arrêté avant CEF quand son identité est sûre ; si CEF est déjà actif ou si un enfant connu peut avoir échappé à l'admission, la porte ferme et Beaver déclenche immédiatement une vraie fermeture coordonnée.

Le comportement supposé d'auto-terminaison des processus Chromium n'est jamais utilisé comme preuve contractuelle. Le repli sûr est l'absence de lancement, pas l'espoir qu'un helper non supervisé finira par sortir.

L'état `Unavailable avant lancement` écrit uniquement une catégorie locale bornée et nettoyée — création d'objet, permission, admission, reaper ou sandbox — sans chemin, jeton ni détail système sensible. Ces catégories permettent de distinguer une machine réellement défaillante d'une incompatibilité de livraison sans ajouter de télémétrie distante.

La preuve de livraison ne se limite pas aux runners CI propres. Le build empaqueté est exercé sur une machine Windows avec les protections Microsoft actives, puis sur au moins un environnement renforcé représentatif — antivirus tiers ou politique d'entreprise — ainsi que sur un macOS avec Gatekeeper et quarantaine actifs. Les produits et versions observés sont consignés dans la Git note. Une incompatibilité reproductible sur une configuration normalement supportée bloque la fusion ; elle n'est pas reclassée en simple panne locale.

## Contraintes Windows

L'exécutable Beaver sert aussi au bac à sable shell. Le filtre CEF exige donc réservation, génération, parent, heure de démarrage et exécutable canonique ; le chemin seul est insuffisant. L'autorité et les handles restent exclusivement dans le parent. La zone modifiable par le helper ne porte jamais un handle, un état d'admission ou les identifiants d'un autre slot.

Avant `Admitted`, le parent ouvre l'identité stable puis place le helper dans son Job Object propre, compatible avec le job du sandbox Chromium actif. Tout échec ferme les guards locaux et refuse l'admission. La sortie brute du parent ferme le dernier handle non héritable du Job.

## Contraintes macOS

Le helper ouvre sa boîte, sa page de contrôle et ses événements, puis crée son groupe avant `sandbox.initialize`. Il publie seulement après l'application du sandbox. Le parent revérifie PID, parent, heure de démarrage, exécutable et PGID avant `Admitted`.

Le reaper parent précréé rescane les générations admises et revalide l'identité avant chaque signal. Le moniteur du helper s'auto-termine si le parent disparaît ou au plus tard une seconde avant la sortie brute. Il ne suppose pas que Seatbelt l'autorise à signaler tout son groupe.

## Contrat du dossier Cargo Windows

Le build empaqueté Windows conserve un dossier Cargo court afin de ne pas réintroduire les dépassements de chemins rencontrés pendant la compilation CEF. `scripts/cef/tauri-launch.mjs` reste l'unique autorité qui choisit ce dossier : valeur configurée validée lorsqu'elle existe, sinon `target` à la racine du worktree. La valeur effective est absolue et transmise par `CARGO_TARGET_DIR` à Cargo ainsi qu'à chaque étape qui relit un artefact compilé.

Les consommateurs n'inventent plus leur propre chemin :

- le préparateur de l'updater lit son binaire dans le dossier Cargo effectif ;
- `prepare-cef-windows.ps1` lit `cl_go_dash_lib.dll` et `cl-go-dash.exe` dans ce même dossier ;
- les dossiers `src-tauri/target/updater-helper` et `src-tauri/target/cef-runtime/windows` restent des destinations de ressources Tauri, pas des dossiers d'artefacts Cargo ;
- en exécution directe sans `CARGO_TARGET_DIR`, les scripts conservent le repli historique `src-tauri/target` ;
- macOS, Linux, le profil E2E et les runtimes d'extensions ne changent pas de dossier dans ce correctif.

Toute valeur configurée est bornée, sans caractère de contrôle ni segment `..`, normalisée une seule fois par le lanceur puis revalidée avant lecture. Un chemin absent, ambigu, lié ou incohérent fait échouer la préparation ; aucun script ne cherche silencieusement dans un second dossier.

La preuve comprend les dossiers par défaut et personnalisés, une cible Rust explicite, les chemins invalides, un contrat interdisant aux lecteurs d'artefacts Windows de recoder `src-tauri/target/release`, puis un vrai `npm run tauri build`. La CI complète Windows/macOS/Linux est rejouée après la correction.

## Tests obligatoires

- arrêt natif CEF normal suivi du balayage final dans l'ordre existant ;
- `cef::shutdown()` bloqué avec helper réel admis, rendu non exécutable avant la sortie forcée ;
- callback sans ligne de commande exploitable, marqueur impossible, capacité saturée et identité ambiguë : aucun appel CEF de repli ;
- callback après `Closing`, réservation expirée et publication après 13 secondes : génération invalide et sortie avant CEF ;
- permis encore inachevé après la barrière de 50 millisecondes : admission tardive impossible ;
- candidat jamais publié juste avant 15 secondes : aucun appel CEF et disparition dans la fenêtre de constat ;
- helpers CEF et shell simultanés sous Windows : seul le rôle réservé est adopté ;
- chaque type CEF Windows réel publie en état `Ready supervisé` avec le sandbox actif, les SIDs de restriction et le MIC minimal ; un échec local injecté avant initialisation prouve séparément qu'aucun helper n'est créé ;
- corruption de boîte, réécriture après scellement, faux handle et tentative inter-slot : aucune autorité parent modifiée et aucun processus externe signalé ;
- Job Object imbriqué Windows réussi avec le sandbox actif ; échec d'affectation fermé ;
- `TerminateProcess` accepté mais handle pas encore signalé : slot conservé puis revérifié ;
- panne du traqueur et watchdog général bloqué : Job Object Windows ou reaper macOS reste efficace ;
- macOS : objets et groupe avant sandbox, publication et moniteur après sandbox, PGID réutilisé refusé ;
- reaper macOS absent avant initialisation dans le test injecté : CEF indisponible localement et aucun helper ; reaper arrêté après ouverture : fermeture coordonnée ;
- échec de chaque prérequis avant initialisation : `BrowserCapability::Unavailable`, Beaver utilisable et aucun helper créé ;
- test Linux confirmant que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé ;
- smoke test natif `Ready supervisé` obligatoire sur Windows et macOS ; l'échec injecté avant initialisation couvre séparément `Unavailable avant lancement` sans en faire un état de livraison ;
- build empaqueté `Ready supervisé` avec protections système actives, puis Windows renforcé par antivirus tiers ou politique d'entreprise ; aucune désactivation du sandbox, aucune exception antivirus demandée à l'utilisateur ;
- catégories locales d'indisponibilité bornées, nettoyées et testées sans chemin ni détail interne ; aucune télémétrie distante ajoutée ;
- test Linux limité à l'absence de `native_browser` et de helper CEF, l'intégration Linux complète restant hors périmètre.

## Critères de fusion

- aucune identification fondée seulement sur un nom ou un PID ;
- aucune permission de sandbox élargie pour faire passer la publication ;
- Windows et macOS sont obligatoirement `Ready supervisé` avec leurs preuves natives ;
- la matrice réelle de compatibilité du build empaqueté est verte et consignée ; une incompatibilité reproductible sur une machine supportée bloque la fusion ;
- le chemin local `Unavailable avant lancement` est testé par injection et ne remplace jamais la preuve normale ;
- aucun helper CEF admis ne poursuit d'exécution dans le scénario d'arrêt natif bloqué ;
- tout objet noyau résiduel et tout bootstrap refusé disparaissent dans la fenêtre de constat ;
- aucune application externe ni aucun helper shell adopté ou signalé ;
- job CI macOS CEF, validations Windows natives et essais avec protections système réellement exécutés ;
- toutes les sous-lignes J1B de l'inventaire sont fermées et référencent leurs tests ;
- fichiers de production sous 230 lignes, suites complètes et CI native vertes ;
- Git note détaillant les preuves `Ready supervisé`, les environnements renforcés testés, la défense locale, les alternatives rejetées et la décision de ne pas livrer une plateforme avec le navigateur désactivé. Ce jalon est nécessaire mais ne suffit pas à autoriser une release, qui attend encore la validation du jalon 4.

## État factuel des preuves au 10 août 2026

La PR J1B compile et teste les chemins natifs avec CEF vérifié et sandbox actif. Le run GitHub Actions `31404848819` est vert sur Windows, macOS et Linux : Clippy strict, suites complètes voisines, autorité native Windows/macOS, contrats de sandbox et absence de CEF Linux. Les commits d'implémentation sont `0e505ca` à `4812d09`; les durcissements CI et multi-OS sont `84a7fb2`, `3f517a1`, `3936a14`, `baf14dc` et `1b4f15d`.

Cette preuve automatisée valide le protocole, les appels natifs et la terminaison de vrais processus enfants confinés. Elle ne remplace pas les essais de livraison du navigateur complet. Avant passage de la PR hors brouillon, il reste donc à consigner séparément :

- build empaqueté Windows avec protections Microsoft actives ;
- build empaqueté Windows dans un environnement renforcé représentatif ;
- build empaqueté macOS avec Gatekeeper et quarantaine actifs ;
- pour chaque essai, ouverture réelle du navigateur, navigation, fermeture de Beaver pendant que CEF est actif et constat d'absence de helper runnable.

Ces lignes ne peuvent être cochées ni déduites de la CI. Une Git note finale ne les déclare réussies qu'avec l'environnement réellement observé ; sinon la PR reste en brouillon.
