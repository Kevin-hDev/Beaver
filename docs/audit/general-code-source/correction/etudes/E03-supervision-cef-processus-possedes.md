# E03 — Frontière entre supervision CEF et processus possédés

Date du relevé : 17 septembre 2026
Commit examiné : `d280e5cf`

## Décision

Ne pas faire adopter les helpers CEF par `OwnedProcess` et ne pas fusionner les suivis macOS et Windows. Leurs propriétaires et leurs garanties diffèrent réellement. Un partage futur est acceptable uniquement pour des lectures natives pures, sans enregistrement ni effet sur le cycle de vie, si un changement concret permet de supprimer plus de code qu'il n'en ajoute.

Aucun prototype n'est justifié dans ce plan. Les copies sont visibles, mais leur extraction immédiate ajouterait une nouvelle interface, des conversions d'erreurs et des paramètres propres à CEF sans retirer les mécanismes spécialisés.

## Contrats comparés

| Sujet | `OwnedProcess` | Supervision CEF | Décision |
|---|---|---|---|
| Création | Beaver lance ou adopte explicitement un processus. | Chromium crée ses helpers avant que Beaver les voie. | Garder deux admissions. |
| Preuve d'identité | PID, date de démarrage, exécutable et groupe ou Job possédé. | Marqueur publié, génération, rôle, parent, date, exécutable autorisé et délai. | Ne partager que la lecture brute éventuelle. |
| macOS | Le processus admis doit diriger son propre groupe. | Le helper publie aussi son parent et son groupe ; CEF contrôle leur cohérence avant le signal. | Les règles d'admission restent séparées. |
| Windows | Job global pour les processus de Beaver ; `DedicatedJob` pour certains propriétaires dédiés. | Job dédié détenu par le suivi CEF et rempli après publication du helper. | Ne pas utiliser le Job global. Une primitive de Job dédiée ne sera extraite que si un troisième appelant réel apparaît. |
| Capacité | Registre global des processus possédés. | 64 emplacements, réservations expirables, objets IPC et emplacements d'urgence. | Garder les limites et tables CEF. |
| Arrêt | Signal exact ou terminaison du Job possédé. | Barrière d'arrêt liée au moteur, revalidation, filet de secours et attente avant déchargement de CEF. | Garder l'orchestration CEF. |
| Erreurs | `OwnedProcessError`, destiné aux services internes. | `CefUnavailableCategory`, publié dans l'état du navigateur. | Pas de conversion commune tant qu'aucun appel partagé ne l'exige. |

## Ce qui peut être partagé

Une primitive commune ne devra faire qu'observer un processus : vérifier la plage du PID, lire sa date de démarrage, son parent, son groupe et son exécutable, puis rendre ces faits sans enregistrer le processus et sans décider de son admission. CEF appliquera ensuite son marqueur, sa génération, son rôle, son parent, son chemin autorisé et son délai. `OwnedProcess` appliquera son appartenance au groupe ou au Job qu'il possède.

Cette frontière évite le piège actuel de `OwnedProcess::identity` : sous Windows, cette fonction exige déjà l'appartenance au Job de Beaver ; sous macOS, le signal exact exige que le groupe appartienne au PID. Ces conditions ne décrivent pas à elles seules le protocole CEF.

Le Job Windows CEF et `DedicatedJob` exécutent des appels système proches. Ils ne sont pourtant pas interchangeables aujourd'hui : CEF conserve la poignée native dans son emplacement d'urgence et vérifie l'admission avant de publier le helper comme suivi. Extraire un type générique maintenant déplacerait surtout le code et les erreurs.

## Ce qui reste propre à CEF

- le ticket aléatoire, le marqueur, la génération et le rôle du helper ;
- la validation du parent et du chemin dans la liste des exécutables CEF ;
- l'échéance de publication et le refus fermé d'un helper en retard ;
- les 64 emplacements, les objets IPC, les réservations en attente et les emplacements d'urgence ;
- la séquence de fermeture liée au déchargement du moteur et son filet de terminaison ;
- les catégories d'indisponibilité visibles par le navigateur.

La réservation macOS et Windows semble très proche au premier regard, mais Windows installe un emplacement d'urgence avant l'admission alors que macOS porte un modèle d'échec et une racine d'objets différents. Une fonction commune devrait recevoir ces différences sous forme de paramètres ou de traits. Elle serait plus difficile à relire que les deux petites fonctions actuelles et ne supprimerait pas leur état natif.

## Admission de secours et limites

La voie de secours CEF reste bornée par le même nombre d'emplacements que la voie normale. Elle ne doit jamais enregistrer un PID sur sa seule existence : l'identité, la génération et les objets publiés restent obligatoires. Un échec de lecture ou d'assignation ferme l'admission et laisse le filet natif terminer le helper.

Le remplacement du protocole Windows par un Job hérité du processus Beaver est écarté. Il étendrait le Job à tous les enfants, changerait la relation avec les processus autorisés à survivre et perdrait l'échéance propre à chaque helper. Aucun bénéfice mesuré ne justifie cette modification de sécurité.

## Preuves à exiger si un partage devient utile

- macOS : PID réutilisé, parent ou date modifié, groupe inattendu, zombie, exécutable différent, arrêt normal et forcé ;
- Windows : PID réutilisé, parent ou date modifié, chemin différent sans dépendre de la casse, échec d'assignation, appartenance au Job et fermeture de sa dernière poignée ;
- deux OS : limite de 64, expiration, réservation abandonnée, publication tardive, fermeture pendant l'admission et absence de helper survivant après arrêt du moteur ;
- CI Windows avec et sans moteur lié, car la machine macOS locale ne prouve pas les appels natifs Windows.

## Conséquences pour le plan

- E03 ne crée aucune correction autonome et ne bloque pas M27, M28 ou M29.
- M27 et M28 doivent conserver le suivi de cycle de vie CEF ; ils ne concernent que l'état des vues et sa publication.
- M29 peut uniformiser les conditions de compilation sans toucher aux propriétaires natifs.
- M32 peut ranger les fichiers CEF par domaine, mais ne doit pas déplacer leur autorité dans `owned_process`.
- Un futur correctif qui touche simultanément les deux lecteurs d'identité devra d'abord tenter une petite fonction d'observation commune. Il sera abandonné si le nombre de conversions, paramètres et branches dépasse le code natif supprimé.

## Validation de l'étude

Les chemins macOS et Windows ont été relus depuis la réservation jusqu'à l'identité, l'assignation et l'arrêt. La comparaison a inclus `owned_process_unix.rs`, `owned_process_macos.rs`, `owned_process_windows.rs`, `owned_process_windows_support.rs` et les fichiers `cef_supervision/{macos,windows}` correspondants. Le constat C1 est confirmé pour les appels natifs dupliqués, mais sa recommandation de Job partagé est réduite. Le constat C7 ne justifie pas une abstraction : les différences d'état et de secours sont des garanties, pas seulement des noms de plateforme.
