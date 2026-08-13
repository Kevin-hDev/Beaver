# Conception — corrections finales du lot mineur du jalon 2

Date : 2026-08-13

## But et périmètre

Cette série corrige les sept écarts qui bloquent la fusion de `codex/shutdown-milestone-2-minors` dans la review `2026-08-13-shutdown-milestone-2-minors-review.md`. Elle ne fusionne pas la branche et ne transforme pas les nouveaux mineurs explicitement prévus après cette fusion en refactorisations opportunistes.

Les sept résultats attendus sont : une décision correcte pour les réveils longs, une rotation de journal qui retrouve une marge, un échec Discord transitoire qui reste reconnectable, des trames WebSocket de contrôle qui ne cassent pas une connexion saine, tout refus gateway compté, un callback OAuth MCP non bloquable par une connexion muette, et aucune couture d'injection Forecast dans le binaire publié.

## Scheduler : occurrence en cours et journal glissant

Le runtime possède un registre borné `InFlightWakeups`, indexé par l'identifiant du réveil et son instant planifié. La réservation précède le lancement du travail et une garde retire automatiquement l'occurrence à la fin, y compris après annulation ou panique. Sa capacité dérive de l'unique capacité des travaux scheduler ; il ne crée donc pas une seconde limite indépendante.

La réconciliation sépare les occurrences décidables de celles encore en cours. Elle journalise les premières mais n'avance pas le curseur tant qu'une occurrence candidate est en cours. La raison est la durabilité : si Beaver tombe pendant un réveil, le prochain démarrage doit encore pouvoir le classer `missed`; si le travail finit normalement, son résultat terminal est écrit avant le retrait de la garde, puis la prochaine réconciliation peut avancer sans produire de faux résultat.

La rotation du journal conserve 250 lignes, nouvelle entrée comprise, au lieu de revenir immédiatement au plafond de 500. Le plafond d'affichage reste 500 ; seule la marge après rotation change. Ainsi les 250 ajouts suivants restent incrémentaux.

## Gateway : issues Discord, trames et refus

Le traitement des événements Discord retourne une issue fermée : continuer la connexion, reconnecter, ou arrêter parce que le consommateur est fermé. L'échec d'envoi d'`IDENTIFY` demande une reconnexion locale et ne termine plus la tâche supervisée. L'échec de zéroïsation est tracé sans donnée sensible et ne transforme pas un envoi réussi en panne réseau ; la copie applicative reste libérée au plus tôt, sous la limite imposée par tungstenite.

Discord et Slack consomment un classificateur commun des trames entrantes. `Text` est traité, `Ping`, `Pong`, `Binary` et les trames internes sont ignorés, tandis que `Close`, fin de flux et erreur déclenchent une reconnexion. La politique de backoff existante reste l'unique autorité des délais.

Le refus du travail consommateur passe par `record_refusal`, qui incrémente d'abord le compteur en mémoire puis tente l'audit persistant. L'audit peut rester indisponible sans effacer la preuve locale de la perte.

## OAuth MCP : connexions concurrentes bornées

Le serveur MCP reprend le mécanisme déjà éprouvé du callback Codex : un `JoinSet` possède les lectures acceptées, chaque connexion expire après 5 secondes, et le nombre total accepté reste borné à 50. Une connexion muette ne bloque donc plus l'acceptation du vrai callback. La limite globale de 300 secondes, l'annulation, la comparaison constante du `state` et la taille maximale de requête restent inchangées.

Le premier callback au `state` valide gagne. Un mauvais callback ou une lecture expirée libère sa tâche ; un refus ne consomme jamais une collection non bornée. Quitter l'attente détruit le `JoinSet` et annule ses lecteurs restants.

## Forecast : production sans injection de test

La désinstallation devient une petite transaction composée d'étapes de production explicites : supprimer le staging, supprimer le modèle, puis supprimer le runtime seulement s'il n'est plus partagé. Ces étapes ne connaissent aucune injection.

`UninstallBoundary` et le scénario qui injecte une panne entre les étapes sont entièrement placés sous `#[cfg(test)]`. Le test appelle les mêmes étapes réelles, mais l'orchestration d'injection n'existe pas dans la compilation publiée. La note d'audit de `bb2ec62` sera corrigée après création du commit afin de nommer le vrai correctif de l'obligation 28.

## Tests et preuves

Chaque comportement modifié suit rouge, vert, puis vérification du domaine :

- un test garde une occurrence en cours hors des décisions `missed`, puis la rend décidable après libération ;
- un test prouve qu'après la rotation, le 502e ajout ne relit pas le journal ;
- les tests Discord distinguent reconnexion, arrêt du consommateur et échec de zéroïsation ;
- un test commun exerce `Ping`, `Pong`, `Binary`, `Close`, erreur et texte ;
- un test prouve que le refus du consommateur incrémente le compteur sans dépendre de l'écrivain ;
- un vrai socket muet reste ouvert pendant qu'un vrai callback MCP valide aboutit ;
- les tests d'invariant Forecast utilisent les étapes réelles et `cargo check` vérifie la compilation de production.

La vérification finale reprend `cargo test --lib --features windows-tests -- --test-threads=1`, `cargo check`, Clippy strict, formatage, tests frontend, TypeScript, lint, limites de 230 lignes, puis `graphify update .`. Chaque commit cohérent reçoit une git note avec la raison et les sorties réellement lues. La branche est poussée et la CI surveillée, sans fusion.
