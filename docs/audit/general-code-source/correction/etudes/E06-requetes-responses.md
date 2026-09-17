# E06 — Socle commun des requêtes Responses

Date du relevé : 17 septembre 2026
Commit examiné : `7dd8f23f`

## Décision

Conserver trois constructeurs de corps : Codex, OpenAI Responses et xAI OAuth Responses. Le code critique qu'ils partagent passe déjà par les mêmes autorités. Le reste expose directement les différences de route et ne justifie pas un constructeur universel à options.

Aucune réduction nette n'est démontrée. Un socle supplémentaire devrait soit abandonner le type `CodexRequest` utilisé par HTTP et WebSocket, soit recevoir des paramètres pour les aperçus, la limite de sortie, le niveau de service et le format du raisonnement. Il déplacerait les branches sans les supprimer.

## Parties déjà communes

| Élément | Autorité actuelle | Statut |
|---|---|---|
| Conversion des messages, appels et résultats d'outils | `codex_client::convert::convert_messages_with_tools_and_continuity_evidence` | Commune aux trois routes. |
| Rejeu et preuve de continuité du raisonnement | même conversion et `reasoning_wire::responses` | Commune, refus fermé avant l'envoi. |
| Conversion des définitions d'outils | `convert_tools_to_responses_api` | Commune, politique de route en paramètre. |
| Placement des résultats d'outils | `route_profile::payload_policy` | Commun, choisi par fournisseur et modèle. |
| Politique des schémas d'outils | `route_profile::tool_policy` | Commune. |
| Clé de cache | `prompt_cache_policy::routing_key` | Commune. |
| Image Responses | `llm::vision::responses_image_part` | Commune lorsque la route l'autorise. |
| Comptage du contexte | `prepared_context_count::responses` | Commun aux parcours qui le publient. |

Ces fonctions portent les conversions susceptibles de diverger ou de perdre des données. Les centraliser produit déjà le bénéfice recherché par le constat 01-C9.

## Différences à garder visibles

| Route | Différences de contrat |
|---|---|
| Codex | Corps typé sérialisable, partagé entre HTTP et WebSocket ; raisonnement toujours présent avec `summary: "auto"` ; niveau de service Codex ; pas de champ de limite de sortie. |
| OpenAI Responses | Corps JSON ; aperçus de résultats ajoutés uniquement si la politique autorise les médias en ligne ; champ de limite choisi par la politique ; effort `none` sans résumé, autres efforts avec résumé ; niveau de service API. |
| xAI OAuth Responses | Modèle validé par le catalogue OAuth ; texte uniquement ; aucun aperçu ; effort issu du catalogue sans résumé ; limite de sortie uniquement pour les fixtures de développement ; aucune route xAI publique exposée. |

Les dix champs identiques restent courts et proches de ces branches. Leur présence locale permet de relire le payload complet d'une route sans suivre une fonction de base puis une série de mutations.

## Options évaluées

### Corps JSON commun

Une fonction renvoyant un `serde_json::Value` forcerait Codex à perdre son type ou à désérialiser un JSON construit dans le même processus. Elle rendrait aussi les accès directs de la voie WebSocket moins sûrs. Rejetée.

### Structure commune avec champs optionnels

Cette structure contiendrait les unions des trois contrats : résumé facultatif, niveau de service facultatif, limite au nom dynamique et aperçus facultatifs. Les constructeurs continueraient à préparer ces valeurs, puis la structure les rebrancherait. Le nombre de décisions ne baisserait pas. Rejetée.

### Fonction de base puis mutations par route

Elle réduirait quelques lignes de JSON, mais séparerait la définition finale entre deux fichiers. Les champs critiques `store`, `parallel_tool_calls`, `include` et la clé de cache seraient moins visibles dans les tests propres à chaque route. Rejetée tant qu'aucune divergence réelle n'est constatée.

## Contrat de maintien

- toute modification d'un champ commun doit rechercher les trois constructeurs ;
- les conversions de messages et d'outils restent interdites hors des autorités communes ;
- chaque route garde un test du payload complet, pas seulement de la fonction commune ;
- Codex garde les mêmes valeurs sur HTTP et WebSocket ;
- xAI OAuth reste texte uniquement jusqu'à preuve de son propre contrat média ;
- l'échec de continuité du raisonnement bloque la requête sur les trois routes ;
- aucun contenu utilisateur n'est modifié ou masqué par ce socle.

## Seuil pour réexaminer la décision

Une extraction sera justifiée si un quatrième client Responses apparaît avec le même contrat, ou si un défaut réel montre qu'un champ commun a divergé malgré les tests. La solution devra supprimer plus de branches qu'elle n'ajoute et garder un type final propre à chaque transport qui en a besoin.

## Conséquences pour le plan

- E06 ne crée aucune correction autonome.
- Les constructeurs restent courts et séparés ; les politiques et conversions communes restent leurs seules dépendances partagées.
- M32 peut ranger les fichiers sans fusionner leurs contrats.
- Une évolution de fournisseur doit modifier son payload et les tests complets associés, puis vérifier explicitement les deux autres routes.

## Validation de l'étude

Les trois constructeurs actuels, leurs types, leurs politiques et leurs consommateurs HTTP/WebSocket ont été relus. Le constat 01-C9 surestime la partie réellement dupliquée : les transformations critiques sont déjà partagées. La forme commune du JSON est confirmée, mais l'extraction proposée n'offre pas les 60 à 100 lignes annoncées une fois les différences et les types conservés.
