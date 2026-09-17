# E09 — Diagnostics et limites des favicons

Date : 17 septembre 2026
Commit de départ : `d6d7134b`

## Décision

Conserver le watchdog, la réparation après panique et la saturation actuelle. Ces trois mécanismes répondent à des pannes différentes et leur coût est borné. Aucun ne doit être supprimé pour gagner quelques dizaines de lignes.

## Watchdog

Le téléchargement natif CEF n'expose pas de poignée d'annulation. Le watchdog ne prétend donc pas libérer le permis après 60 secondes : il signale pendant le blocage que le permis reste détenu et que seule la fin du rappel natif ou le redémarrage peut le rendre.

Ce diagnostic est utile précisément parce qu'un log au `Drop` n'existerait qu'après la fin du blocage. Il permet de distinguer une file normale d'un rappel CEF qui ne revient pas. Son coût reste limité à une tâche temporelle par téléchargement admis, soit quatre tâches au maximum, et chaque tâche est annulée avec son permis.

## Panique et verrou empoisonné

La garde FFI empêche une panique Rust de traverser CEF, mais elle ne répare pas l'état partagé et ne publie pas la disparition des icônes déjà affichées. `favicon_runtime` capture donc la panique autour de la mutation, puis `FaviconStore` :

- récupère le verrou empoisonné ;
- remet l'état à zéro ;
- désactive les futurs travaux avec la révision terminale ;
- conserve une invalidation bornée par conversation jusqu'à ce qu'un handle d'application puisse la publier ;
- relance ensuite la panique pour que la garde FFI la traite.

Remplacer ce parcours par un simple `into_inner()` laisserait potentiellement un état partiellement modifié et des favicons périmées dans l'interface. La double garde porte donc deux responsabilités complémentaires.

## Saturation de la révision

La limite `2^53 - 1` correspond à la précision entière sûre de JavaScript. Elle est pratiquement inatteignable, mais le comportement de frontière reste nécessaire : vider les entrées, publier une dernière invalidation puis rendre le système inerte évite un retour à une révision ancienne qui pourrait réaccepter des rappels périmés.

Un `checked_add` qui ignorerait seulement la mise à jour laisserait l'interface afficher l'ancien état. Le comportement actuel échoue fermé et reste plus sûr pour quelques lignes supplémentaires.

## Bornes conservées

- dix favicons au total, alignées sur le budget global de vues ;
- quatre téléchargements concurrents ;
- huit candidats par document ;
- 32 Kio et 64 × 64 pixels par image ;
- invalidations différées bornées à vingt instantanés ;
- rejet des rappels tardifs par époque, document, requête et délai.

## Validation

`cargo test --lib favicon` réussit avec 23 tests, zéro échec et un test de génération explicitement ignoré. La suite couvre le déclenchement et l'annulation du watchdog, la panique et le verrou empoisonné, la saturation, les bornes, les évictions, les rappels tardifs et le contrat généré.

## Condition de réexamen

Le watchdog pourra être retiré si CEF expose une annulation fiable ou si un diagnostic global fournit la même preuve pendant le blocage. La réparation pourra être simplifiée seulement si les mutations deviennent transactionnelles et qu'une invalidation de l'interface est garantie après toute panique.
