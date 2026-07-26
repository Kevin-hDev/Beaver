# Revue différentielle Forecast — vérification complémentaire

## Résumé exécutif

| Sévérité | Nombre |
|---|---:|
| Critique | 0 |
| Haute | 0 |
| Moyenne | 0 |
| Faible | 0 restant |
| Faux positifs ou comportements acceptables | 4 |

**Risque global :** faible, sans problème ouvert
**Recommandation :** approuver le commit `4081fb35`.

La vulnérabilité XLSX signalée n'est pas présente. Toutes les valeurs texte sont
écrites avec `rust_xlsxwriter::Worksheet::write_string`, qui produit une cellule
texte. Une formule nécessite l'API distincte `write_formula` et une cellule XML
contenant l'élément `<f>`.

## Périmètre

- Commit étudié : `a5a4fbb0..4081fb35`
- Fichiers directement concernés par les cinq signalements : 8
- Dépendance vérifiée : `rust_xlsxwriter 0.82.0`
- Stratégie : ciblée, avec lecture du diff, historique Git, appels directs,
  tests associés et implémentation locale de la dépendance XLSX.

## Résultats

### 1. Injection de formule XLSX — faux positif

**Fichiers :**

- `src-tauri/src/services/forecast/export/xlsx.rs:57-60`
- `src-tauri/src/services/forecast/export/xlsx.rs:94-97`
- `src-tauri/src/services/forecast/export/xlsx.rs:136-141`
- `src-tauri/src/services/forecast/export/xlsx.rs:168-175`
- `src-tauri/src/services/forecast/export/xlsx_input.rs:24-30`
- `src-tauri/src/services/forecast/export/xlsx_advanced.rs:92-109`

Toutes les données textuelles, y compris celles provenant des notes et des
données d'entrée, passent par `write_string`. Dans `rust_xlsxwriter 0.82.0`,
cette API appelle `store_string`, puis génère une cellule XML de type chaîne
(`t="s"` ou `t="inlineStr"`). Les formules passent par une autre API et sont
écrites avec un élément XML `<f>`.

Une valeur comme `=HYPERLINK(...)` reste donc du texte dans le classeur. Lui
appliquer le préfixe CSV ajouterait une apostrophe visible sans améliorer la
sécurité.

**Statut : résolu.** Un test XLSX de non-régression exporte désormais une note
commençant par `=` et vérifie la présence de la valeur comme chaîne ainsi que
l'absence d'élément `<f>` dans les feuilles.

### 2. Correspondance de `meta_value` — faux positif

**Fichier :** `src-tauri/src/services/forecast/notes_format.rs:62-66`

Le préfixe recherché inclut les deux-points : `id:`. La chaîne `identifier:` ne
commence pas par `id:` puisque son troisième caractère est `e`, pas `:`.

Les identifiants lus sont en plus comparés aux identifiants attendus du chemin,
puis validés. Le scénario décrit ne fonctionne donc pas.

### 3. Création d'un dossier vide pendant la lecture — positif faible

**Fichiers :**

- `src-tauri/src/services/forecast/notes_files.rs:12-14`
- `src-tauri/src/services/forecast/notes_files.rs:94-111`

`notes::list` effectue volontairement une réparation automatique : les fichiers
manquants sont reconstruits depuis les annotations sauvegardées. Cette lecture
peut donc légitimement écrire.

Cependant, quand l'analyse ne contient aucune annotation et aucun fichier de
note, un dossier vide est créé inutilement. Il n'y a ni fuite de données ni
problème de permissions, mais un petit résidu disque.

**Statut : résolu.**

1. `sync_annotation_files` retourne immédiatement quand la liste des
   annotations est vide ;
2. `read_notes` utilise `directory_if_exists` et retourne une liste vide
   si le dossier n'existe pas.

### 4. `upsert` avant suppression — faux positif

**Fichiers :**

- `src-tauri/src/services/forecast/notes.rs:95-113`
- `src-tauri/src/services/forecast/notes_annotations.rs:7-30`

Cette étape est un point de récupération, pas un doublon. Pour une ancienne
annotation, elle sauvegarde d'abord le contenu complet de la note dans
l'analyse. Si l'application s'arrête après la suppression du fichier mais avant
la sauvegarde finale, la note peut alors être reconstruite sans perdre son
titre, son type ou son contenu.

Pour les annotations déjà à jour, `upsert` retourne `false` et aucune sauvegarde
intermédiaire n'est faite. Supprimer cette logique réintroduirait une perte
possible lors d'un arrêt au mauvais moment.

### 5. Nettoyage différé — comportement acceptable

**Fichier :** `src-tauri/src/services/forecast/notes_cleanup.rs:22-25`

Le nettoyage n'est pas abandonné : le dossier conserve le nom
`.delete-<analysis_id>` et sera repris au prochain démarrage ou avant la
prochaine suppression. Un message générique est déjà écrit dans les journaux.

Le projet n'utilise pas directement `tracing` pour ses journaux applicatifs et
emploie déjà `eprintln!` dans le cycle Forecast. Ajouter `tracing::warn` ici
introduirait une nouvelle infrastructure sans bénéfice concret. Ne pas afficher
d'erreur utilisateur est cohérent puisque l'analyse elle-même a bien été
supprimée.

## Couverture et rayon d'impact

| Fonction | Appels directs de production | Couverture |
|---|---:|---|
| `notes::list` | 2 | réparation, corruption et export incomplet testés |
| `notes::delete` | 1 | révision persistée et rollback testés |
| `notes_cleanup::delete_analysis` | 1 | staging et récupération testés |
| export XLSX | 1 | comportement sûr par API, test anti-formule explicite conseillé |

Les changements critiques de stockage ont des appels directs peu nombreux. Les
tests du commit couvrent la corruption, les permissions, les liens symboliques,
la reconstruction, les révisions et la récupération après suppression.

## Recommandations

### Corrections réalisées

- La création du dossier vide lors d'une liste sans note est évitée.
- Un test XLSX explicite documente durablement que
  `write_string` conserve les valeurs commençant par `=` comme texte.

### À ne pas modifier

- Ne pas préfixer les cellules XLSX déjà écrites avec `write_string`.
- Ne pas supprimer l'`upsert` de récupération précédant la suppression d'une
  note.
- Ne pas ajouter `tracing` uniquement pour le nettoyage différé.

## Limites et confiance

Cette vérification porte sur les cinq signalements fournis, leurs appels directs
et le comportement de `rust_xlsxwriter 0.82.0`. Elle ne constitue pas une
nouvelle revue exhaustive de toute la feature Forecast.

**Confiance : élevée** pour les cinq points analysés.
