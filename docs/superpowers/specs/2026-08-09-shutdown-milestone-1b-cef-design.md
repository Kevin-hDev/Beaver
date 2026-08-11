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

La prévalidation distingue les erreurs déterministes des erreurs transitoires avant tout appel CEF et avant tout lancement de helper. Une erreur déterministe — runtime absent ou corrompu, permission incompatible, identité ambiguë, sandbox indisponible ou contrat de sécurité invalide — échoue immédiatement. Une erreur transitoire explicitement reconnue par son code OS reçoit exactement une seconde tentative après une attente centrale de 200 millisecondes. Aucune chaîne de message n'est utilisée pour décider du retry, aucune troisième tentative n'existe et aucun état natif de la première tentative ne peut être réutilisé.

`cef::initialize` reste appelé une seule fois par processus Beaver. S'il retourne faux, Beaver n'appelle plus aucune fonction CEF, y compris `cef::shutdown`, puis lance une fermeture coordonnée. Il n'existe aucun retry de `cef::initialize` dans la même session : le contrat CEF impose alors la sortie du processus. La relance complète de Beaver est l'unique nouvelle tentative sûre après cette frontière.

Quand seule la prévalidation échoue, Beaver reste utilisable et émet un code d'événement public stable sans détail système. L'interface réutilise le composant de notification existant et son design, enrichi d'une action optionnelle générique. La notification traduite dans les sept langues n'est ni modale ni bloquante, peut être fermée immédiatement et disparaît entièrement après 10 secondes. Son bouton `Redémarrer` disparaît avec elle ; aucun bouton permanent n'est ajouté à l'écran du navigateur. L'action passe par le coordinateur de fermeture, attend le nettoyage normal, puis seulement demande le redémarrage Tauri. Si l'échéance ultime force la sortie, la sécurité gagne et aucune relance n'est promise.

Si les tests natifs montrent qu'un type sandboxé ne peut pas publier avec les droits minimaux, la fusion du jalon et la release sont bloquées. Le sandbox n'est jamais désactivé, les droits ne sont jamais élargis et aucun suivi approximatif n'est ajouté. Le calendrier est ajusté ; le navigateur n'est pas retiré de la plateforme livrée.

Après qu'un processus a potentiellement été créé, une publication invalide, une identité ambiguë ou une admission impossible ne peut plus être convertie en simple indisponibilité silencieuse. Le bootstrap est arrêté avant CEF quand son identité est sûre ; si CEF est déjà actif ou si un enfant connu peut avoir échappé à l'admission, la porte ferme et Beaver déclenche immédiatement une vraie fermeture coordonnée.

Le comportement supposé d'auto-terminaison des processus Chromium n'est jamais utilisé comme preuve contractuelle. Le repli sûr est l'absence de lancement, pas l'espoir qu'un helper non supervisé finira par sortir.

L'état `Unavailable avant lancement` écrit uniquement une catégorie locale bornée et nettoyée — création d'objet, permission, admission, reaper ou sandbox — sans chemin, jeton ni détail système sensible. Ces catégories permettent de distinguer une machine réellement défaillante d'une incompatibilité de livraison sans ajouter de télémétrie distante.

La preuve de livraison ne se limite pas aux runners CI propres. Le build empaqueté est exercé sur une machine Windows avec les protections Microsoft actives, puis sur au moins un environnement renforcé représentatif — antivirus tiers ou politique d'entreprise — ainsi que sur un macOS avec Gatekeeper et quarantaine actifs. Les produits et versions observés sont consignés dans la Git note. Une incompatibilité reproductible sur une configuration normalement supportée bloque la fusion ; elle n'est pas reclassée en simple panne locale.

## Contraintes Windows

L'exécutable Beaver sert aussi au bac à sable shell. Le filtre CEF exige donc réservation, génération, parent, heure de démarrage et exécutable canonique ; le chemin seul est insuffisant. L'autorité et les handles restent exclusivement dans le parent. La zone modifiable par le helper ne porte jamais un handle, un état d'admission ou les identifiants d'un autre slot.

Le bootstrap possède un routeur de rôle unique, exécuté avant toute branche CEF : parent Beaver, helper CEF réservé, helper d'isolation du shell ou entrée invalide. Les rôles sont mutuellement exclusifs. `--beaver-shell-sandbox` atteint le helper shell dans les builds de développement et empaquetés, sans marqueur CEF et sans occuper un slot CEF ; toute combinaison shell + marqueur/type CEF est refusée. Un test empaqueté avec accès disque restreint exécute une commande inoffensive et prouve que le helper shell réel est atteint.

Le classificateur CEF parcourt la ligne de commande sans la recopier et ne borne que les arguments privés Beaver qu'il interprète. Il ne rejette plus une ligne Chromium à cause d'un nombre arbitraire d'arguments, d'un argument opaque long ou d'un argument sans représentation Unicode. Le transfert du bootstrap de développement reste borné par les limites réelles de `CreateProcess` — 32 767 unités UTF-16 au total — et refuse toujours les substitutions de module. Les tests couvrent plus de 64 arguments CEF valides et les limites réelles Windows.

Avant `Admitted`, le parent ouvre l'identité stable puis place le helper dans son Job Object propre, compatible avec le job du sandbox Chromium actif. Tout échec ferme les guards locaux et refuse l'admission. La sortie brute du parent ferme le dernier handle non héritable du Job.

## Contraintes macOS

Le helper ouvre sa boîte, sa page de contrôle et ses événements, puis crée son groupe avant `sandbox.initialize`. Il publie seulement après l'application du sandbox. Le parent revérifie PID, parent, heure de démarrage, exécutable et PGID avant `Admitted`.

Le reaper parent précréé rescane les générations admises et revalide l'identité avant chaque signal. Le moniteur du helper s'auto-termine si le parent disparaît ou au plus tard une seconde avant la sortie brute. Il ne suppose pas que Seatbelt l'autorise à signaler tout son groupe.

Le reaper est un thread distinct du traqueur normal, créé et validé pendant la prévalidation. Il possède son propre contrôle atomique et lit directement une table d'urgence parent privée à capacité fixe ; `emergency_force` ne se contente jamais de poser un drapeau destiné au traqueur normal. Une panique ou un arrêt injecté du traqueur ferme la porte et déclenche la fermeture coordonnée, mais ne détruit ni la table ni le reaper. Au passage à `Closing`, le parent écrit l'échéance absolue et signale la fermeture dans chaque objet publié avant d'armer le reaper. Le moniteur de chaque helper conserve ainsi une seconde preuve indépendante et s'auto-termine à 14 secondes ou dès la disparition du parent.

La barrière CEF de 50 millisecondes est une échéance absolue dérivée de la `ShutdownTimeline`, jamais une valeur locale. Elle ferme d'abord toutes les portes puis invalide les générations non admises. Si un permis antérieur n'a pas fini à l'échéance, ce dépassement est journalisé sans détail sensible et le nettoyage coordonné continue : il ne devient jamais une `InvariantViolation` ni une sortie brute immédiate. Le détenteur tardif observe la porte fermée ou la génération invalidée, nettoie ses objets et ne peut pas atteindre CEF. Seule une corruption réelle de l'autorité monotone peut emprunter le filet de sortie brute.

## Contrat du dossier Cargo Windows

Le build empaqueté Windows conserve un dossier Cargo court afin de ne pas réintroduire les dépassements de chemins rencontrés pendant la compilation CEF. `scripts/cef/tauri-launch.mjs` reste l'unique autorité locale qui choisit ce dossier : valeur configurée validée lorsqu'elle existe, sinon `target` à la racine du worktree. La valeur effective est absolue et transmise par `CARGO_TARGET_DIR` à Cargo ainsi qu'à chaque étape qui relit un artefact compilé. Un job CI qui sépare le build de la relecture configure cette même valeur au niveau du job, avant le build, car l'environnement d'un processus enfant ne remonte pas vers une étape sœur.

Les consommateurs n'inventent plus leur propre chemin :

- le préparateur de l'updater lit son binaire dans le dossier Cargo effectif ;
- `prepare-cef-windows.ps1` lit `cl_go_dash_lib.dll` et `cl-go-dash.exe` dans ce même dossier ;
- les dossiers `src-tauri/target/updater-helper` et `src-tauri/target/cef-runtime/windows` restent des destinations de ressources Tauri, pas des dossiers d'artefacts Cargo ;
- le préparateur updater peut conserver le repli historique `src-tauri/target` en exécution directe, car il recompile lui-même le binaire exact avant de le copier ;
- `prepare-cef-windows.ps1` refuse au contraire de relire un artefact sans `CARGO_TARGET_DIR` explicite : il consomme l'exécutable principal construit par Tauri et ne doit jamais accepter silencieusement un ancien `src-tauri/target` ;
- macOS, Linux, le profil E2E et les runtimes d'extensions ne changent pas de dossier dans ce correctif.

Toute valeur configurée est bornée, sans caractère de contrôle ni segment `..`, normalisée une seule fois par le lanceur puis revalidée avant lecture. Un chemin absent, ambigu, lié ou incohérent fait échouer la préparation ; aucun script ne cherche silencieusement dans un second dossier.

La preuve comprend les dossiers par défaut et personnalisés, une cible Rust explicite, les chemins invalides, le refus CEF sans autorité explicite, la propagation entre étapes sœurs du job de release, un contrat interdisant aux lecteurs d'artefacts Windows de recoder `src-tauri/target/release`, puis un vrai `npm run tauri build`. La CI complète Windows/macOS/Linux est rejouée après la correction.

## Tests obligatoires

- arrêt natif CEF normal suivi du balayage final dans l'ordre existant ;
- `cef::shutdown()` bloqué avec helper réel admis, rendu non exécutable avant la sortie forcée ;
- callback sans ligne de commande exploitable, marqueur impossible, capacité saturée et identité ambiguë : aucun appel CEF de repli ;
- callback après `Closing`, réservation expirée et publication après 13 secondes : génération invalide et sortie avant CEF ;
- permis encore inachevé après la barrière de 50 millisecondes : admission tardive impossible ;
- permis encore inachevé après la barrière : Ollama, SearXNG et l'hôte d'extensions suivent tout de même le nettoyage coordonné ; aucune sortie brute anticipée ;
- candidat jamais publié juste avant 15 secondes : aucun appel CEF et disparition dans la fenêtre de constat ;
- helpers CEF et shell simultanés sous Windows : seul le rôle réservé est adopté ;
- bootstrap Windows empaqueté : le rôle shell isolé atteint son helper, le rôle parent démarre Beaver et chaque rôle CEF réel atteint son admission sans collision ;
- plus de 64 arguments Chromium opaques, argument long non privé et argument opaque non Unicode : le marqueur privé reste détecté sans refus de la ligne complète ;
- chaque type CEF Windows réel publie en état `Ready supervisé` avec le sandbox actif, les SIDs de restriction et le MIC minimal ; un échec local injecté avant initialisation prouve séparément qu'aucun helper n'est créé ;
- corruption de boîte, réécriture après scellement, faux handle et tentative inter-slot : aucune autorité parent modifiée et aucun processus externe signalé ;
- Job Object imbriqué Windows réussi avec le sandbox actif ; échec d'affectation fermé ;
- `TerminateProcess` accepté mais handle pas encore signalé : slot conservé puis revérifié ;
- panne du traqueur et watchdog général bloqué : Job Object Windows ou reaper macOS reste efficace ;
- panique du traqueur macOS : le reaper distinct reçoit directement l'échéance, rend le groupe non exécutable et le moniteur du helper sort au plus tard à 14 secondes ;
- macOS : objets et groupe avant sandbox, publication et moniteur après sandbox, PGID réutilisé refusé ;
- reaper macOS absent avant initialisation dans le test injecté : CEF indisponible localement et aucun helper ; reaper arrêté après ouverture : fermeture coordonnée ;
- échec de chaque prérequis avant initialisation : `BrowserCapability::Unavailable`, Beaver utilisable et aucun helper créé ;
- erreur transitoire de prévalidation : une seule seconde tentative propre ; erreur déterministe : aucun retry ; `cef::initialize` faux : aucune seconde initialisation ni `cef::shutdown`, puis fermeture coordonnée ;
- notification d'indisponibilité : composant visuel existant, sept langues, action de redémarrage coordonnée, fermeture manuelle et disparition complète à 10 secondes sans bloquer le reste de l'interface ;
- test Linux confirmant que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé ;
- smoke test natif `Ready supervisé` obligatoire sur Windows et macOS ; l'échec injecté avant initialisation couvre séparément `Unavailable avant lancement` sans en faire un état de livraison ;
- les filtres CI énumèrent d'abord leurs tests et échouent si le nombre attendu n'est pas présent ; aucun succès avec zéro test n'est accepté ;
- le contrat Node est lancé comme processus direct sur Windows, macOS et Linux avec une fixture volontairement invalide qui doit produire un code non nul ;
- un smoke test du build empaqueté lance un vrai enfant CEF avec sa ligne de commande réelle, vérifie chaque type observé, ouvre une page, puis ferme Beaver pendant que CEF est actif ; les tests de protocole avec `windows-tests` restent complémentaires et ne sont jamais présentés comme cette preuve ;
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
- chaque commit qui porte une décision de sécurité ou de cycle de vie reçoit une Git note ; la note finale relie ces décisions à leurs tests et à la matrice manuelle réelle ;
- l'avertissement `__TAURI_BUNDLE_TYPE` du bootstrap Windows possède une preuve de cause et un test d'empaquetage. La PR ne passe pas hors brouillon tant que le bootstrap et le module qui exécute Tauri ne reçoivent pas le type de bundle attendu, ou qu'une solution officielle équivalente n'est pas démontrée ; l'avertissement ne peut pas être seulement consigné dans une note.

## État factuel des preuves au 11 août 2026

La PR J1B compile les chemins natifs avec CEF vérifié et sandbox actif. Le run GitHub Actions `31404848819` est vert sur Windows, macOS et Linux : Clippy strict, suites complètes voisines, tests des autorités natives Windows/macOS, contrats de sandbox et absence de CEF Linux. Les commits d'implémentation sont `0e505ca` à `4812d09`; les durcissements CI et multi-OS sont `84a7fb2`, `3f517a1`, `3936a14`, `baf14dc` et `1b4f15d`.

Les corrections suivantes ferment désormais les angles rouverts par la review complète : la barrière CEF continue le nettoyage après son échéance, le reaper macOS possède une voie d'urgence indépendante, le routeur Windows sépare les rôles CEF et shell sans plafonner la ligne Chromium, l'entrée directe du contrat Node échoue réellement et les jobs natifs ouvrent une vraie page CEF avant une fermeture coordonnée. Les filtres Rust échouent aussi lorsqu'ils ne sélectionnent aucun test.

Le type de bundle Tauri possède une autorité unique. Chaque invocation Windows construit exactement un type : NSIS par défaut ou MSI explicitement demandé. Le bootstrap porte le marqueur officiel inconnu que Tauri remplace, tandis que seule la copie de la DLL destinée au paquet reçoit le même type. Le vérificateur compare les deux fichiers empaquetés à leurs références issues du même build et refuse toute différence autre que ce remplacement. Les preuves locales comprennent un NSIS construit, installé, vérifié puis désinstallé, ainsi qu'un MSI construit, extrait et vérifié ; aucun avertissement `__TAURI_BUNDLE_TYPE` n'est masqué.

Ces preuves automatisées et locales ne remplacent ni la CI du HEAD final ni la matrice manuelle renforcée. Avant passage de la PR hors brouillon, il reste à consigner séparément :

- build empaqueté Windows avec protections Microsoft actives ;
- build empaqueté Windows dans un environnement renforcé représentatif ;
- build empaqueté macOS avec Gatekeeper et quarantaine actifs ;
- pour chaque essai, ouverture réelle du navigateur, navigation, fermeture de Beaver pendant que CEF est actif et constat d'absence de helper runnable.

Ces lignes ne peuvent être cochées ni déduites de la CI. Une Git note finale ne les déclare réussies qu'avec l'environnement réellement observé ; sinon la PR reste en brouillon.
