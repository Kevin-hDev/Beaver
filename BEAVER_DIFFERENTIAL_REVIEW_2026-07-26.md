# Audit différentiel de la migration Beaver

Date : 26 juillet 2026

Branche : `codex/beaver-migration`

Base : `origin/main` à `1a8a5843919bcee34dbba571920deebc5b19996f`

Sommet audité : `21822ab0e9e74f89dcaca02c3ec325d74669b482`

Portée : 18 commits, 188 fichiers, 11 256 ajouts et 1 142 suppressions

## Résumé initial avant corrections

La migration est techniquement bien plus sûre qu'un renommage global naïf. Les
identités qui protègent les données existantes sont conservées, l'updater a été
fortement durci, les tests automatisés sont très fournis et le paquet macOS
Beaver est construit et validé correctement.

La branche ne doit toutefois pas encore être publiée. L'audit confirme trois
portes bloquantes :

1. des textes Git créés par les sous-agents portent encore le nom CL-GO, et le
   garde-fou de marque ne bloque pas cette régression ;
2. le retour arrière annoncé n'existe réellement que sur macOS, pas sur Linux
   ni Windows ;
3. les migrations natives depuis une vraie installation CL-GO n'ont pas encore
   été exécutées sur les trois systèmes.

Aucune faille critique n'a été trouvée dans le nouveau code de mise à jour.
Aucun indice de perte ou de déplacement involontaire des données utilisateur
n'a été trouvé. Il reste néanmoins des risques résiduels liés à l'absence de
signature publique des installateurs, à des dépendances héritées et aux
comportements propres à chaque OS.

Verdict actuel : **NO-GO pour une publication, GO pour une phase de correction
et de validation native**.

## État après le lot de corrections

Le lot lancé après cet audit ferme les oublis qui pouvaient être corrigés
localement sans augmenter le risque :

| ID | État après correction | Décision |
|---|---|---|
| F-01 | Corrigé | L'identité et les messages Git générés sont centralisés sous Beaver. |
| F-02 | Contrat corrigé | Tous les OS vérifient le redémarrage. Seul macOS promet un retour arrière automatique ; Linux et Windows ont une procédure de récupération explicite. |
| F-03 | Ouvert | La matrice réelle sur trois OS reste obligatoire avant publication. |
| F-04 | Corrigé | Toute modification d'une mention visible historique fait maintenant échouer le contrôle de marque, même si le nombre total reste identique. |
| F-05 | Corrigé | Le script macOS/Linux est limité aux installations fraîches, revérifie l'absence d'installation juste avant la copie finale, contrôle l'identité du dossier déplacé et vérifie l'exécutable déclaré par le bundle. |
| F-06 | Risque accepté | L'absence de signature publique reste documentée pour ce projet personnel open source. |
| F-07 | Corrigé dans le code | Si l'ancienne entrée ne peut pas être supprimée, l'entrée Beaver ajoutée est retirée et la migration sera retentée. Les essais natifs restent requis. |
| F-08 | Partiellement corrigé | Les mises à jour sûres et les dépendances Rust inutilisées sont traitées. La pile de lint npm et `glib` restent suivies sans forcer de rupture majeure. |
| F-09 | Corrigé | Le test verrouille désormais chaque référence Rust à `data_dir()`, y compris les constructions non littérales. |
| F-10 | Corrigé dans le plan | Les cases reflètent le code terminé et restent ouvertes pour les validations manuelles. |
| F-11 | Corrigé | Les tests trop longs sont découpés et le log Ollama non sûr est supprimé. |
| F-12 | Corrigé | Les quatre dépendances de test inutilisées sont retirées du manifeste et du lock. |

Le verdict de publication reste **NO-GO** tant que F-03 et les autres contrôles
natifs encore décochés dans le plan ne sont pas exécutés. Le code peut rester
dans la CL brouillon pour cette phase de validation.

## Tableau initial des constats

| ID | Niveau | Constat | État |
|---|---|---|---|
| F-01 | Élevé | Des auteurs et messages Git générés restent nommés CL-GO | À corriger |
| F-02 | Élevé | Linux et Windows vérifient le redémarrage mais ne restaurent pas l'ancienne version | À décider et corriger |
| F-03 | Élevé | La vraie chaîne CL-GO vers Beaver n'a pas encore été testée sur les trois OS | Bloquant avant publication |
| F-04 | Moyen | La CI de marque tolère toute nouvelle occurrence « visible à renommer » | À corriger |
| F-05 | Moyen | L'installateur manuel macOS supprime sa sauvegarde après un simple lancement | À durcir ou documenter |
| F-06 | Moyen | La chaîne de confiance repose sur GitHub et un manifeste non signé | Risque accepté ou signature à ajouter |
| F-07 | Moyen | Un échec de nettoyage autostart peut laisser deux entrées actives temporairement | À rendre visible et retester |
| F-08 | Moyen | Une alerte Rust runtime Linux et des alertes d'outils npm restent ouvertes | Hérité, à traiter |
| F-09 | Faible | Le test d'inventaire du stockage ne détecte que les appels littéraux `data_dir().join(...)` | À renforcer |
| F-10 | Faible | Le plan, la CL et les cases cochées ne reflètent plus exactement l'état réel | À remettre à jour |
| F-11 | Faible | Trois fichiers modifiés atteignent ou dépassent la limite locale de 200 lignes | À découper |
| F-12 | Faible | Quatre dépendances Rust de test sont réellement inutilisées | Nettoyage sûr |

## Ce qui est solide

### Compatibilité des données

Les contrats importants restent stables :

- le dossier de données reste `~/.local/share/cl-go-dash/` ;
- le bundle ID reste `com.clgo.dash` ;
- le service keyring reste `cl-go-dash` ;
- l'exécutable interne reste `cl-go-dash` ;
- les clés `localStorage`, les identités OAuth, les branches de sous-agents et
  les dossiers `.cl-go` restent inchangés ;
- aucune nouvelle racine de stockage Beaver n'a été trouvée.

Le profil synthétique couvre 25 domaines et 62 fichiers représentatifs. Les
tests de compatibilité comparent aussi des extraits avec le commit CL-GO de
référence.

### Sécurité de l'updater

Le nouveau chemin de mise à jour apporte de vrais progrès :

- dépôt, hôtes, tags et noms d'assets strictement validés ;
- réponses, listes, manifestes et téléchargements bornés ;
- redirections limitées à des hôtes GitHub précis ;
- SHA-256 obligatoire et comparaison en temps constant ;
- fichiers temporaires privés, noms aléatoires et chemins vérifiés ;
- helper autonome sans commande shell construite depuis une entrée distante ;
- arguments système séparés ;
- jeton de santé généré aléatoirement, borné, zéroïsé et comparé en temps
  constant ;
- rollback transactionnel complet du bundle sur macOS ;
- erreurs visibles génériques.

L'analyse Semgrep de 58 fichiers sensibles avec 206 règles n'a trouvé aucun
signal.

### Packaging macOS

Le build local complet a produit :

- `src-tauri/target/release/bundle/macos/Beaver.app` ;
- `src-tauri/target/release/bundle/dmg/Beaver_1.0.2_aarch64.dmg`.

Les contrôles ont confirmé :

- `CFBundleDisplayName = Beaver` ;
- `CFBundleIdentifier = com.clgo.dash` ;
- `CFBundleExecutable = cl-go-dash` ;
- les cinq helpers CEF sont nommés Beaver ;
- la signature ad hoc interne est cohérente ;
- le DMG a une somme de contrôle valide ;
- les icônes et les noms de fichiers attendus sont présents.

## Détail des constats

### F-01 — Textes Git CL-GO encore générés

Le plan exige explicitement de renommer les auteurs et messages Git visibles
dans `docs/BEAVER-RENAME-PLAN.md:829-835`.

Les omissions confirmées sont notamment :

- `src-tauri/src/services/agent_local/subagent_directory_workspace.rs:47-54` ;
- `src-tauri/src/services/agent_local/subagent_directory_change.rs:115-124` ;
- `src-tauri/src/services/agent_local/subagent_git_command.rs:29-39` ;
- `src-tauri/src/services/agent_local/subagent_git_run.rs:107-126`.

Ces fichiers créent encore des commits avec `user.name=CL-GO`, l'adresse
`cl-go@local`, des titres `CL-GO temporary...` et le marqueur visible
`CL-GO-Subagent-Change`.

Impact : l'interface principale est bien Beaver, mais l'utilisateur peut encore
voir CL-GO dans l'historique Git créé après la migration.

Correction recommandée :

- centraliser l'auteur et les messages générés dans les constantes de marque ;
- conserver uniquement les vraies identités de compatibilité documentées,
  comme les noms de branches `cl-go/subagent/*` ;
- ajouter des tests qui couvrent les quatre chemins de création de commits.

### F-02 — Retour arrière absent sur Linux et Windows

Sur macOS, la transaction conserve l'ancien bundle jusqu'au signal de santé et
le restaure en cas d'échec.

Sur Linux et Windows, l'installateur est appliqué avant le test de santé. En cas
d'échec, le nouveau processus est tué, mais l'ancienne installation n'est pas
restaurée :

- `src-tauri/src/updater_worker/linux.rs:12-25` ;
- `src-tauri/src/updater_worker/windows.rs:12-24`.

Cette réalité contredit les textes destinés aux utilisateurs :

- `CHANGELOG.md:58-61` ;
- `app-release-notes.json:8` ;
- `app-release-notes.json:15` ;
- `app-release-notes.json:22` ;
- `app-release-notes.json:29` ;
- `app-release-notes.json:36` ;
- `app-release-notes.json:43` ;
- `app-release-notes.json:50`.

Impact : un paquet Beaver valide mais incapable de redémarrer peut remplacer
CL-GO sur Linux ou Windows sans restauration automatique, alors que les notes
promettent le contraire.

Décision nécessaire avant publication :

1. soit implémenter une vraie sauvegarde/restauration sur Linux et Windows ;
2. soit annoncer honnêtement une vérification de redémarrage sans rollback,
   fournir une procédure de récupération et supprimer toute promesse globale de
   restauration automatique.

Pour une migration qui minimise réellement le risque, la première option est
préférable.

### F-03 — Migrations natives réelles encore non exécutées

La CI construit sur macOS, Linux et Windows et inspecte les artefacts. Elle
n'exécute cependant pas une installation CL-GO réelle suivie d'une migration
Beaver complète :

- macOS : inspection du bundle seulement, `.github/workflows/release.yml:203-207` ;
- Linux : inspection du paquet et des métadonnées, `.github/workflows/release.yml:209-214` ;
- Windows : installation Beaver fraîche et contrôle du hook,
  `.github/workflows/release.yml:216-231`.

Les scénarios suivants restent donc manuels :

- CL-GO `1.0.1` vers la version-pont `1.0.2` ;
- CL-GO `1.0.2` vers Beaver `1.1.0` ;
- reprise d'un profil réel contenant sessions, coffre, OAuth, modèles, projets,
  worktrees, navigateur et réglages ;
- absence de doublon d'application, raccourci et autostart ;
- conservation des autorisations OS, notamment macOS ;
- échec volontaire au premier démarrage et récupération ;
- installation fraîche sur les trois OS.

Ces tests doivent être réalisés sur des copies de données et des machines ou VM
contrôlées avant de rendre une release publique.

### F-04 — Le garde-fou de marque ne bloque pas les oublis visibles

Le scanner classe correctement les références en trois groupes. Son état actuel
est :

- 149 références visibles à renommer ou justifier ;
- 457 références internes à conserver ;
- 0 référence inconnue.

Le test du dépôt ne vérifie pourtant que :

- les contrats explicites ;
- l'absence de références inconnues ;
- le nombre exact de références internes.

Il ne vérifie jamais le groupe visible :
`scripts/brand/brand-boundaries.test.mjs:187-202`.

Conséquence : ajouter demain un nouveau `tooltip("CL-GO")` produit une référence
visible, mais la CI continue de passer. C'est exactement ce qui a permis aux
messages Git de F-01 de rester présents.

Correction recommandée : maintenir une allowlist explicite des mentions
historiques visibles, par fichier et contexte, et faire échouer le test sur
toute autre occurrence.

### F-05 — Installateur manuel macOS moins transactionnel que l'updater

L'installateur `install.sh` vérifie le SHA-256, le bundle ID et l'exécutable,
puis conserve une sauvegarde de `Beaver.app`. Il supprime toutefois cette
sauvegarde dès que `/usr/bin/open -n` accepte le lancement :
`install.sh:107-141`.

Il n'attend pas le signal de santé utilisé par l'updater. Il ne supprime pas non
plus une ancienne `CL-GO.app`, ce qui peut laisser deux applications si un
utilisateur existant emploie par erreur le chemin « première installation ».

Correction recommandée :

- réserver et documenter strictement ces scripts aux installations fraîches ;
- ou réutiliser le protocole de santé et ne nettoyer l'ancien bundle qu'après
  confirmation ;
- vérifier aussi `CFBundleExecutable` dans le plist, comme le fait l'updater.

### F-06 — Le manifeste ne protège pas contre un compte GitHub compromis

Le manifeste et les installateurs sont publiés dans la même release GitHub.
Cela protège bien contre une corruption accidentelle ou un téléchargement
tronqué. Cela ne protège pas contre un attaquant capable de remplacer à la fois
l'asset et son manifeste.

Le projet utilise une signature macOS ad hoc
(`src-tauri/tauri.conf.json:65-68`) et documente l'absence de signature publique
dans `SECURITY.md:136-140`.

Risque résiduel :

- compromission du compte ou du dépôt GitHub ;
- Gatekeeper/SmartScreen et avertissements variables selon l'OS ;
- autorisations macOS potentiellement redemandées après changement de chemin.

Pour un projet personnel open source, ce risque peut être accepté. Il ne peut
pas être présenté comme nul. La réduction maximale passe par une signature de
publication distincte de GitHub et, idéalement, la notarisation macOS.

### F-07 — Deux autostarts possibles après une erreur de nettoyage

La migration active Beaver avant de désactiver CL-GO afin de préserver la
continuité. Si la désactivation de l'ancienne entrée échoue, les deux peuvent
rester actives jusqu'à une nouvelle tentative :

- comportement : `src-tauri/src/services/autostart_migration.rs:118-146` ;
- test qui l'entérine :
  `src-tauri/src/services/autostart_migration_tests.rs:112-129`.

Ce compromis est raisonnable, mais la promesse « une seule entrée » n'est pas
absolue.

Correction recommandée : conserver la stratégie de continuité, mais rendre
l'échec visible, retenter au prochain démarrage et couvrir le résultat sur les
trois OS.

### F-08 — Alertes de dépendances héritées

`npm audit` remonte 11 alertes hautes dans les outils de développement. Le lock
n'a pas changé hors numéro de version pendant cette migration et
`npm audit --omit=dev` remonte zéro vulnérabilité embarquée.

Dependabot remonte aussi une alerte moyenne runtime Linux :

- `glib 0.18.5`, GHSA-wrw7-89jp-8q8g ;
- dépendance transitive de la pile GTK/Tauri Linux ;
- aucun appel direct à `glib` ou `VariantStrIter` dans le projet ;
- déjà présente sur `origin/main`.

Ces alertes ne sont pas causées par Beaver, mais doivent être suivies. Les
dépendances npm de lint peuvent être mises à jour séparément. Pour `glib`, il
faut vérifier la voie de mise à jour compatible avec Tauri/GTK.

### F-09 — Inventaire des racines persistantes incomplet par construction

Le test `scripts/migration/persistence-roots.test.mjs` est utile mais son
expression régulière ne trouve que la forme littérale
`data_dir().join("...")` :

- motif : lignes 15-16 ;
- boucle de contrôle : lignes 39-55.

Il peut manquer les alias de chemin, les fonctions qui reçoivent `data_dir` en
paramètre, les chemins construits dynamiquement et les accès frontend.

La revue manuelle et le profil des 25 domaines compensent en partie cette
limite. Le nom du test ne doit toutefois pas laisser croire qu'il prouve à lui
seul l'exhaustivité.

### F-10 — Plan et description de CL devenus partiellement obsolètes

Les tâches 6 à 14 du plan sont encore entièrement décochées alors que leurs
commits existent. La tâche 5 et les validations natives sont partiellement
cochées. La version finale `1.1.0`, le changelog Beaver et ses notes dans les
sept langues ne sont pas encore ajoutés.

L'état actuel reste volontairement en `1.0.2` :

- `package.json:3` ;
- `src-tauri/Cargo.toml:4` ;
- `src-tauri/tauri.conf.json:4`.

Le dépôt Beaver existe mais est encore vide. La release-pont CL-GO `v1.0.2` est
un brouillon complet, mais le tag Git public n'existe pas encore. Ces éléments
sont normaux à ce stade, mais rendent toute publication actuelle incorrecte.

Avant la suite :

- cocher uniquement ce qui est réellement terminé ;
- corriger les nombres « 94/97 contrats » en distinguant domaines, fichiers et
  extraits de preuve ;
- mettre la description de la CL au même niveau que le code et ce rapport.

### F-11 — Limite de taille des fichiers

La règle projet exige moins de 200 lignes pour les fichiers code et test.
Trois fichiers modifiés dépassent ou atteignent cette limite :

- `src-tauri/src/services/git/branch_commit_tests.rs` : 208 lignes ;
- `scripts/brand/brand-boundaries.test.mjs` : 202 lignes ;
- `src-tauri/src/services/agent_local/ollama_registry.rs` : 200 lignes.

Le découpage est un nettoyage structurel, pas un blocage fonctionnel.

### F-12 — Quatre dépendances de test inutilisées

`cargo +nightly udeps --all-targets` a signalé quatre dépendances. Des
vérifications indépendantes ont confirmé qu'il s'agit de vrais positifs :

- `proptest` à `src-tauri/Cargo.toml:183` ;
- `rstest` à `src-tauri/Cargo.toml:185` ;
- `pretty_assertions` à `src-tauri/Cargo.toml:187` ;
- `tokio-test` à `src-tauri/Cargo.toml:189`.

Elles ne sont utilisées par aucun import, macro, test, doctest ou cible
conditionnelle. Elles existaient déjà avant Beaver. Leur suppression est un
nettoyage à très faible risque, suivi de la régénération de `Cargo.lock`.

## Couverture de validation

| Validation | Résultat |
|---|---|
| `npm ci` | Réussi |
| `npm test` | 338 fichiers, 1 554 tests réussis |
| `npm run build` | Réussi |
| `npm run lint` | Réussi |
| `npm run test:install` | Réussi |
| `npm run test:brand-boundaries` | 12 tests réussis, contenu et contexte exacts verrouillés |
| `npm run test:persistence-migration` | 9 tests réussis |
| `npm run test:release-workflow` | 14 tests réussis |
| `npm run test:update-manifest` | 5 tests réussis |
| `npm run test:bridge-release` | 10 tests réussis |
| `npm run test:coverage` | 74,36 % des lignes frontend |
| `cargo fmt --check` | Réussi |
| `cargo check` | Réussi |
| `cargo clippy --all-targets -- -D warnings` | Réussi |
| `cargo test` | 2 306 tests réussis |
| `cargo +nightly udeps --all-targets` | Aucune dépendance Rust inutilisée |
| Semgrep `p/default` | 0 constat, 209 règles appliquées aux fichiers suivis |
| `npm audit --omit=dev` | 0 vulnérabilité de production |
| Build Tauri macOS après corrections | Réussi avec autorisation locale ad hoc explicite |
| Validation Beaver.app + DMG | Identités, helpers, signature ad hoc et somme du DMG validés |
| CI GitHub de la CL | À reconfirmer sur le commit correctif poussé |

Le contrôle fournisseur vivant `npm run test:provider-usage` n'a pas été
exécuté, car il exige une vraie clé API. Il n'est pas compté comme un échec de
la migration.

Une dernière revue contradictoire du lot correctif a détecté une fenêtre de
temps entre le premier contrôle d'installation existante et la copie finale.
Le contrôle a été déplacé juste avant cette copie et l'identité du dossier
déplacé est vérifiée avant tout nettoyage. Aucun nouveau constat critique ou
élevé n'est resté ouvert après cette correction.

## Blast radius

Les zones les plus exposées par cette migration sont :

1. le chemin de mise à jour, appelé depuis un seul hook frontend et deux
   commandes Tauri ;
2. le stockage, partagé par tous les domaines de l'application ;
3. les identités OAuth, keyring, WebView et navigateur ;
4. le démarrage automatique ;
5. les formats de paquets et raccourcis propres aux trois OS ;
6. les messages et auteurs Git générés ;
7. la chaîne GitHub release, manifeste et nouveau dépôt.

Le changement visible est large, mais les identités internes les plus
destructrices ont volontairement un rayon de changement nul.

## Contexte historique

La branche suit une séquence saine :

- gel des contrats ;
- updater de la version-pont ;
- manifeste et helper ;
- préparation du brouillon CL-GO `1.0.2` ;
- marque visible Beaver ;
- autostart et packaging ;
- CI et profil de compatibilité.

Le nouveau helper remplace les anciens scripts de mise à jour générés à la
volée. C'est une amélioration nette de sécurité. Les problèmes de dépendances
F-08 et F-12 existaient déjà sur la branche principale et ne sont pas une
régression de la migration.

## Ordre de correction recommandé

1. Garde toutes les releases en brouillon.
2. Corrige F-01 et F-04 ensemble, avec des tests de non-régression.
3. Décide le contrat de rollback Linux/Windows et aligne le code, le changelog
   et les sept notes de release.
4. Durcis ou limite explicitement les installateurs manuels.
5. Nettoie les dépendances inutilisées et découpe les trois fichiers trop longs.
6. Mets à jour le plan et la description de la CL.
7. Exécute la matrice native CL-GO vers Beaver sur des copies de profils réels.
8. Publie la version-pont seulement après succès.
9. Pousse l'historique validé vers le dépôt Beaver, ajoute `1.1.0`, ses notes et
   son changelog.
10. Crée d'abord une release Beaver en brouillon, retélécharge et vérifie ses
    quatre assets, puis teste la mise à jour réelle avant publication.

## Méthode et limites

L'audit a utilisé le graphe Graphify, le diff Git complet, l'historique des
commits, une lecture ligne par ligne des zones de mise à jour et d'installation,
les tests automatisés, la CI GitHub, Semgrep, Dependabot, npm audit,
`cargo-udeps` et un build natif macOS.

La revue a couvert entièrement les zones à haut risque et a effectué un scan de
surface sur le reste des 188 fichiers. Elle ne remplace pas :

- un test réel sur Linux et Windows ;
- un test avec de vrais jetons OAuth/API ;
- une vérification de permissions macOS après remplacement de l'application ;
- un audit externe du compte GitHub ;
- une preuve cryptographique de provenance sans signature de publication.

L'audit initial n'a pas modifié le code. Le tableau « État après le lot de
corrections » documente les changements appliqués ensuite.
