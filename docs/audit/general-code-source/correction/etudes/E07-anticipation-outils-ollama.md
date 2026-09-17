# E07 — Intérêt des outils anticipés Ollama

Date de la mesure : 17 septembre 2026
Commit de départ : `50e424f2`
Machine : Apple M4 Pro, Ollama 0.32.15, `qwen3.5:4b`, température 0

## Décision

Conserver l'anticipation des outils en lecture seule pour Ollama. Son gain est négligeable sur un appel local isolé, mais le protocole peut publier plusieurs appels pendant plus d'une seconde avant la fin du flux. Cette fenêtre peut masquer une part perceptible d'une recherche réseau ou d'une lecture lente. L'absence de mesure d'un vrai fournisseur de recherche empêche de chiffrer le gain complet ; cette incertitude conduit à conserver l'optimisation plutôt qu'à retirer un comportement utile.

Le contrôle a révélé un défaut indépendant du gain : une reprise après crash du parseur pouvait recommencer un flux alors qu'un appel d'outil complet avait déjà été publié au collecteur anticipé. La reprise est désormais interdite dès que `result.tool_calls` n'est plus vide. Un appel déjà parti ne peut donc plus être exécuté une seconde fois par cette voie.

## Méthode

Une instance Ollama isolée a été lancée sur `127.0.0.1:11501`, avec le bundle et les modèles locaux existants. Elle a été arrêtée après les mesures et le port a été vérifié libre.

Pour chaque scénario, quatre flux ont été exécutés après chargement du modèle. Le script a horodaté chaque bloc `message.tool_calls` et le bloc final `done`. La fenêtre d'anticipation est la durée entre le premier appel complet et `done`. Elle représente le gain maximal que l'exécution anticipée peut cacher ; sans anticipation, l'outil ne commence qu'après `done`.

Les opérations locales ont été mesurées séparément sur le dépôt, 30 fois : lecture de `package.json`, lecture de trois fichiers et recherche `rg` de `OwnedProcess`. Les temps de génération à froid apparaissent dans les relevés bruts mais la médiane limite leur influence.

## Résultats

| Scénario | Médiane du flux | Fenêtre médiane | Coût local médian | Effet observé ou borné |
|---|---:|---:|---:|---|
| un `read_file` | 1 263,7 ms | 18,6 ms | 0,017 ms | gain négligeable ; la lecture est déjà finie avant d'être perceptible |
| trois `read_file` | 2 889,0 ms | 1 125,0 ms | 0,040 ms pour les trois | appels publiés progressivement, mais fichiers locaux trop rapides pour profiter de la fenêtre |
| un `grep` | 1 800,0 ms | 18,2 ms | 37,8 ms | environ 18 ms peuvent être masquées ; le reste s'exécute après le flux |
| trois `web_search` | 3 179,9 ms | 1 250,9 ms | non exécuté | la première recherche peut commencer environ 1,25 s avant la fin ; gain réel borné par sa propre latence |

Sur un relevé détaillé à trois lectures, les appels sont arrivés à 2 119,6 ms, 2 708,3 ms et 3 223,0 ms, puis `done` à 3 241,2 ms. Sur les trois recherches, les appels étaient espacés d'environ 625 ms. Ollama ne publie donc pas toujours les appels seulement dans son dernier bloc.

La recherche externe n'a pas été exécutée : elle aurait utilisé un fournisseur et des identifiants réels alors que la mesure porte sur l'ordonnancement local. Le bénéfice exact pour `web_search` reste donc une borne, pas une mesure réseau.

## Comparaison avec et sans anticipation

Sans anticipation, le délai après `done` est le temps du lot parallèle le plus lent. Avec anticipation, chaque outil éligible commence dès son bloc et le délai résiduel vaut au plus sa durée moins la fenêtre déjà écoulée. Le lot normal reste parallèle avec la même limite de dix ; l'anticipation ne remplace pas ce parallélisme.

Pour les lectures locales mesurées, la différence est imperceptible. Pour un outil plus lent que la fenêtre disponible, la différence peut atteindre 18 ms sur un appel isolé et environ 1,25 s sur les scénarios à trois appels. Ce second cas est suffisant pour conserver le mécanisme, car `web_search` fait explicitement partie des outils anticipables.

## Permissions et interception

- seuls les outils reconnus comme lecture parallèle sont anticipés ;
- une extension exigeant une confirmation n'est pas anticipée ;
- les pré-hooks et `before_tool_effect` s'exécutent avant l'effet ;
- en présence d'un intercepteur, `agent_loop` désactive l'anticipation avant le flux ;
- si un résultat anticipé existe malgré tout, `agent_loop_tool_batch` le remplace par l'erreur `extensions_eager_interception_conflict` et ne le rejoue pas ;
- le mode fixture draine les appels sans les exécuter.

Avec intercepteur actif, le gain mesuré est donc nul par construction : l'exécution commence dans le lot normal après inspection. Cette différence est une garantie de contrôle et doit rester visible.

## Annulation, reprise et exécution unique

`EagerHandleGuard` annule le collecteur sur chaque sortie anticipée. L'annulation du collecteur détruit aussi ses tâches enfants. Les résultats sont indexés et retirés de la table au moment de leur réutilisation ; un index déjà consommé ne peut pas être rejoué par le lot normal.

La reprise « thinking-only » annule le collecteur précédent puis en crée un nouveau. La reprise du parseur avait une autre forme : elle réutilisait le même canal à l'intérieur du flux. Elle n'est maintenant autorisée que si aucun contenu final et aucun appel d'outil n'a été publié. Le test ajouté couvre les deux côtés de cette condition.

## Conséquences pour le plan

- l'anticipation Ollama reste en place et ne sera pas étendue aux routes cloud dans ce plan ;
- aucune nouvelle abstraction n'est ajoutée autour du collecteur ;
- le garde-fou contre la double exécution fait partie d'E07 ;
- une future suppression demandera une mesure d'un vrai outil réseau et devra montrer un gain non perceptible sur plusieurs modèles Ollama ;
- un futur changement du parseur doit préserver la règle : aucune reprise après publication d'un appel complet.

## Validation

Les mesures ont utilisé les noms réels `read_file`, `grep` et `web_search`, sauf un essai exploratoire `search_files` qui n'est pas retenu dans le tableau. Les tests ciblés couvrent l'annulation des enfants, la limite partagée, l'interception active, le refus de rejouer un résultat sous interception, la consommation des résultats anticipés et le nouveau garde-fou de reprise. Le serveur de mesure a été arrêté et aucun processus n'écoute encore le port 11501.
