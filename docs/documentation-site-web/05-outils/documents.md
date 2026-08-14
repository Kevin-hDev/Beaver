# Documents — `read_document` et `write_document`

**Emplacement site** — Outils › Documents
**Répond à** — « L'agent peut-il lire un PDF et créer un document Word ? »
**Sources** — `tool_definitions_office.rs`, `tool_document_read.rs`, `tool_document_write.rs`, `tool_document_write_styles.rs`, `tool_document_write_list.rs`, `tool_document_write_numbering.rs`, `tool_document_write_xml.rs`, `tool_office_limits.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Un groupe à activer
2. Lire un PDF ou un document Word
3. Ce que la lecture donne — et ce qu'elle perd
4. Créer un document Word
5. Les blocs disponibles
6. Les limites et les protections

---

## Contenu

### Un groupe à activer

Les deux outils forment le groupe **Document / Word**, optionnel et **éteint par défaut**.

| Outil | Rôle | Approbation |
|---|---|---|
| `read_document` | Extrait le texte d'un PDF ou d'un `.docx` | Non |
| `write_document` | Crée un document Word | **Oui** |

**Asymétrie à souligner** : Beaver lit le PDF et le Word, mais n'écrit que le Word. Il ne produit pas de PDF.

### Lire un PDF ou un document Word

**Formats acceptés** : `.pdf` et `.docx` — et rien d'autre. Ni `.doc` ancien format, ni `.odt`, ni `.rtf`, ni `.pages`.

L'agent reçoit le texte extrait, son format d'origine et **le nombre de caractères**.

### Ce que la lecture donne — et ce qu'elle perd

C'est le point que le site doit énoncer sans détour : **seul le texte est extrait**.

Sont perdus dans les deux formats :

- la mise en forme — gras, italique, couleurs, tailles ;
- les images et les schémas ;
- la mise en page — colonnes, en-têtes, pieds de page, numérotation ;
- **le contenu des tableaux, en tant que tableaux** — leur texte peut sortir, mais sa structure en lignes et colonnes est perdue.

Pour un document Word, l'extraction procède **paragraphe par paragraphe**, avec un soin particulier apporté aux espaces : le format Word découpe un paragraphe en fragments stylés, et une extraction naïve colle les mots entre eux. Beaver conserve les espaces significatifs et normalise ceux qui viennent de la mise en forme du fichier.

Pour un PDF, l'extraction dépend entièrement du fichier. **Un PDF qui n'est qu'une image scannée ne donnera aucun texte** : il n'y a pas de reconnaissance optique de caractères dans Beaver.

Le texte extrait est plafonné à **1 million de caractères**.

### Créer un document Word

**Un seul format en sortie : `.docx`.**

L'agent ne fournit pas un fichier mais **une suite de blocs de contenu**, dans l'ordre où ils doivent apparaître. C'est proche de la façon dont on rédige, et cela donne un document propre plutôt qu'un assemblage de texte brut.

### Les blocs disponibles

| Bloc | Ce qu'il produit | Options |
|---|---|---|
| **Titre** | Un titre de niveau 1 à 6 | Alignement |
| **Paragraphe** | Un paragraphe de texte | Gras, italique, alignement, ou **segments stylés** |
| **Tableau** | Un tableau avec en-têtes et lignes | — |
| **Liste** | Une liste à puces ou numérotée | Ordonnée ou non |

Le mécanisme des **segments stylés** mérite une mention : plutôt qu'un paragraphe entièrement en gras, l'agent peut composer un paragraphe de plusieurs morceaux, chacun avec son propre style — gras, italique, souligné, couleur. C'est ce qui permet d'écrire une phrase dont deux mots seulement sont mis en évidence.

L'alignement accepte quatre valeurs : à gauche, centré, à droite, justifié. Les couleurs s'expriment en hexadécimal.

**Ce qui n'existe pas** : images, en-têtes et pieds de page, notes de bas de page, sommaire automatique, sauts de page, styles nommés, modèles. L'outil produit un document structuré et lisible, pas une mise en page élaborée.

Un document est limité à **5 000 blocs**.

### Les limites et les protections

Un fichier `.docx` est une archive compressée. Les mêmes protections que pour les tableurs s'appliquent, et pour la même raison — une archive peut être fabriquée pour saturer la mémoire une fois ouverte :

- **taille du fichier source** : 50 Mo ;
- **taille du contenu interne** une fois décompressé : 10 Mo ;
- **nombre d'entrées dans l'archive** : 4 096 ;
- **taux de compression** : refusé au-delà de 100 pour 1 ;
- **taille décompressée totale** : 200 Mo.

Un document au contenu malformé est refusé avec un message clair plutôt que de produire un texte partiel présenté comme complet.

---

## Tableaux

### Les formats

| Format | Lecture | Écriture |
|---|---|---|
| `.docx` | **Oui** | **Oui** |
| `.pdf` | **Oui** | Non |
| `.doc` (ancien) | Non | Non |
| `.odt`, `.rtf`, `.pages` | Non | Non |

### Les limites

| Limite | Valeur |
|---|---|
| Taille du fichier source | **50 Mo** |
| Contenu interne d'un `.docx` décompressé | **10 Mo** |
| Texte extrait | **1 million de caractères** |
| Blocs par document créé | **5 000** |
| Entrées dans l'archive | **4 096** |
| Taux de compression accepté | **100 pour 1** |

### Les erreurs

| Message | Cause |
|---|---|
| Format non supporté. Formats acceptés : pdf, docx | Autre extension |
| Impossible de lire le fichier PDF | PDF corrompu, protégé, ou sans couche texte |
| Fichier DOCX invalide ou corrompu | Archive illisible |
| Structure DOCX invalide | Le contenu attendu est absent de l'archive |
| Document XML malformé | Contenu interne corrompu |
| Document trop volumineux | Plus d'un million de caractères |
| Document DOCX trop volumineux | Plus de 50 Mo |
| Compression excessive | Archive suspecte |

---

## Encadrés

> **Le groupe est éteint par défaut.**
> Sans activation dans Réglages › Agent › Outils, l'agent ne sait pas qu'il peut lire un PDF. Il essaiera de l'ouvrir comme du texte et échouera.

> **Seul le texte est extrait.**
> Mise en forme, images, mise en page et structure des tableaux sont perdues. L'agent lit ce qui est écrit, pas comment c'est présenté.

> **Un PDF scanné ne donne rien.**
> Il n'y a pas de reconnaissance optique de caractères. Un document numérisé sans couche texte est illisible pour l'agent.

> **Beaver n'écrit pas de PDF.**
> Il crée des documents Word. Pour obtenir un PDF, il faut convertir ensuite — par le terminal ou par un autre logiciel.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent n'ouvre pas mon PDF » | Groupe éteint par défaut | L'activer dans les réglages |
| « Le PDF est lu mais vide » | Document scanné, sans couche texte | Passer le fichier par une reconnaissance de caractères |
| « Mon tableau est illisible dans le texte extrait » | La structure des tableaux est perdue | Comportement attendu ; convertir le tableau en tableur |
| « L'agent ne peut pas lire mon `.doc` » | Ancien format non supporté | Convertir en `.docx` |
| « J'ai demandé un PDF, j'ai reçu un Word » | L'outil n'écrit que du `.docx` | Convertir ensuite |
| « Je voulais lire seulement les pages 3 à 7 » | **Le filtrage par page ne fonctionne pas** — voir ci-dessous | Le document entier est lu |
| « Document trop volumineux » | Plus d'un million de caractères | Découper le document |

---

## Renvois

- `05-outils/tableurs.md` — pour les données tabulaires
- `05-outils/images.md`
- `05-outils/fichiers.md` — pourquoi `read_file` échoue sur un PDF
- `12-reference/formats-supportes.md`
- `03-interface/arbre-de-fichiers-et-previews.md` — la prévisualisation de documents

---

## Points à confirmer

- **Le paramètre de plage de pages des PDF est annoncé au modèle mais ignoré par le code.** La définition de l'outil indique que l'agent peut demander « les pages 1 à 5 » ; l'implémentation reçoit ce paramètre et ne s'en sert pas — le document entier est toujours extrait. **Défaut réel, à corriger côté produit** : soit implémenter le filtrage, soit retirer le paramètre de la définition. En attendant, ne rien promettre de tel sur le site. C'est le seul écart franc entre promesse et implémentation relevé dans toute la section 05.
- La **limite d'un million de caractères** s'applique à l'extraction Word. Je n'ai **pas vérifié** qu'une limite équivalente protège l'extraction PDF — le code de lecture PDF ne semble pas en poser. À faire vérifier : un PDF très volumineux pourrait produire un texte sans borne.
- Je n'ai **pas lu en détail** le module de numérotation des listes. La distinction puces / numéros vient de la définition de l'outil.
- Je n'ai **pas vérifié à l'écran** l'affichage d'un document lu dans la conversation ni sa prévisualisation.
