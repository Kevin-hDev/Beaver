# Tableurs — `read_spreadsheet` et `write_spreadsheet`

**Emplacement site** — Outils › Tableurs
**Répond à** — « L'agent peut-il lire et créer des fichiers Excel ? »
**Sources** — `tool_definitions_office.rs`, `tool_spreadsheet_read.rs`, `tool_spreadsheet_calamine.rs`, `tool_spreadsheet_write.rs`, `tool_spreadsheet_range.rs`, `tool_spreadsheet_error.rs`, `tool_office_limits.rs`, `tool_office_array.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Un groupe à activer
2. Lire un tableur
3. Ce que l'agent reçoit
4. Écrire un fichier Excel
5. Les opérations disponibles
6. Les limites et les protections

---

## Contenu

### Un groupe à activer

Les deux outils forment le groupe **Spreadsheet / Excel**, optionnel et **éteint par défaut**.

Tant qu'il est éteint, l'agent **ignore que ces outils existent**. Une demande « ouvre-moi ce fichier Excel » se soldera par un contournement — une tentative de lecture en texte brut, qui échouera — plutôt que par un message clair.

C'est le piège le plus courant de toute la section 05. Il mérite un encadré.

| Outil | Rôle | Approbation |
|---|---|---|
| `read_spreadsheet` | Lit les données d'un tableur | Non |
| `write_spreadsheet` | Crée ou modifie un fichier Excel | **Oui** |

### Lire un tableur

**Formats acceptés en lecture** : `.xlsx`, `.xls`, `.xlsm`, `.ods`, `.csv`, `.tsv`.

- La **première ligne sert toujours d'en-têtes** de colonnes. Ce n'est pas configurable.
- Sur un classeur, l'agent peut viser **une feuille précise** — la première par défaut — et **une plage de cellules**.
- Sur un fichier CSV ou TSV, la feuille et la plage sont ignorées, et **le séparateur est détecté automatiquement** en analysant la première ligne : tabulation, point-virgule ou virgule.
- **Les formules sont rendues telles quelles**, sous forme de texte. L'agent voit la formule, pas son résultat calculé. C'est un point important à écrire : demander « quel est le total » sur une cellule contenant une somme renverra la formule, à charge pour le modèle de la comprendre ou de la recalculer.

### Ce que l'agent reçoit

Un ensemble structuré contenant :

- le nom de la feuille lue ;
- **la liste de toutes les feuilles du classeur** — l'agent sait donc qu'il y en a d'autres ;
- les en-têtes ;
- les lignes de données ;
- **le nombre total de lignes**, même quand toutes ne sont pas transmises ;
- un indicateur de troncature.

Ce dernier point est bien conçu : l'agent sait qu'il n'a pas tout vu, et peut demander la suite.

**Limites de lecture** : **500 lignes** par défaut, **5 000** au maximum, et **1 000 colonnes**.

### Écrire un fichier Excel

**Un seul format en écriture : `.xlsx`.**

L'agent ne fournit pas un fichier complet mais **une liste d'opérations** à appliquer, ce qui permet de modifier un fichier existant sans le réécrire.

Un fichier neuf reçoit une feuille par défaut. Il n'est pas nécessaire d'en créer une pour un classeur à une seule feuille — c'est même explicitement déconseillé dans la définition de l'outil.

Chaque opération peut viser **une feuille précise**, la première par défaut.

### Les opérations disponibles

| Opération | Ce qu'elle fait |
|---|---|
| Écrire une cellule | Pose une valeur dans une cellule |
| Écrire une formule | Pose une formule |
| Écrire une ligne | Remplit une ligne entière d'un coup |
| Ajouter une feuille | Crée une feuille supplémentaire |
| Largeur de colonne | Ajuste une colonne |
| Hauteur de ligne | Ajuste une ligne |
| Mise en forme | Gras, italique, souligné, couleur de texte, couleur de fond, taille de police |
| Format de nombre | Décimales, date, monnaie, séparateur de milliers |
| Bordure | Fine, moyenne ou épaisse, sur les côtés choisis |
| Fusion de cellules | Fusionne un rectangle de cellules |

Les couleurs s'expriment en hexadécimal. La mise en forme peut au passage réécrire la valeur de la cellule, ce qui évite deux opérations.

**Ce qui n'existe pas** : formules calculées, graphiques, tableaux croisés dynamiques, filtres, mise en forme conditionnelle, images. L'outil écrit des données et de l'apparence, pas des objets Excel avancés.

### Les limites et les protections

Les fichiers de bureautique modernes sont des archives compressées. Une archive peut être fabriquée pour occuper une place démesurée une fois décompressée — c'est une attaque connue. Beaver s'en protège :

- **taille du fichier source** : 50 Mo maximum ;
- **nombre d'entrées dans l'archive** : 4 096 ;
- **taux de compression** : au-delà de 100 pour 1, l'archive est refusée ;
- **taille totale décompressée** : 200 Mo.

S'ajoute une protection propre aux classeurs : une feuille peut **déclarer** des dimensions énormes tout en pesant quelques kilooctets. Beaver vérifie le produit lignes × colonnes **avant** de charger la feuille et refuse au-delà de **5 millions de cellules**. Sans ce contrôle, un petit fichier pourrait faire consommer plusieurs gigaoctets de mémoire.

Enfin, le nombre d'opérations d'écriture est plafonné à **10 000**.

---

## Tableaux

### Les formats

| Extension | Lecture | Écriture |
|---|---|---|
| `.xlsx` | **Oui** | **Oui** |
| `.xlsm` | Oui | Non |
| `.xls` | Oui | Non |
| `.ods` | Oui | Non |
| `.csv` | Oui | Non |
| `.tsv` | Oui | Non |

### Les limites

| Limite | Valeur |
|---|---|
| Lignes lues par défaut | **500** |
| Lignes lues au maximum | **5 000** |
| Colonnes lues | **1 000** |
| Taille du fichier source | **50 Mo** |
| Cellules chargeables dans une feuille | **5 millions** |
| Opérations d'écriture | **10 000** |
| Entrées dans l'archive | **4 096** |
| Taux de compression accepté | **100 pour 1** |
| Taille décompressée totale | **200 Mo** |

---

## Encadrés

> **Le groupe est éteint par défaut.**
> Sans activation dans Réglages › Agent › Outils, l'agent ne sait pas qu'il peut lire un tableur. Il essaiera de le lire comme du texte et échouera, sans dire pourquoi.

> **Les formules ne sont pas calculées.**
> L'agent voit `=SOMME(A1:A5)`, pas le total. Pour obtenir des valeurs, il faut un fichier qui les contient, ou laisser l'agent faire le calcul lui-même.

> **La première ligne est toujours l'en-tête.**
> Un fichier qui commence par un titre ou une ligne vide sera mal interprété. C'est une contrainte, pas un réglage.

> **Écriture en `.xlsx` uniquement.**
> Beaver lit six formats et n'en écrit qu'un. Un fichier `.ods` ou `.csv` peut être lu, pas modifié.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent n'ouvre pas mon fichier Excel » | Groupe éteint par défaut | L'activer dans les réglages |
| « L'agent me donne une formule au lieu du résultat » | Les formules sont rendues telles quelles | Comportement attendu |
| « Les colonnes sont décalées » | La première ligne n'était pas un en-tête | Ajouter une ligne d'en-têtes |
| « L'agent n'a vu que 500 lignes » | Limite par défaut | Il peut en demander jusqu'à 5 000 |
| « Format non supporté » | Extension hors de la liste | Convertir en `.xlsx` ou `.csv` |
| « Feuille trop volumineuse » | Plus de 5 millions de cellules déclarées | Réduire la plage utilisée dans le fichier |
| « Compression excessive » | Archive suspecte ou très inhabituelle | Rouvrir et réenregistrer le fichier depuis un tableur |
| « L'agent ne peut pas modifier mon `.csv` » | Écriture en `.xlsx` seulement | Lui demander d'écrire un `.xlsx`, ou passer par le terminal |

---

## Renvois

- `05-outils/documents.md` — les documents Word et PDF
- `05-outils/vue-densemble.md` — activer un groupe d'outils
- `05-outils/fichiers.md` — pourquoi `read_file` échoue sur un tableur
- `12-reference/formats-supportes.md`
- `03-interface/arbre-de-fichiers-et-previews.md` — la prévisualisation de tableurs dans l'interface

---

## Points à confirmer

- Je n'ai **pas lu en détail** le module d'écriture des mises en forme. La liste des opérations vient de la définition de l'outil, qui fait autorité pour ce que l'agent peut demander, mais les cas limites — fusionner des cellules déjà fusionnées, formats de nombres exotiques — ne sont pas vérifiés.
- **La syntaxe de plage de cellules** est mentionnée mais je n'ai pas lu sa validation. À vérifier si le site donne des exemples.
- Je n'ai **pas vérifié à l'écran** l'affichage d'un tableur lu dans la conversation, ni la prévisualisation dans le panneau latéral.
- Le comportement sur un fichier **CSV dont la première ligne contient un seul champ** — donc sans séparateur détectable — n'a pas été vérifié : le code retombe sur la virgule par défaut.
