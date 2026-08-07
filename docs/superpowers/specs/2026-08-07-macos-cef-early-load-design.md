# Chargement précoce de CEF sur macOS

## Objectif

Éliminer les plantages intermittents `SIGTRAP` provoqués lorsque Chromium
modifie les zones mémoire de macOS pendant que les tâches Tauri sont déjà
actives, sans modifier le démarrage de Windows ou de Linux.

## Comportement attendu

- Sur macOS, la bibliothèque CEF est chargée avant AppKit, Tauri, la capture de
  l'environnement shell et la création de tâches secondaires.
- La bibliothèque reste chargée jusqu'après l'arrêt propre de CEF.
- Si la bibliothèque est absente, invalide ou impossible à charger, Beaver
  continue de démarrer mais le navigateur intégré est déclaré indisponible.
- Sur Windows, le bootstrap CEF existant reste inchangé.
- Sur Linux, CEF reste entièrement exclu de la compilation et le navigateur
  intégré reste masqué.

## Solutions considérées

### 1. Chargement précoce limité à macOS — retenu

Le point d'entrée macOS charge CEF avant l'initialisation native et transmet
un garde opaque au cycle de vie de l'application. Le moteur CEF ne recharge
plus la bibliothèque lors de l'événement `Ready`.

Cette solution suit l'ordre recommandé par CEF, conserve une propriété claire
de la bibliothèque et limite le changement à macOS.

### 2. Suspendre les tâches pendant le chargement tardif — rejeté

Suspendre toutes les tâches Tauri serait fragile et ne couvrirait pas les fils
créés par AppKit, WebKit ou des bibliothèques système.

### 3. Lier CEF directement au lancement — rejeté

Cette solution modifierait le packaging et la signature macOS, alors que le
chargement dynamique est le modèle prévu par CEF pour son bac à sable.

## Architecture retenue

Un type public opaque et spécifique à macOS représente la bibliothèque CEF
chargée. Il conserve aussi les chemins du runtime résolus et validés au moment
du chargement afin que le moteur utilise exactement les mêmes fichiers.

Le point d'entrée macOS suit cet ordre :

1. traiter l'éventuel helper qui remplace le processus ;
2. configurer la politique réseau Git avant toute création de fil ;
3. charger CEF ;
4. préparer l'application native uniquement si CEF est disponible ;
5. capturer l'environnement du shell ;
6. lancer Tauri en conservant le garde CEF ;
7. arrêter CEF après la boucle native ;
8. décharger explicitement la bibliothèque avant la sortie du processus.

La capture du shell reste bornée même si un descendant échappé conserve le
tube de sortie ouvert. Son éventuel fil lecteur tardif ne peut pas chevaucher
le chargement de CEF, puisque celui-ci est déjà terminé à ce stade.

Le point d'entrée Linux garde son flux actuel. Le point d'entrée Windows et
`windows_entry.rs` ne sont pas modifiés.

## Gestion des erreurs

Tout échec de résolution ou de chargement bloque l'activation du navigateur,
mais pas le reste de Beaver. L'état du navigateur passe à `Unavailable` au lieu
de rester indéfiniment en préparation. Les messages visibles restent
génériques et aucun chemin interne n'est exposé.

## Tests

- Un test comportemental vérifie que le chargement macOS précède la préparation
  d'AppKit puis la capture de l'environnement shell.
- Un test vérifie que le moteur CEF ne charge plus lui-même la bibliothèque.
- Un test vérifie que la bibliothèque est déchargée seulement après l'arrêt de
  CEF et avant la sortie du processus.
- Des tests protègent le câblage réel de la fermeture et interdisent l'arrêt de
  CEF depuis le nettoyage asynchrone.
- Un test protège directement le câblage macOS de `main.rs` afin que la capture
  du shell reste incluse dans le coordinateur qui charge CEF en premier.
- Un test vérifie que l'initialisation du navigateur attend l'événement `Ready`.
- Les tests de politique de compilation vérifient que le nouveau bootstrap est
  absent de Linux et ne modifie pas le bootstrap Windows.
- La suite Rust complète, `cargo check` et Clippy strict doivent réussir.
- Le lancement macOS est répété plusieurs fois afin de couvrir le caractère
  intermittent du plantage. La validation Windows et Linux est assurée par les
  contrôles de compilation disponibles et la CI multi-plateforme.

## Hors périmètre

Les anciens plantages `SIGABRT` observés pendant la fermeture de CEF ont une
signature différente et ne sont pas traités par ce correctif.
