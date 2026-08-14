# Fichiers — `read_file`, `write_file`, `edit_file`, `list_dir`

**Emplacement site** — Outils › Fichiers
**Répond à** — « Comment l'agent lit et modifie mes fichiers, et qu'est-ce qui l'empêche d'en abîmer un ? »
**Sources** — `tool_files.rs`, `tool_file_write.rs`, `tool_list_dir.rs`, `tool_file_error.rs`, `write_guard.rs`, `tool_executor_helpers.rs`, `tool_workspace_notice.rs`, `security.rs`, `tool_definitions_core.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Les quatre outils
2. Lire un fichier
3. La règle « lire avant d'écrire »
4. Modifier un fichier existant
5. Créer ou remplacer un fichier
6. Parcourir un dossier
7. Ce qui est refusé
8. Quand l'agent sort du dossier de travail

---

## Contenu

### Les quatre outils

Ils forment le groupe **Fichiers**, qui est **verrouillé** : impossible à désactiver. Sans eux l'agent ne pourrait rien faire d'un projet.

| Outil | Rôle | Écrit sur le disque |
|---|---|---|
| `read_file` | Lit un fichier texte | Non |
| `edit_file` | Remplace un passage précis dans un fichier existant | **Oui** |
| `write_file` | Crée un fichier ou le remplace entièrement | **Oui** |
| `list_dir` | Affiche l'arborescence d'un dossier | Non |

Les deux outils qui écrivent demandent une approbation en mode **Demande d'approbation**. Les deux autres non.

### Lire un fichier

- `read_file` ne lit que du **texte encodé en UTF-8**. Une image, un PDF, un `.docx` ou un exécutable renvoient une erreur : ce ne sont pas des fichiers texte. Les outils dédiés (documents, tableurs, images) existent pour ça, dans des groupes optionnels à activer.
- Taille maximale : **20 Mo**. Au-delà, le fichier est refusé, pas tronqué.
- Le contenu revient **numéroté ligne par ligne**, ce qui permet à l'agent de désigner un emplacement précis et de citer un numéro de ligne dans sa réponse.
- Par défaut l'agent reçoit les **2 000 premières lignes**, et jusqu'à **50 000** s'il le demande. Quand il en reste, le résultat indique combien de lignes n'ont pas été lues et à partir d'où reprendre — l'agent enchaîne alors tout seul.
- Le chemin peut être relatif au répertoire de travail ou absolu. Dans les deux cas il doit tomber dans une zone autorisée en lecture.

### La règle « lire avant d'écrire »

C'est le garde-fou le plus utile de Beaver, et il mérite une explication sur le site parce qu'il se voit dans la conversation.

**L'agent ne peut pas modifier un fichier existant sans l'avoir vu d'abord.** S'il essaie, l'écriture est bloquée avec un message qui lui dit d'aller le lire.

Pourquoi c'est important : sans cette règle, un modèle qui « croit se souvenir » du contenu d'un fichier peut le réécrire à partir de sa mémoire et effacer tout ce qu'il ne savait pas. Avec la règle, il doit constater l'état réel avant d'y toucher.

Ce qui compte comme « avoir vu le fichier » :

- l'avoir lu avec `read_file`, ou avec les outils de lecture de documents, de tableurs et d'images ;
- l'avoir vu **apparaître dans un résultat** de recherche par motif, de recherche par nom ou de listing de dossier ;
- l'avoir écrit ou modifié soi-même auparavant dans la même conversation ;
- l'avoir modifié via une commande shell — les fichiers touchés par une commande sont enregistrés automatiquement.

Deux précisions :

- **La règle ne s'applique qu'aux fichiers existants.** Créer un fichier neuf ne demande rien.
- La liste des fichiers vus est **bornée à 1 000 chemins par conversation**. Au-delà, les 100 plus anciens sont oubliés — un fichier lu il y a très longtemps dans une conversation très longue peut donc redemander une lecture. C'est rare et sans gravité.

### Modifier un fichier existant

`edit_file` remplace **un passage exact par un autre**, une seule fois.

- Le passage cherché doit être **présent une seule fois** dans le fichier. S'il apparaît plusieurs fois, l'outil refuse et **dit combien de fois** il l'a trouvé, en suggérant d'ajouter des lignes autour pour lever l'ambiguïté. L'agent recommence avec plus de contexte.
- La correspondance est **exacte au caractère près** : espaces, tabulations et retours à la ligne compris.
- Il n'y a **pas de remplacement global**. Renommer un symbole présent vingt fois demande vingt appels, ou une réécriture complète du fichier.
- En cas de succès, l'outil renvoie le **numéro de la ligne modifiée**, ce qui alimente l'affichage des différences dans la conversation.

C'est l'outil à préférer pour toute modification : il ne transmet que la portion changée, là où une réécriture complète renvoie tout le fichier et consomme beaucoup plus de contexte.

### Créer ou remplacer un fichier

`write_file` écrit un contenu entier. Il sert à **créer** un fichier, ou à le **réécrire de bout en bout**.

- Les dossiers parents manquants sont créés automatiquement.
- L'écriture est refusée hors des zones autorisées en écriture — c'est-à-dire le répertoire de travail, les chemins configurés dans les réglages avancés, et les espaces de travail gérés par l'application.
- **Les liens symboliques ne sont jamais suivis.** Écrire sur un lien est refusé, et le refus est vérifié trois fois : avant la résolution du chemin, après, et au moment même de l'ouverture du fichier. C'est ce qui empêche qu'un lien placé dans le projet serve de passerelle pour écrire ailleurs sur le disque.
- Le dossier parent réel est revérifié après résolution : un chemin qui traverse un lien pour ressortir hors des zones autorisées est refusé.

### Parcourir un dossier

`list_dir` donne une **arborescence indentée**, pas une liste à plat.

- Descend jusqu'à **3 niveaux** de profondeur.
- Les dossiers sont suffixés d'une barre oblique. Tri alphabétique.
- **Aucune taille, aucune date, aucune permission** — juste des noms. Pour ces informations, l'agent passe par une commande shell.
- **Sont masqués** : tout ce qui commence par un point, ainsi que `node_modules` et `target`. Ces deux-là sont exclus parce qu'ils contiennent des dizaines de milliers d'entrées sans intérêt.
- Plafonné à **500 entrées**. Au-delà, le résultat le signale explicitement.
- Un dossier vide renvoie « (dossier vide) », pas un résultat vide.
- Quand certains sous-dossiers n'ont pas pu être ouverts, le résultat le dit et **donne leur nombre** — il ne prétend jamais avoir tout listé.

> À dire sur le site : les fichiers cachés étant masqués, un agent à qui on demande « lis mon `.env` » ne le trouvera pas en listant le dossier. Il faut lui donner le nom. Cela dit, les fichiers sensibles font l'objet d'une protection à part — voir la page sur les permissions.

### Quand l'agent sort du dossier de travail

Toute opération qui modifie un fichier **hors du répertoire de travail** ajoute un avertissement au résultat, qui rappelle à l'agent quel est l'espace de travail actif et lui demande d'y revenir sauf demande explicite de l'utilisateur.

Ce n'est **pas un blocage** : l'opération a eu lieu. C'est un rappel destiné au modèle, pour éviter la dérive silencieuse où une session finit par éparpiller des fichiers un peu partout.

---

## Tableaux

### Les limites

| Limite | Valeur |
|---|---|
| Taille maximale d'un fichier lu | **20 Mo** |
| Lignes par lecture, par défaut | **2 000** |
| Lignes par lecture, maximum | **50 000** |
| Profondeur de `list_dir` | **3 niveaux** |
| Entrées listées par `list_dir` | **500** |
| Fichiers mémorisés comme « vus » | **1 000** par conversation |

### Les erreurs et leur signification

| Message | Cause | Est-ce réessayable |
|---|---|---|
| Fichier introuvable | Le chemin n'existe pas | Non |
| Permission refusée | Le système d'exploitation refuse | Non |
| Lecture interdite hors des zones autorisées | Chemin en dehors de la portée d'accès | Non |
| Écriture interdite hors des zones autorisées | Idem, en écriture | Non |
| Fichier trop volumineux (max 20 Mo) | Dépassement de taille | Non |
| Le fichier n'est pas de l'UTF-8 | Fichier binaire ou autre encodage | Non |
| Le chemin est un dossier | Confusion fichier / dossier | Non |
| Écriture sur symlink interdite | La cible est un lien symbolique | Non |
| Chaîne non trouvée | Le passage cherché n'existe pas dans le fichier | Non |
| Chaîne trouvée N fois (doit être unique) | Passage ambigu | Non — l'agent doit reformuler |
| Écriture bloquée : fichier non lu avant modification | Règle « lire avant d'écrire » | Non — l'agent doit lire d'abord |
| Délai d'entrée-sortie dépassé | Disque lent, montage réseau | **Oui**, sauf en écriture |

> Détail qui compte, à garder pour la page sur les erreurs : une **écriture** interrompue par un délai dépassé n'est **jamais** annoncée comme réessayable, et le message ajoute que l'état du fichier peut être partiel. On ne rejoue pas à l'aveugle une écriture dont on ignore si elle a abouti.

### Ce que les messages d'erreur ne disent pas

Les erreurs système sont **réécrites** avant d'être remontées. L'agent et l'utilisateur voient « Fichier introuvable », « Permission refusée », « Le chemin est un dossier » ou « Erreur système ».

Ils ne voient jamais le message brut du système d'exploitation, qui révélerait des chemins internes, des noms d'utilisateur ou la structure du disque.

---

## Encadrés

> **L'agent ne réécrit pas un fichier de mémoire.**
> Il doit l'avoir consulté dans la conversation en cours avant de pouvoir le modifier. C'est ce qui empêche qu'il « reconstruise » un fichier depuis ce qu'il croit savoir et efface le reste au passage.

> **Les liens symboliques ne sont jamais suivis en écriture.**
> Un lien posé dans le projet ne peut pas servir de passage vers le reste du disque.

> **`edit_file` d'abord, `write_file` en dernier recours.**
> La modification ciblée est plus sûre — elle échoue si le fichier n'est pas dans l'état attendu — et beaucoup moins coûteuse en contexte. Une réécriture complète, elle, écrase sans discuter tout ce qui n'était pas dans le nouveau contenu.

> **`list_dir` ne montre pas tout.**
> Fichiers cachés, `node_modules` et `target` sont masqués, la profondeur s'arrête à trois niveaux et la liste à 500 entrées. Ce n'est pas un explorateur de fichiers : c'est un aperçu de structure.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent n'arrive pas à lire mon PDF » | `read_file` ne lit que du texte UTF-8 | Activer le groupe Document, ou convertir le fichier |
| « L'agent me dit qu'il doit d'abord lire le fichier » | Règle « lire avant d'écrire » | Comportement normal : il lit puis modifie, sans intervention |
| « L'agent s'y reprend à trois fois pour une modification » | Le passage cherché apparaissait plusieurs fois | Comportement normal : il ajoute du contexte jusqu'à lever l'ambiguïté |
| « Mon fichier de configuration n'apparaît pas dans la liste » | Les fichiers cachés sont masqués | Donner le nom du fichier à l'agent |
| « L'agent ne voit pas les fichiers de mon sous-dossier profond » | Profondeur limitée à 3 niveaux | Lui donner le chemin du sous-dossier, ou passer par une commande shell |
| « L'écriture est refusée alors que le dossier existe » | Chemin hors des zones autorisées, ou lien symbolique | Vérifier la portée d'accès dans les réglages |
| « Le fichier n'a été modifié qu'à moitié » | Écriture interrompue | L'agent est averti que l'état peut être partiel ; lui demander de relire le fichier |

---

## Renvois

- `04-agent/permissions.md` — quelles écritures demandent une approbation, et les fichiers protégés
- `04-agent/repertoire-de-travail.md` — la portée d'accès en lecture et en écriture
- `05-outils/recherche-fichiers.md` — trouver un fichier avant de le lire
- `05-outils/tableurs.md`, `05-outils/documents.md`, `05-outils/images.md` — les formats que `read_file` ne sait pas lire
- `11-securite/acces-fichiers.md`
- `12-reference/formats-supportes.md`

---

## Points à confirmer

- **La description interne de l'outil est plus stricte que le code.** Elle annonce que l'agent doit avoir appelé `read_file` sur le fichier, alors qu'un fichier simplement **apparu dans un résultat de recherche ou de listing** suffit à débloquer l'écriture. La page doit décrire le comportement réel — c'est ce que j'ai fait — mais l'écart mérite d'être remonté à l'équipe : soit la description est trop stricte, soit la garde est trop permissive. Un fichier repéré par une recherche n'a pas été *lu* : son contenu n'est pas passé sous les yeux du modèle.
- Je n'ai **pas vérifié à l'écran** comment s'affichent une lecture, une modification et l'aperçu des différences dans la conversation. La page décrit le mécanisme, pas la présentation.
- L'avertissement de sortie du dossier de travail est rédigé **en anglais** dans le code, alors qu'il peut apparaître dans une conversation en français. À vérifier : est-il visible par l'utilisateur ou seulement transmis au modèle ? Si l'utilisateur le voit, c'est un texte à traduire.
- Le comportement de `list_dir` sur le **dossier de données de l'application** est particulier : la profondeur tombe à zéro quand ce dossier n'est pas dans les zones autorisées. À décrire seulement si le site documente ce dossier.
