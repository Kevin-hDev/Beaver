# Recherche dans les fichiers — `grep` et `glob`

**Emplacement site** — Outils › Recherche de fichiers
**Répond à** — « Comment l'agent retrouve un fichier ou une ligne de code dans mon projet ? »
**Sources** — `tool_grep.rs`, `tool_glob.rs`, `tool_scan_timeout.rs`, `tool_definitions_search.rs`, `tool_result_truncate.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Deux outils, deux questions
2. Chercher dans le contenu
3. Chercher par nom de fichier
4. Ce qui est fouillé — et ce que ça implique
5. Les limites de résultats
6. Quand une recherche est trop longue
7. Ce qui échappe à ces outils

---

## Contenu

### Deux outils, deux questions

Ils forment le groupe **Recherche de fichiers**, **verrouillé** — impossible à désactiver.

| Question de l'agent | Outil |
|---|---|
| « Où est écrit ce mot dans le projet ? » | `grep` — cherche dans le **contenu** |
| « Quels fichiers portent ce nom ? » | `glob` — cherche par **nom de fichier** |

Aucun des deux ne modifie quoi que ce soit. Ils ne demandent jamais d'approbation.

Ils partent du répertoire de travail par défaut, et peuvent viser un sous-dossier. Comme partout, le dossier ciblé doit rester dans les zones autorisées en lecture.

**Effet secondaire à connaître** : un fichier apparu dans un résultat de recherche est enregistré comme « vu ». Il devient donc modifiable sans nouvelle lecture — voir la règle « lire avant d'écrire » dans `05-outils/fichiers.md`.

### Chercher dans le contenu

`grep` cherche une **expression régulière** dans les fichiers.

- Chaque correspondance revient sous la forme `chemin:ligne:contenu`. Le numéro de ligne est **toujours** présent : l'agent peut donc citer un emplacement exact.
- La recherche **distingue les majuscules des minuscules**, sans possibilité de l'en empêcher.
- Il n'y a **pas de lignes de contexte** autour d'une correspondance, pas de mode « liste des fichiers seulement », pas de mode comptage. Un seul mode : les lignes qui correspondent.
- Un filtre par nom de fichier peut restreindre la recherche — par exemple aux fichiers d'une seule extension. C'est ce qui rend une recherche rapide sur un gros projet.
- Le motif est limité à **500 caractères**.
- La syntaxe des expressions régulières est celle de Rust : pas de références arrière, pas d'anticipation. Un motif qui les emploie est rejeté avec un message explicite, et l'agent le reformule.

### Chercher par nom de fichier

`glob` liste les fichiers dont le chemin correspond à un motif de nom.

- La syntaxe est la syntaxe classique des motifs de fichiers : `*` pour n'importe quelle suite de caractères, `?` pour un caractère, `**` pour traverser les sous-dossiers, `[abc]` pour un choix de caractères, `{a,b}` pour une alternative.
- **Seuls les fichiers sont renvoyés**, jamais les dossiers.
- Les chemins reviennent **absolus**, un par ligne.
- **L'ordre n'est pas garanti** : il dépend du système de fichiers, et n'est pas alphabétique. Ce point compte pour qui compare deux exécutions.
- Le motif s'applique au chemin **relatif** au dossier de départ, pas au chemin absolu.

### Ce qui est fouillé — et ce que ça implique

**Les deux outils ignorent complètement les règles d'exclusion de Git.** C'est un choix explicite, et il a des conséquences que le site doit énoncer :

- les fichiers **cachés** sont fouillés ;
- les fichiers listés dans un `.gitignore` sont fouillés — y compris `node_modules`, les dossiers de compilation, les caches ;
- les fichiers d'environnement et de secrets présents dans le projet sont fouillés.

Deux conséquences pratiques :

1. **Une recherche sur un projet avec des dépendances installées est lente et bruyante.** L'agent apprend vite à filtrer par extension ou à viser un sous-dossier, mais la première recherche d'une conversation peut ratisser très large.
2. **L'agent voit les fichiers de configuration et d'environnement du projet.** C'est voulu, et c'est nécessaire : un agent aveugle à un fichier `.env` conclut qu'il n'existe pas, et part sur une fausse piste — il proposera de créer une configuration qui est déjà là. Tous les assistants de développement en ligne de commande fonctionnent ainsi. La protection ne vit pas dans ce que l'agent peut voir, mais dans le mode de permission, la portée d'accès disque et le choix du modèle.

Un contournement existe et est même suggéré à l'agent dans la documentation interne de l'outil : passer par une commande shell quand il faut une vue qui respecte les exclusions de Git.

### Les limites de résultats

| Outil | Résultats maximum |
|---|---|
| `grep` | **250 correspondances** |
| `glob` | **100 fichiers** |

Au-delà, la recherche **s'arrête** et le résultat porte une mention de troncature. Elle ne continue pas en silence.

S'ajoute une seconde limite, plus loin dans la chaîne : le résultat transmis au modèle est plafonné à **10 000 caractères** pour `grep` et **5 000** pour `glob`. Un résultat plus long est réduit à un aperçu, le texte complet étant écrit sur le disque et relisible par l'agent.

Quand aucun fichier ne correspond, le résultat dit « (aucun résultat) » — jamais une réponse vide.

### Quand une recherche est trop longue

- Une recherche est abandonnée au bout de **600 secondes**, soit dix minutes. Elle renvoie alors une erreur de délai dépassé, signalée comme **réessayable**.
- Ce délai est très large exprès : il est là pour éviter qu'une recherche partie sur un disque entier bloque la conversation indéfiniment, pas pour cadencer les recherches ordinaires. Une recherche normale prend moins d'une seconde.
- L'arrêt est propre : le travail en cours est interrompu, il ne continue pas en tâche de fond.

### Ce qui échappe à ces outils

Quand certains fichiers ou dossiers n'ont pas pu être lus — droits insuffisants, lien cassé, fichier en cours d'écriture — le résultat est marqué comme **partiel** et **donne le nombre** d'éléments ignorés. Il ne prétend jamais avoir tout fouillé.

Si **aucun** fichier n'a pu être lu, c'est une erreur franche, pas un résultat vide. La distinction compte : « je n'ai rien trouvé » et « je n'ai rien pu lire » ne mènent pas aux mêmes conclusions.

---

## Tableaux

### Comparaison des deux outils

| | `grep` | `glob` |
|---|---|---|
| Cherche dans | Le contenu des fichiers | Le nom des fichiers |
| Syntaxe | Expression régulière (style Rust) | Motif de fichier (`*`, `**`, `?`, `[]`, `{}`) |
| Résultat | `chemin:ligne:contenu` | Chemins absolus |
| Sensible à la casse | **Oui**, toujours | Selon le motif |
| Filtre supplémentaire | Filtre par nom de fichier | Aucun |
| Résultats maximum | **250** | **100** |
| Longueur du motif | **500 caractères** | Non limitée explicitement |
| Ordre | Ordre de parcours | **Non garanti** |
| Renvoie des dossiers | Non | Non |

### Les erreurs

| Message | Cause | Réessayable |
|---|---|---|
| Motif d'expression régulière invalide | Syntaxe non supportée (référence arrière, anticipation) | Non |
| Motif trop long (max 500 caractères) | Motif trop long | Non |
| Filtre de fichiers invalide | Motif de nom mal formé | Non |
| Motif de fichiers invalide | Idem, pour `glob` | Non |
| Fichier introuvable | Le dossier de départ n'existe pas | Non |
| Permission refusée | Droits insuffisants sur le dossier de départ | Non |
| Chemin de recherche invalide | Chemin hors des zones autorisées | Non |
| Aucun fichier lisible ; N erreur(s) de lecture | Rien n'a pu être ouvert | **Oui** |
| Délai dépassé après 600 s | Recherche trop vaste | **Oui** |
| Le moteur de recherche interne s'est interrompu | Défaillance interne | **Oui** |

---

## Encadrés

> **Les exclusions de Git ne sont pas respectées.**
> Fichiers cachés, dossiers de dépendances, artefacts de compilation : tout est fouillé. C'est délibéré — l'agent doit pouvoir trouver un fichier de configuration ignoré par Git, sans quoi il conclurait qu'il n'existe pas. En contrepartie, une recherche large est lente sur un projet avec ses dépendances installées.

> **`grep` distingue les majuscules des minuscules.**
> Toujours, sans option pour le désactiver. Une recherche de `Config` ne trouve pas `config`. L'agent le sait et adapte son motif ; l'utilisateur qui lit la commande doit le savoir aussi.

> **Une recherche tronquée le dit.**
> À 250 correspondances ou 100 fichiers, la recherche s'arrête et l'annonce. Un résultat qui semble complet peut donc ne pas l'être : c'est écrit dans le résultat, et l'agent affine son motif quand il le voit.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent ne trouve pas un mot que je vois dans mon fichier » | Recherche sensible à la casse, ou expression régulière dont les caractères spéciaux ne sont pas échappés | Comportement attendu ; l'agent reformule seul en général |
| « La recherche remonte des résultats dans `node_modules` » | Les exclusions de Git ne sont pas appliquées | Demander à l'agent de viser un sous-dossier ou de filtrer par extension |
| « L'agent dit avoir trouvé 250 résultats, pile » | Limite atteinte, résultat tronqué | Restreindre la recherche |
| « Une recherche a pris dix minutes puis a échoué » | Dossier de départ trop vaste | Restreindre le dossier de départ ; vérifier la portée d'accès disque |
| « Les fichiers ne sortent pas dans le même ordre d'une fois sur l'autre » | L'ordre n'est pas garanti | Comportement attendu |

---

## Renvois

- `05-outils/fichiers.md` — lire ce qui a été trouvé, et la règle « lire avant d'écrire »
- `05-outils/terminal-et-shell.md` — l'alternative pour une recherche qui respecte les exclusions de Git
- `04-agent/repertoire-de-travail.md` — la portée de la recherche
- `04-agent/permissions.md` — la protection des fichiers sensibles

---

## Points à confirmer

- Le délai de **600 secondes** n'est pas dérivé d'un budget global : c'est une valeur locale fixe. À vérifier si l'application impose par ailleurs un budget de temps par tour d'agent, auquel cas les deux pourraient se contredire.
- Affichage à vérifier lors de la passe d'interface : présentation d'un résultat de recherche dans la conversation, et repli quand il compte 250 lignes.
- La description interne de `grep` mentionne un comportement « identique à ripgrep » ; l'implémentation utilise bien les mêmes bibliothèques, mais **les options de ripgrep ne sont pas exposées**. La formulation prête à confusion pour un lecteur technique. Sans conséquence pour le site, à signaler à l'équipe.
