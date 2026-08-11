# Corrections finales de la review de fermeture

## Objectif

Tu corriges N1, N2, N12, N3 et N4 sans élargir le jalon. La fermeture conserve une seule chronologie globale, une seule intention de sortie et une supervision CEF bornée à 64 slots sur Windows et macOS.

La branche devient fusionnable lorsque les vérifications de publication Windows ne peuvent plus réussir à vide, que `Redémarrer` redémarre aussi au passage du watchdog, que tous les budgets CEF s'accordent sur 13 secondes et que les réservations ou helpers défaillants ne restent plus orphelins.

## Hors périmètre

- Tu ne corriges pas N5 à N15, dont le détail complet n'a pas été fourni.
- Tu ne modifies pas l'interface ni les traductions.
- Tu ne remplaces pas l'autorité native Windows, le Job Object ou le reaper macOS.
- Tu ne promets pas de redémarrage après la sortie brute ultime à 15 secondes : ce filet reste volontairement indépendant de Tauri.

## N1 — Vérification réelle du paquet Windows

Tu fournis `CARGO_BUILD_TARGET` et `BEAVER_TAURI_BUNDLE_TYPE` directement à l'étape `Inspect and install Windows package`. Les valeurs d'une étape GitHub Actions ne remontent pas vers une étape sœur ; les redéclarer au consommateur rend leur portée explicite.

Tu contrôles immédiatement le code de sortie de `tauri-bundle-marker.mjs verify` et tu fais échouer l'étape s'il est non nul. PowerShell ne transforme pas automatiquement l'échec d'un programme natif en échec de l'étape, et une commande réussie ensuite masque sinon la vérification ratée.

Le test du workflow vérifie que les deux variables et la propagation d'erreur appartiennent à la même étape Windows que la commande de vérification.

## N2 — Intention de redémarrage immuable jusqu'au watchdog

`ExitIntent` reste l'unique autorité qui distingue une sortie d'un redémarrage. Tu le transmets au watchdog avec le code de sortie dès le début de la fermeture, au lieu de réduire le watchdog à un simple entier.

Le chemin normal et le watchdog des 10 secondes appellent la même action finale :

- `Exit` demande la sortie Tauri avec le code possédé par la première requête ;
- `Restart` demande le redémarrage Tauri ;
- une requête concurrente ne peut pas remplacer l'intention initiale.

La sortie brute ultime à 15 secondes conserve seulement un code sûr. Elle ne dépend ni de l'intention, ni de la boucle Tauri, car sa raison d'être est de terminer Beaver même si ces mécanismes sont bloqués.

## N12 — Une seule échéance CEF à 13 secondes

`ShutdownTimeline::cef_helper_exit_deadline()` renvoie l'échéance d'urgence globale. Tu ne recalcules plus une échéance locale à partir de la marge de la sortie ultime.

Le parent commence donc la phase forcée à 13 secondes et les moniteurs enfants reçoivent exactement la même échéance absolue. La sortie ultime reste à 15 secondes et fournit deux secondes indépendantes pour constater ou répéter la terminaison forcée.

## N3 — Expiration bornée des publications absentes

Chaque réservation CEF en attente porte une échéance de publication calculée une seule fois lors de la réservation. Cette échéance utilise `CEF_ADMISSION_TIMEOUT`, déjà autorité du budget d'admission, afin de ne pas créer un second délai concurrent.

Les boucles de suivi Windows et macOS traitent `Unpublished` de deux façons :

- avant l'échéance, elles continuent d'attendre ;
- à l'échéance, elles retirent l'entrée en attente et laissent tomber la réservation et ses objets.

L'expiration d'une publication est une panne récupérable du lancement du helper. Elle libère le slot, écrit un avertissement générique et ne ferme pas Beaver. Une publication présente mais invalide, une identité incohérente ou une admission sans confinement restent des erreurs fatales, car un processus a alors franchi une frontière de sécurité avec un état contradictoire.

La même règle s'applique aux 64 slots des deux plateformes. Un test remplit les slots avec des réservations déjà expirées, constate leur nettoyage, puis prouve qu'une nouvelle réservation est de nouveau acceptée sans enregistrer de panne du superviseur.

## N4 — Signal de fermeture Windows réellement consommé

Tu rends `WindowsPublicationObjects::begin_closing` disponible en production. Le parent conserve les objets des helpers admis dans une table d'urgence Windows séparée, bornée aux 64 slots existants et indexée par slot plus génération. Cette table est créée avant le thread normal et reste accessible au chemin de fermeture même si le suivi normal est bloqué.

Au passage à `Closing`, l'autorité Windows :

1. ferme la porte de lancement et invalide les réservations tardives ;
2. convertit l'échéance globale de 13 secondes avec une horloge monotone Windows ;
3. écrit cette échéance et signale l'événement de fermeture aux objets en attente et admis ;
4. conserve le Job Object comme filet forcé indépendant à partir de 13 secondes.

Le helper Windows démarre un moniteur borné avant d'attendre son admission. Ce moniteur lit seulement sa propre page de contrôle et son événement. Quand l'échéance signalée est atteinte, il termine le helper ; une page invalide ou une génération différente échoue fermée. Le moniteur s'arrête et est rejoint si `cef::execute_process` revient normalement.

L'horloge Windows vit dans un fichier dédié et utilise des ticks monotones partagés par les processus. Les ticks ne sont jamais comparés à une horloge murale, afin qu'un changement de date système ne prolonge pas la vie d'un helper.

## Propriété et sens des dépendances

- `AppExitCoordinator` possède l'intention et la chronologie ; le nettoyage et le watchdog les consomment sans les réinventer.
- `CefAuthorityTable` possède les réservations ; les conteneurs `pending` ne font que conserver leurs objets jusqu'à publication ou expiration.
- la table d'urgence Windows possède les objets de signalisation admis ; `WindowsNativeAuthority` reste seule propriétaire des handles de processus et des Job Objects.
- le helper consomme uniquement ses objets privés ; il ne reçoit aucun handle ni état appartenant à un autre slot.

## Gestion des erreurs et traces

- Une publication absente à l'échéance est récupérable : nettoyage local et avertissement borné.
- Une donnée publiée incohérente est fatale : fermeture coordonnée existante.
- Un échec de signal de fermeture est fatal pour la supervision, mais la phase forcée et la sortie ultime restent armées.
- Les traces ne contiennent ni chemin, ni nonce, ni ligne de commande, ni détail de jeton Windows.
- Toute collection ajoutée conserve la capacité fixe de 64 entrées et remplace une génération seulement après avoir libéré la précédente.

## Tests obligatoires

Tu écris chaque test avant son correctif et tu observes son échec attendu.

- workflow de release : variables disponibles dans l'étape consommatrice et échec natif propagé ;
- watchdog : une intention `Restart` atteint l'action de redémarrage à 10 secondes ;
- politique : échéance enfant CEF strictement égale à l'échéance d'urgence ;
- Windows et macOS : 64 publications absentes expirent, les slots redeviennent disponibles et le superviseur reste sain ;
- Windows : le signal de fermeture atteint un helper admis avec l'échéance globale ;
- Windows : le moniteur enfant refuse une génération invalide et réagit à l'échéance ;
- régressions : tests Rust ciblés, tests Node du workflow, `cargo fmt --check`, `cargo check`, Clippy strict et suite Rust complète.

## Critères d'acceptation

- Une release Windows ne peut pas être verte si la vérification du bundle échoue ou manque ses paramètres.
- Le bouton `Redémarrer` redémarre après nettoyage normal et après le watchdog Tauri des 10 secondes.
- Tous les mécanismes CEF écrits avant la sortie ultime utilisent l'échéance absolue de 13 secondes.
- Une suite de lancements qui ne publient jamais ne sature pas définitivement les 64 slots et ne ferme pas Beaver.
- Les helpers Windows en attente et admis reçoivent le signal de fermeture, puis le Job Object reste capable de les forcer.
- Aucun nouveau fichier de code source ne dépasse 230 lignes et chaque nouvelle table reste bornée.
