# Images — `transform_image`

**Emplacement site** — Outils › Images
**Répond à** — « L'agent peut-il redimensionner, recadrer ou convertir mes images ? »
**Sources** — `tool_definitions_office.rs`, `tool_image_process.rs`, `tool_image_process_geometry.rs`, `tool_image_inspect.rs`, `tool_office_limits.rs`, `permission_gate.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Un outil, trois usages
2. Inspecter une image
3. Redimensionner
4. Recadrer
5. Convertir
6. La qualité
7. Les limites

---

## Contenu

### Un outil, trois usages

`transform_image` forme à lui seul le groupe **Images**, optionnel et **éteint par défaut**.

Un seul outil couvre trois intentions, distinguées par ce que l'agent fournit :

| Intention | Ce que fournit l'agent | Écrit un fichier |
|---|---|---|
| **Inspecter** | Une image, une liste d'opérations vide, **pas** de destination | Non |
| **Convertir** | Une image et une destination avec une autre extension, **pas** d'opérations | Oui |
| **Transformer** | Une image, une destination, et des opérations | Oui |

La combinaison « liste d'opérations vide **et** destination » est **refusée** comme ambiguë, avec un message qui explique les deux formes correctes. C'est un bon exemple de conception : plutôt que de deviner l'intention, l'outil demande de la clarifier.

**L'approbation dépend de l'intention** : inspecter ne demande rien, transformer et convertir déclenchent une demande en mode Demande d'approbation. C'est cohérent — l'inspection ne touche à rien.

### Inspecter une image

Renvoie les **dimensions**, le **format** et la **taille du fichier**, sans rien écrire.

C'est l'opération que l'agent utilise pour répondre à « quelle taille fait cette image ? », ou pour préparer un redimensionnement en connaissant les proportions d'origine.

### Redimensionner

Trois modes, et le choix change le résultat :

| Mode | Comportement |
|---|---|
| **Ajuster** (par défaut) | L'image tient dans les dimensions demandées **en gardant ses proportions** — le résultat peut être plus petit que demandé |
| **Remplir** | L'image remplit exactement les dimensions demandées, **en rognant** ce qui dépasse |
| **Exact** | L'image prend exactement les dimensions demandées, **quitte à la déformer** |

Le rééchantillonnage utilise un filtre de haute qualité — le résultat d'une réduction est net, pas crénelé.

### Recadrer

L'agent donne un point de départ et une taille. Un recadrage qui sortirait de l'image est **refusé** avec un message explicite, plutôt que d'être silencieusement ramené aux bords.

### Convertir

**Formats de sortie** : JPEG, PNG, WebP, GIF, BMP. Le format est déterminé par **l'extension du fichier de destination** — il n'y a pas de paramètre de format séparé.

Pour une conversion simple, l'agent ne fournit aucune opération : une image d'entrée, une destination avec la bonne extension, et c'est tout.

### La qualité

Le réglage de qualité va de **1 à 100** et ne s'applique **qu'au JPEG**.

Le comportement sur les autres formats mérite d'être écrit, parce qu'il est honnête et rare :

- sur du **JPEG**, la qualité est appliquée ;
- sur du **WebP**, elle est **ignorée** — l'encodage est sans perte — et le résultat porte un avertissement qui le dit ;
- sur **tout autre format**, elle est ignorée, avec un avertissement également.

L'agent n'apprend donc pas après coup que son réglage n'a servi à rien : il est prévenu dans le résultat même.

Autre détail de la même veine : quand Beaver ne parvient pas à relire la taille du fichier produit, il **le signale** au lieu d'annoncer une taille de zéro.

### Les limites

| Limite | Valeur |
|---|---|
| Taille du fichier source | **50 Mo** |
| Dimension maximale, en largeur comme en hauteur | **8 000 pixels** |
| Nombre total de pixels après transformation | **50 millions** |
| Opérations par appel | **128** |
| Qualité | **1 à 100** |

La limite en nombre de pixels est vérifiée **avant** la transformation. Sans elle, un redimensionnement demandant 8 000 × 8 000 pixels ferait allouer plusieurs centaines de mégaoctets de mémoire.

---

## Tableaux

### Les opérations

| Opération | Paramètres | Notes |
|---|---|---|
| Redimensionner | Largeur, hauteur, mode | Mode « ajuster » par défaut |
| Recadrer | Position de départ, largeur, hauteur | Refusé si hors limites |
| Qualité | Valeur de 1 à 100 | **JPEG uniquement** |

Les opérations s'appliquent **dans l'ordre donné** : recadrer puis redimensionner ne donne pas le même résultat que l'inverse.

### Les formats

| Format | Entrée | Sortie |
|---|---|---|
| PNG | Oui | **Oui** |
| JPEG | Oui | **Oui** |
| WebP | Oui | **Oui** (sans perte) |
| GIF | Oui | **Oui** |
| BMP | Oui | **Oui** |
| Autres formats courants | Selon la bibliothèque | Non |

### Les erreurs

| Message | Cause |
|---|---|
| Demande ambiguë | Liste d'opérations vide **avec** une destination |
| `output_path` requis | Transformation demandée sans destination |
| Image trop volumineuse | Fichier au-delà de 50 Mo |
| Dimensions invalides | Image dépassant 8 000 pixels de côté |
| Impossible d'ouvrir l'image | Fichier corrompu ou format non reconnu |
| Recadrage hors limites | La zone demandée sort de l'image |
| Mode de redimensionnement invalide | Mode autre que les trois acceptés |
| Qualité hors bornes | Valeur en dehors de 1 à 100 |
| Trop d'opérations | Plus de 128 |
| Opération inconnue | Type d'opération non reconnu |

---

## Encadrés

> **Le groupe est éteint par défaut.**
> Sans activation dans Réglages › Agent › Outils, l'agent ne sait pas qu'il peut traiter une image.

> **Inspecter ne demande aucune approbation.**
> C'est la seule des trois intentions qui n'écrit rien. Demander les dimensions d'une image ne déclenche jamais de validation.

> **La qualité ne concerne que le JPEG.**
> Sur les autres formats, le réglage est ignoré — et le résultat le dit explicitement plutôt que de laisser croire qu'il a été appliqué.

> **Le format de sortie vient de l'extension.**
> Écrire vers un fichier en `.webp` produit du WebP. Il n'y a pas d'autre façon de choisir le format.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne traite pas mes images » | Groupe éteint par défaut | L'activer dans les réglages |
| « Demande ambiguë » | Opérations vides et destination fournie | L'agent corrige seul |
| « Mon image redimensionnée est plus petite que demandé » | Mode « ajuster » : les proportions sont préservées | Utiliser « remplir » ou « exact » |
| « Mon image est déformée » | Mode « exact » | Utiliser « ajuster » ou « remplir » |
| « La qualité n'a rien changé » | Format autre que JPEG | L'avertissement le signale ; convertir en JPEG |
| « Dimensions invalides » | Image de plus de 8 000 pixels de côté | La réduire d'abord avec un autre outil |
| « Recadrage hors limites » | Zone dépassant les bords | Inspecter l'image d'abord pour connaître ses dimensions |

---

## Renvois

- `05-outils/vue-densemble.md` — activer un groupe d'outils
- `04-agent/pieces-jointes.md` — joindre une image à un message, ce qui est différent
- `05-outils/fichiers.md` — pourquoi `read_file` échoue sur une image
- `12-reference/formats-supportes.md`

---

## Points à confirmer

- **La distinction entre traiter une image et la regarder** doit être limpide sur le site. `transform_image` manipule un fichier — il redimensionne, recadre, convertit. Il **ne montre pas** l'image au modèle. Pour qu'un modèle voie le contenu d'une image, il faut la joindre au message et disposer d'un modèle capable de traiter des images. Les deux mécanismes n'ont rien à voir et la confusion est certaine. Vérifier ce point avec l'équipe avant de rédiger la page.
- La **liste exacte des formats d'entrée acceptés** dépend de la bibliothèque utilisée et n'est pas fixée dans le code de Beaver. Le tableau ci-dessus liste ceux qui sont sûrs en sortie. À compléter ou à laisser volontairement vague.
- Je n'ai **pas vérifié à l'écran** si l'image produite s'affiche dans la conversation ou seulement son chemin.
- Le comportement sur une **image animée** (GIF de plusieurs images) n'a pas été vérifié : redimensionner ne conserve probablement que la première image.
