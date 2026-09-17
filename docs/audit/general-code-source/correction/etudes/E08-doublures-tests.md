# E08 — Valeur des abstractions et doublures de tests

Date : 17 septembre 2026
Commit de départ : `6d2d1150`

## Décision

Conserver les trois ensembles examinés : la projection du faux transport Codex, les doublures Ollama et `InstallSignal`. Leur volume est réel, mais chaque variante vérifie une garantie différente. Les fusionner déplacerait ces différences dans une doublure à options plus longue et moins lisible, sans réduction nette démontrée.

Aucun code de production ni test n'est supprimé dans E08.

## Faux transport Codex

Le serveur de test reçoit le vrai corps HTTP construit par le client. Ce corps peut contenir des messages, résultats d'outils et marqueurs sensibles, même si le jeton OAuth reste dans les en-têtes. La projection manuelle ne conserve que les sept informations nécessaires aux assertions, borne la taille, la profondeur, le nombre d'éléments et la longueur des clés, puis zéroïse les buffers temporaires.

Un remplacement par `serde_json::Value` matérialiserait toutes les valeurs ignorées. Un visiteur Serde personnalisé devrait réimplémenter les mêmes bornes, la sélection des champs et l'effacement ; il ne démontrerait donc pas le gain annoncé. La projection reste l'autorité du faux transport sensible.

## Doublures Ollama

Les trois implémentations de `OllamaDurableFs` ne sont pas interchangeables :

| Doublure | Garantie propre |
|---|---|
| `ScriptedFs` | vérifie l'ordre exact des primitives du journal et chaque frontière de publication atomique |
| `CutpointFs` | modélise rapidement l'état logique des répertoires pour les décisions de reprise |
| `RealCutpointFs` | exécute les vraies opérations disque et injecte les coupures autour de `rename`, suppression et synchronisation |

Remplacer les deux premières par `RealCutpointFs` ferait perdre les assertions d'ordre et ralentirait les matrices de reprise. Étendre `RealCutpointFs` pour les récupérer recréerait leurs états en mémoire au-dessus du disque.

Les deux implémentations de `UpdateBackend` portent aussi deux phases différentes. `FakeBackend` vérifie la préparation, l'ordre arrêt/récolte/renommage, les métadonnées et la propriété du sidecar. `CompletionHarness` modélise les dispositions après publication et leur convergence après chaque coupure de nettoyage ou de retour arrière. Leur fusion exigerait des champs optionnels, deux vocabulaires de coupure et des branches par mode.

## `InstallSignal`

La production passe par `InstallControl`, qui persiste les phases, le budget, la source résolue, le verrou de dépendances et l'identité du processus. Les tests git et npm emploient `ServiceWorkCancellation` pour exercer directement le transport, la résolution et le lancement sans fabriquer un travail durable complet.

Remplacer le trait par `InstallControl` couplerait ces tests au magasin des travaux et à ses checkpoints. Créer un nouveau faux minimal laisserait toujours une deuxième implémentation. Le trait actuel est donc la plus petite frontière commune entre l'installation durable et les tests ciblés ; ses méthodes par défaut décrivent précisément le comportement sans journal dont ces tests ont besoin.

## Validation

Les suites ciblées réussissent :

- faux transport Codex : 19 tests ;
- système de fichiers durable Ollama : 28 tests ;
- reprise Ollama : 27 tests ;
- ordre et reprise de mise à jour : 6 + 6 tests ;
- fin de mise à jour : 5 tests ;
- sources git, exécution npm et dépendances git : 5 + 6 + 1 tests.

Total : 103 tests réussis, aucun échec. Deux premiers filtres de mise à jour ont exécuté zéro test ; ils n'ont pas été comptés et les chemins de modules exacts ont été relancés avec 12 réussites.

## Condition de réexamen

Réexaminer seulement si deux doublures finissent par porter le même état, les mêmes points de coupure et les mêmes assertions. Une simple ressemblance de trait ne suffit pas : la réduction doit rester nette après ajout des modes nécessaires et conserver les garanties d'effacement, d'annulation, de reprise et d'ordre.
