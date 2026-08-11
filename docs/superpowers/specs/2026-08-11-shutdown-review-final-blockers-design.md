# Shutdown Review Final Blockers Design

## Périmètre

Ce correctif ferme les deux blocages de fusion confirmés après N2 et l'interaction N3×N4, remet le contrat de marque de la release en accord avec le dépôt, et stabilise le test d'horloge Windows. Aucun autre constat N5–N15 n'entre dans ce train.

## Diagnostic différentiel

### Redémarrage

1. **Hypothèse confirmée — code réservé Tauri utilisé trop tôt.** Le bouton appelle `app.exit(tauri::RESTART_EXIT_CODE)`. Tauri ignore `prevent_exit()` pour ce code, donc sa boucle se termine pendant le nettoyage asynchrone.
2. **Hypothèse réfutée — le bouton ne rejoint pas le coordinateur.** `restart_application` appelle bien `app_exit::request_restart`, puis `RunEvent::ExitRequested` rejoint `handle_requested`.
3. **Hypothèse réfutée — deux actions finales s'annulent.** La transition atomique `Closing → ReadyToExit` donne déjà un gagnant unique et les tests N2 le prouvent.

### Course expiration/réservation Windows

1. **Hypothèse confirmée — deux autorités sont libérées dans le mauvais ordre.** `CefReservation::expire()` libère la table centrale avant la destruction de `WindowsEmergencyRegistration`. Une nouvelle génération peut donc réserver la case mais échouer à l'inscrire dans la table d'urgence encore occupée.
2. **Hypothèse réfutée — la génération de `clear` efface la nouvelle entrée.** `clear(slot, generation)` compare déjà la génération ; le défaut précède l'installation de la nouvelle entrée.
3. **Hypothèse réfutée — la table `pending` empêche la réutilisation.** `take_if_expired` retire le pointeur `pending` avant la libération centrale ; elle ne sérialise pas la table d'urgence.

### Contrat de marque

1. **Hypothèse confirmée — compteurs figés périmés.** Le test observe `cl-go-dash = 229` et `cl_go_dash = 46`, contre `211` et `32` attendus.
2. **Hypothèse réfutée — nouvelles références inconnues.** Le balayage produit `INCONNU ET BLOQUANT (0)`.
3. **Hypothèse réfutée — pollution par des fichiers locaux non suivis.** Le dépôt de travail est propre et les écarts viennent des fichiers suivis de la branche.

## Décisions

### 1. Sentinelle de redémarrage Beaver

Beaver définit un code interne distinct de `tauri::RESTART_EXIT_CODE`. Le bouton demande d'abord `app.exit(BEAVER_RESTART_REQUEST_CODE)`. Comme ce code n'est pas réservé par Tauri, `ExitRequestApi::prevent_exit()` retient réellement la boucle pendant le nettoyage.

`requested_intent` reconnaît uniquement cette sentinelle comme intention initiale `Restart`. Après le nettoyage ou au watchdog, l'action finale unique appelle alors `app.request_restart()`. Le code réservé Tauri n'apparaît donc qu'au dernier maillon, lorsque l'état est déjà `ReadyToExit` et que plus aucun nettoyage ne doit être retenu.

Alternatives rejetées : appeler directement le coordinateur depuis la commande contournerait l'autorité `RunEvent::ExitRequested`; appeler immédiatement `AppHandle::request_restart()` reproduirait la panne ; retarder arbitrairement la commande créerait un second budget.

### 2. Libération Windows dans l'ordre inverse de l'acquisition

Pour une publication expirée, le tracker retire déjà l'entrée `pending`. Il détruit ensuite explicitement l'inscription d'urgence et les objets IPC, puis seulement libère la réservation centrale. Ainsi, une nouvelle génération ne peut observer la case centrale libre tant que l'ancienne surface d'urgence existe encore.

La génération reste vérifiée dans `clear`; elle protège les autres chemins de destruction. Aucun verrou global commun aux deux tables n'est ajouté : l'ordre de destruction suffit et conserve les autorités bornées existantes.

Alternatives rejetées : réessayer `emergency.install` masquerait une incohérence réelle ; fusionner les deux tables élargirait fortement N3/N4 ; transformer l'échec en non-fatal laisserait un helper sans filet d'urgence.

### 3. Contrat de marque et horloge

Les deux compteurs internes sont actualisés seulement après vérification que le groupe inconnu reste vide et que les autres compteurs n'ont pas changé. Le test de release demeure l'autorité : aucun relâchement de règle ni exclusion supplémentaire.

Le test d'horloge conserve une échéance courte mais remplace l'assertion immédiate à 20 ms par une marge qui ne peut pas expirer sous un simple retard d'ordonnancement. Il continue de prouver qu'une échéance monotone est d'abord future puis atteinte avant une limite absolue.

## Tests obligatoires

- Un test traverse le point d'entrée réel du bouton jusqu'à `requested_intent` avec une action de sortie injectée : il exige la sentinelle Beaver, vérifie qu'elle diffère du code Tauri, puis prouve l'intention `Restart`.
- Le code réservé Tauri reçu à l'action finale ne doit pas recréer une seconde phase de nettoyage lorsque l'état est `ReadyToExit`.
- Un test Windows synchronise expiration et nouvelle réservation. La nouvelle réservation ne peut démarrer qu'après destruction de l'inscription d'urgence et elle ne doit jamais rendre le tracker fatal.
- `npm run test:brand-boundaries` passe avec `INCONNU ET BLOQUANT (0)` implicitement conservé par le test.
- Les tests d'horloge passent de façon répétée sans marge nulle.
- Les suites ciblées sont exécutées rouge puis vert ; la validation finale comprend `npm test`, `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings` et les tests Rust Windows avec `--features windows-tests`.

## Critères d'acceptation

- Cliquer Redémarrer retient la boucle Tauri pendant le nettoyage, puis relance exactement une fois.
- Une expiration Windows ne crée aucune fenêtre où la table centrale est libre et la table d'urgence encore occupée.
- La première release ne s'arrête plus sur le contrat de marque.
- Aucune nouvelle collection, aucun nouveau délai métier et aucun message sensible ne sont introduits.
