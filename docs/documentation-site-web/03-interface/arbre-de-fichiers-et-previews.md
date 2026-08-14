# Arbre de fichiers et prévisualisations

**Emplacement site** — Interface › Fichiers et prévisualisations
**Répond à** — « Comment je navigue dans mes fichiers, et que puis-je consulter sans quitter Beaver ? »
**Sources** — `src-tauri/src/commands/file_preview.rs` (lignes 6-8), `file_preview_office.rs` (lignes 4-7, 48-72), `file_preview_editors/`, `src-tauri/src/commands/file_tree.rs`, `file_tree_watcher.rs`, `src-tauri/src/services/file_watcher.rs` (ligne 15), `src/components/file-tree/`, `src/components/file-preview/`
**Vérification** — Vérifié dans le code : formats acceptés, limites de taille et délai de surveillance

---

## Plan de page proposé

1. L'arbre de fichiers
2. La surveillance des modifications
3. Les prévisualisations
4. Les formats acceptés
5. Les limites de taille
6. Ouvrir dans un éditeur externe

---

## Contenu

### 1. L'arbre de fichiers

Navigation dans le répertoire de travail de la conversation, sans quitter l'application. Un clic sur un fichier ouvre sa prévisualisation dans le panneau latéral.

### 2. La surveillance des modifications

- Les changements sur le disque sont détectés automatiquement.
- Un délai de regroupement de **200 millisecondes** évite de recalculer l'affichage à chaque écriture. Une compilation qui touche cent fichiers ne provoque pas cent rafraîchissements.
- Le mécanisme s'appuie sur les notifications natives de chaque système.

### 3. Les prévisualisations

Quatre familles, chacune avec son traitement :

- **Texte** — affichage direct, avec coloration syntaxique selon le thème de code choisi.
- **Images** — affichage direct.
- **Tableurs** — rendu tabulaire, avec choix de la feuille pour les classeurs qui en comptent plusieurs.
- **Documents** — extraction du contenu pour les formats `docx` et `pdf`.

S'y ajoutent les **aperçus de liens** : quand une réponse contient une URL, Beaver peut en afficher un résumé.

### 4. Les formats acceptés

Tableau complet en section Tableaux. À retenir :

- Tableurs : **`csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm`**
- Documents : **`docx`, `pdf`** — et rien d'autre. Un `.doc` ancien, un `.odt` ou un `.rtf` ne sont pas prévisualisables.
- Tout autre format renvoie « Format non supporté ».

### 5. Les limites de taille

| Type | Limite |
|---|---|
| Fichier texte | **2 Mo** |
| Document `docx` ou `pdf` | **50 Mo** |
| Tableur | **50 Mo** |

Autres bornes :

- **500 lignes** affichées par défaut pour un tableur, **5 000** au maximum.
- Longueur d'un chemin : **4 096 caractères**.
- Vérification d'existence en lot : **500 fichiers** par appel.

La limite de 2 Mo sur le texte surprend quand on ouvre un gros journal ou un jeu de données. Elle mérite d'être annoncée plutôt que découverte.

### 6. Ouvrir dans un éditeur externe

- Beaver détecte les éditeurs installés et propose d'ouvrir le fichier dans l'un d'eux.
- La détection est propre à chaque système : trois implémentations distinctes existent pour macOS, Windows et Linux.
- Le chemin de l'éditeur est validé avant lancement.

---

## Tableaux

### Tableau — Formats prévisualisables

| Famille | Extensions | Limite |
|---|---|---|
| Texte | Formats texte courants | 2 Mo |
| Tableur | `csv`, `tsv`, `xlsx`, `xls`, `ods`, `xlsm` | 50 Mo |
| Document | `docx`, `pdf` | 50 Mo |
| Image | Formats image courants | — |

### Tableau — Bornes d'affichage des tableurs

| | Valeur |
|---|---|
| Lignes affichées par défaut | 500 |
| Lignes affichées au maximum | 5 000 |
| Choix de la feuille | Oui |

---

## Encadrés

**Encadré « Deux mégaoctets pour le texte »**
> Les fichiers texte sont prévisualisables jusqu'à 2 Mo. Au-delà, ouvrez le fichier dans un éditeur externe — un journal ou un jeu de données dépasse vite cette taille.

**Encadré « Documents : docx et pdf uniquement »**
> Seuls les formats `docx` et `pdf` sont prévisualisables. Les formats `.doc`, `.odt` et `.rtf` ne le sont pas.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Format non supporté » | Extension hors des listes ci-dessus | Ouvrir dans un éditeur externe |
| Un gros fichier texte refuse de s'ouvrir | Plafond de 2 Mo | Éditeur externe |
| Un tableur est tronqué | 500 lignes par défaut | Augmenter jusqu'à 5 000, ou ouvrir dans un tableur |
| L'arbre ne reflète pas un changement | Regroupement de 200 ms, ou surveillance non déclenchée | Attendre, puis rafraîchir |
| Aucun éditeur externe proposé | Aucun éditeur reconnu sur le système | Ouvrir le fichier manuellement |

---

## Renvois

- *Interface › Panneau latéral* — où s'affichent les prévisualisations
- *Outils › Tableurs* et *Outils › Documents* — ce que l'agent sait lire et écrire
- *Agent › Répertoire de travail* — ce que l'arbre affiche
- *Référence › Formats supportés*

---

## Points à confirmer

- **La liste exacte des extensions texte et image reconnues.** Les limites sont vérifiées, mais la liste des extensions traitées comme du texte ou comme une image n'a pas été extraite. Nécessaire pour la page *Référence › Formats supportés*.
- **Les éditeurs externes réellement détectés** sur chaque système. Trois fichiers d'implémentation existent ; leur contenu n'a pas été lu.
- **Le comportement de l'aperçu de liens.** Le service existe mais son déclenchement n'a pas été vérifié : automatique, à la demande, sur quels domaines ?
- **La prévisualisation est-elle éditable ?** Un dossier `file_preview_editors` existe, mais il concerne l'ouverture externe. Vérifier si l'on peut modifier un fichier directement dans le panneau.
- **Le champ d'action de la surveillance.** `file_watcher.rs` surveille des fichiers de configuration précis ; `file_tree_watcher.rs` surveille l'arborescence de travail. Vérifier que le délai de 200 ms s'applique bien aux deux.
