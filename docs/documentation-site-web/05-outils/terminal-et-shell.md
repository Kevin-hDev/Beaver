# Terminal et shell — `bash` et `bash_control`

**Emplacement site** — Outils › Terminal et shell
**Répond à** — « Comment l'agent lance-t-il des commandes sur ma machine, et qu'est-ce qui l'en empêche ? »
**Sources** — `tool_bash.rs`, `tool_bash_shell.rs`, `tool_bash_session.rs`, `tool_bash_registry.rs`, `tool_bash_output.rs`, `tool_bash_result.rs`, `tool_bash_platform.rs`, `tool_bash_profile.rs`, `tool_bash_changes.rs`, `security.rs`, `tool_validate.rs`, `shell_sandbox/` (`launch.rs`, `scope.rs`, `macos.rs`, `linux.rs`, `windows.rs`), `directory_access_scope.rs`, `commands/directory_access.rs`, `tool_definitions_core.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Ce que fait l'outil
2. Quel shell est utilisé
3. Votre environnement est chargé
4. Les commandes longues : sessions et reprise
5. Piloter une session en cours
6. Ce qui est bloqué d'office
7. Le bac à sable du shell
8. Ce que l'agent voit de la sortie
9. Les fichiers modifiés sont suivis
10. Arrêter une commande

---

## Contenu

### Ce que fait l'outil

- `bash` exécute **une commande shell sur la machine de l'utilisateur**, avec ses droits d'utilisateur. C'est l'outil le plus puissant de Beaver : tout ce que l'utilisateur peut taper dans un terminal, l'agent peut le lancer.
- Il fait partie du groupe **Terminal**, qui est **verrouillé** : il ne peut pas être désactivé dans les réglages.
- Les commandes démarrent dans le répertoire de travail de la conversation. L'agent peut viser un autre dossier pour un appel précis, à condition de donner un chemin absolu — et **ce choix ne persiste pas** : l'appel suivant repart du répertoire de travail. De même, un `cd` à l'intérieur d'une commande ne survit pas à cette commande.
- Le répertoire visé doit rester dans les dossiers autorisés. Sinon la commande est refusée avant d'être lancée.

### Quel shell est utilisé

**Sur macOS et Linux**, Beaver prend le premier shell disponible dans cet ordre :

1. la variable `SHELL` de l'utilisateur ;
2. `/bin/zsh` (macOS uniquement) ;
3. `/bin/bash` ;
4. `/bin/sh`.

Le candidat doit être un chemin absolu, exister, et faire partie des shells reconnus : **zsh, bash, sh, dash, ksh**. Un shell exotique (fish, nushell, elvish…) est ignoré et Beaver descend au suivant de la liste. Si aucun ne convient, l'outil renvoie « Shell utilisateur indisponible ».

**Sur Windows**, c'est toujours **PowerShell**, pris à son emplacement système (`System32\WindowsPowerShell\v1.0\powershell.exe`), lancé sans bannière, sans profil et en mode non interactif. Le chemin n'est jamais deviné à partir du `PATH`.

> À écrire clairement sur le site : sur Windows, les commandes sont du **PowerShell**, pas du bash. `ls`, `cat` ou `grep` y ont un autre sens ou n'existent pas. L'agent le sait, mais l'utilisateur qui lit la commande doit le savoir aussi.

### Votre environnement est chargé

C'est le point qui distingue Beaver d'un simple `sh -c` et qui mérite une explication sur le site.

- Au premier appel de commande d'une conversation, Beaver **capture une photographie de l'environnement shell de l'utilisateur** : ses alias, ses fonctions, ses variables exportées, ses options de shell.
- Cette photographie est prise en lançant le shell en mode connexion et en lisant `.zshrc` ou `.bashrc` selon le shell.
- Elle est ensuite **rejouée au début de chaque commande** de la conversation. Résultat concret : les gestionnaires de version (nvm, pyenv, rbenv, asdf, mise, volta, bun) et les outils installés dans un dossier personnel fonctionnent, alors qu'ils seraient introuvables sans cette capture.
- La photographie est **mise en cache par conversation**, jusqu'à **64 conversations**. Elle n'est prise qu'une fois.
- Elle est plafonnée à **128 Ko** et abandonnée si elle dépasse **5 secondes**. En cas d'échec, les commandes tournent quand même, mais sans l'environnement personnalisé — c'est la cause la plus fréquente d'un « commande introuvable » alors qu'elle marche dans le terminal de l'utilisateur.
- Le contenu capturé est manipulé en mémoire protégée et effacé après usage : il peut contenir des jetons exportés dans un fichier de configuration.

### Les commandes longues : sessions et reprise

Beaver ne bloque pas sur une commande qui dure.

- Après un délai d'attente — **10 secondes par défaut**, réglable entre **250 ms et 30 secondes** — si la commande tourne encore, l'outil rend la main immédiatement avec ce qui a été produit jusque-là et un **identifiant de session**.
- L'agent poursuit son raisonnement, puis revient sur la session avec `bash_control` quand il le souhaite.
- **Il n'y a aucun délai maximal imposé.** Une commande peut tourner indéfiniment tant que personne ne l'arrête. Un délai maximal n'existe que si l'agent en fixe un explicitement pour cet appel.
- Les processus lancés en tâche de fond avec `&` restent rattachés à la session tant qu'ils n'ont pas fini.
- **64 sessions shell** peuvent coexister. Au-delà, Beaver récupère la place des sessions terminées ; si toutes sont actives, la nouvelle commande est refusée avec « Trop de processus shell actifs ».
- Une session terminée renvoie son code de sortie final puis disparaît du registre.

Pendant qu'une commande tourne, sa sortie **s'affiche en direct** dans la conversation, rafraîchie au fil de l'eau.

### Piloter une session en cours

`bash_control` sert à quatre choses, sur une session identifiée :

| Action | Ce qui se passe |
|---|---|
| Interroger | Rend la sortie produite depuis le dernier passage, et dit si le processus tourne toujours |
| Écrire une entrée | Envoie du texte sur l'entrée du processus — répondre à une invite, saisir un mot de passe, valider un choix |
| Fermer l'entrée | Signale au processus qu'il n'y aura plus rien à lire |
| Arrêter | Termine le processus **et tous ses enfants** |

Détails de comportement à connaître :

- Arrêter est exclusif : la demande d'arrêt ne peut pas être combinée avec l'envoi d'un texte ou la fermeture de l'entrée. L'appel est rejeté.
- Envoyer un caractère d'interruption (l'équivalent de Ctrl-C) dans le texte revient à demander l'arrêt.
- Une entrée est plafonnée à **64 Ko** et son écriture abandonne après **5 secondes** si le processus ne lit rien.
- Une session n'est accessible qu'à la conversation qui l'a créée. Une autre conversation reçoit « Session shell introuvable », sans autre détail.

### Ce qui est bloqué d'office

Certaines commandes sont refusées avant tout lancement, quel que soit le mode de permission, y compris en accès complet. La liste est courte et vise les destructions irréversibles à l'échelle du système, pas la suppression de fichiers ordinaire.

La vérification s'applique aussi au texte envoyé à une session en cours : on ne contourne pas le filtre en tapant la commande dans un shell déjà ouvert.

Quand une commande est bloquée, l'agent reçoit un message qui **nomme le motif détecté**. Il n'y a pas d'écran de confirmation : c'est un refus, pas une demande d'autorisation.

> Ce filtre est un garde-fou contre l'accident, pas une barrière de sécurité. Il reconnaît des formulations connues ; il ne comprend pas l'intention. La protection réelle vient du mode de permission et de la portée d'accès disque.

### Le bac à sable du shell

**C'est le point le plus important de la page, et le moins évident.**

Beaver sait exécuter les commandes dans un bac à sable du système d'exploitation, qui limite réellement ce que le processus peut lire et écrire — pas au niveau de Beaver, mais au niveau du noyau. Chaque plateforme utilise son mécanisme natif :

| Plateforme | Mécanisme |
|---|---|
| macOS | Seatbelt (`sandbox-exec`), avec une politique générée pour chaque commande |
| Linux | Landlock (ABI v3) et espaces de noms |
| Windows | Profil dédié et listes de contrôle d'accès |

**Ce bac à sable ne s'active que si l'accès disque est restreint.** Tant que les dossiers autorisés incluent la racine du disque — **ce qui est le réglage par défaut** — la commande est lancée directement, sans isolation : il n'y aurait rien à restreindre.

Autrement dit, **limiter les dossiers autorisés dans les réglages ne fait pas que filtrer les chemins côté Beaver : cela enferme réellement chaque commande shell dans une cage du système d'exploitation.** C'est la conséquence la plus concrète de ce réglage, et le site doit la présenter comme telle.

Quand le bac à sable est actif :

- l'écriture est limitée aux dossiers autorisés, plus un dossier temporaire propre à la commande ;
- la lecture reste ouverte sur les dossiers systèmes nécessaires à l'exécution des programmes, et sur les emplacements des gestionnaires d'outils ;
- si le mécanisme d'isolation est indisponible sur la machine, **la commande échoue** avec « Isolation du shell indisponible » — elle ne retombe pas silencieusement en mode non isolé ;
- un avertissement peut accompagner le résultat quand une limite interne a été atteinte pendant la construction de la politique.

Les dossiers temporaires du bac à sable vivent dans les données de l'application et sont nettoyés à la fin de chaque commande, ainsi qu'au démarrage suivant pour ceux qu'une interruption aurait laissés.

**Une commande en cours n'échappe pas à un changement de réglage** : si l'utilisateur restreint l'accès disque pendant qu'une commande tourne, toutes les sessions shell actives sont arrêtées et les requêtes en cours annulées. Élargir l'accès n'arrête rien.

### Ce que l'agent voit de la sortie

Trois plafonds se succèdent, du plus proche du processus au plus proche du modèle. Le site n'a pas besoin de tous les détailler, mais la page de référence doit les avoir.

1. **Ce que Beaver garde du processus** — 1 Mo, en conservant **le début et la fin** et en supprimant le milieu, avec une mention du nombre d'octets omis. Une commande très bavarde ne fait donc jamais gonfler la mémoire.
2. **Ce qui est remonté à chaque passage** — environ 28 Ko, répartis entre la sortie normale et la sortie d'erreur.
3. **Ce que reçoit le modèle** — au plus 2 000 lignes ou 50 Ko pour une commande, puis le plafond général de **30 000 caractères** pour les résultats de `bash`.

Quand une sortie est tronquée, le **résultat complet est écrit sur le disque** et son chemin est donné à l'agent, qui peut le relire s'il en a besoin.

Autres comportements visibles :

- une commande qui réussit sans rien afficher renvoie « Commande terminée en N ms (code 0) » plutôt qu'un résultat vide ;
- une commande encore en cours affiche son identifiant de session, son identifiant de processus et son temps écoulé ;
- un arrêt, une annulation ou un délai dépassé sont signalés explicitement, avec un code de sortie de `-1`.

Les tampons de sortie sont effacés de la mémoire après usage : une commande peut afficher une clé.

### Les fichiers modifiés sont suivis

Pendant qu'une commande tourne, Beaver observe le répertoire de travail et rend la liste des fichiers créés, modifiés ou supprimés.

- Le point de comparaison est l'état Git du dépôt quand il y en a un, sinon un instantané du dossier.
- Jusqu'à **500 chemins** sont suivis. Au-delà, la liste est signalée comme incomplète plutôt que d'être présentée comme exhaustive.
- Quand le suivi n'a pas pu démarrer ou a manqué des événements, le résultat le dit — il n'affirme jamais « aucun changement » par défaut.
- Ces changements alimentent l'affichage des différences dans la conversation.

### Arrêter une commande

- **Depuis l'agent** : `bash_control` avec la demande d'arrêt.
- **Depuis l'interface** : arrêter la réponse en cours annule les commandes lancées pour cette réponse.
- **À la fermeture de Beaver** : toutes les sessions shell sont annulées, avec **300 ms** de grâce, puis terminées de force.

L'arrêt vise **tout l'arbre de processus**, pas seulement la commande lancée : un script qui a démarré trois enfants ne laisse pas d'orphelins. Sur macOS et Linux, une demande d'arrêt propre est envoyée au groupe de processus, puis une terminaison forcée **50 ms** plus tard. Sur Windows, l'arbre est terminé d'un coup.

---

## Tableaux

### Les paramètres de `bash`

| Paramètre | Obligatoire | Valeur |
|---|---|---|
| Commande | Oui | Jusqu'à **512 Ko** (macOS, Linux) ou **24 Ko** (Windows) |
| Délai maximal | Non | En secondes ; **aucun délai** si absent ; zéro est refusé |
| Temps d'attente avant de rendre la main | Non | **250 ms à 30 s**, défaut **10 s** |
| Répertoire de travail | Non | Chemin **absolu** existant, valable pour cet appel seulement, obligatoirement dans les dossiers autorisés |

### Les paramètres de `bash_control`

| Paramètre | Obligatoire | Valeur |
|---|---|---|
| Identifiant de session | Oui | Renvoyé par `bash` |
| Texte à envoyer | Non | Jusqu'à **64 Ko** |
| Fermer l'entrée | Non | Oui / non |
| Arrêter | Non | Oui / non — **incompatible** avec les deux paramètres précédents |
| Temps d'attente | Non | **250 ms à 30 s**, défaut **10 s** |

### Ce qui est refusé avant exécution

| Catégorie | Exemples de motifs reconnus |
|---|---|
| Suppression avec élévation de privilèges | `sudo rm` |
| Permissions grandes ouvertes | `chmod 777` |
| Écriture directe sur un disque | `dd if=`, écriture sur `/dev/sd`, `dd` vers un périphérique |
| Formatage | `mkfs.`, `mkfs `, `fdisk`, `format c:`, `format d:` |
| Extinction et redémarrage | `shutdown`, `reboot`, `init 0`, `init 6` |
| Bombe de processus | La forme classique en shell |
| Suppression massive Windows | `del /f /s /q`, `rd /s /q` |
| Suppression en masse déguisée | `find … -delete`, `rsync … --delete` |
| Exécution indirecte | `eval "$…"` |

La reconnaissance ignore la casse et les espaces multiples.

### Les limites chiffrées

| Limite | Valeur |
|---|---|
| Sessions shell simultanées | **64** |
| Taille d'une commande | **512 Ko** Unix, **24 Ko** Windows |
| Taille d'une entrée envoyée | **64 Ko** |
| Sortie conservée par session | **1 Mo** (début + fin) |
| Sortie rendue par passage | **~28 Ko** |
| Sortie transmise au modèle | **2 000 lignes** ou **50 Ko**, puis **30 000 caractères** |
| Fichiers modifiés suivis | **500 chemins** |
| Profils d'environnement en cache | **64** conversations |
| Grâce avant terminaison forcée à la fermeture | **300 ms** |

---

## Encadrés

> **`bash` ne peut pas être désactivé.**
> Le groupe Terminal est verrouillé. Pour restreindre ce que l'agent peut faire sur la machine, les deux leviers sont le **mode de permission** et la **portée d'accès disque** — pas la désactivation de l'outil.

> **Restreindre les dossiers autorisés active une vraie isolation.**
> Tant que l'accès couvre tout le disque, les commandes s'exécutent sans bac à sable. Dès que la portée est réduite, chaque commande est enfermée par le système d'exploitation lui-même — Seatbelt sur macOS, Landlock sur Linux, un profil restreint sur Windows. C'est le seul réglage de Beaver dont l'effet est garanti par le noyau et non par le code de l'application.

> **Aucune commande n'a de durée maximale par défaut.**
> Une commande qui ne rend jamais la main continue de tourner jusqu'à ce qu'elle soit arrêtée ou que Beaver ferme. C'est voulu — une compilation ou un serveur de développement ne doivent pas être coupés à mi-parcours — mais l'utilisateur doit savoir qu'un processus peut survivre longtemps à la question qui l'a lancé.

> **Sur Windows, c'est PowerShell.**
> Les commandes ne sont pas du bash. Ce point mérite une mention dans la page d'installation Windows aussi.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Commande introuvable » alors qu'elle marche dans mon terminal | La capture de l'environnement a échoué ou dépassé 5 secondes, souvent à cause d'un fichier de configuration shell lent | Alléger le fichier de configuration, ou demander à l'agent d'utiliser le chemin complet du programme |
| Mon shell habituel n'est pas utilisé | Seuls zsh, bash, sh, dash et ksh sont reconnus | Comportement attendu ; les commandes restent exécutées par un shell compatible |
| « Isolation du shell indisponible » | Le mécanisme d'isolation du système est absent ou inutilisable | Vérifier la version du système ; sur Linux, un noyau sans Landlock. Élargir la portée d'accès contourne le problème, au prix de la protection |
| « Trop de processus shell actifs » | 64 sessions ouvertes et aucune terminée | Demander à l'agent d'arrêter ses commandes en cours |
| « Session shell introuvable » | La session appartient à une autre conversation, ou elle est déjà terminée | Comportement attendu |
| Mes commandes se sont toutes arrêtées d'un coup | La portée d'accès disque a été réduite pendant leur exécution | Comportement attendu et volontaire |
| Une commande a modifié des fichiers hors du dossier de travail | Le bac à sable n'était pas actif (accès disque complet) | Restreindre la portée d'accès, ou passer en mode Demande d'approbation |
| La sortie de ma commande est coupée en son milieu | Sortie supérieure à 1 Mo : début et fin conservés | Comportement attendu ; le fichier complet est sur le disque quand la troncature a lieu au niveau du résultat |

---

## Renvois

- `04-agent/permissions.md` — les commandes considérées comme sûres et exécutées sans demander
- `04-agent/repertoire-de-travail.md` — la portée d'accès disque et son effet
- `03-interface/terminal-integre.md` — le terminal manuel de l'utilisateur, distinct de cet outil
- `11-securite/acces-fichiers.md` — le modèle d'accès complet
- `11-securite/durcissement.md` — le bac à sable dans la vue d'ensemble sécurité
- `12-reference/limites-et-quotas.md`

---

## Points à confirmer

- **Le terminal intégré de l'interface et l'outil `bash` de l'agent sont deux choses différentes** — l'un est un pseudo-terminal pour l'utilisateur (16 onglets maximum), l'autre un mécanisme de sessions pour l'agent (64 sessions). Ils ne partagent ni les limites, ni le code, ni les processus. Le site doit les distinguer clairement, la confusion est certaine sinon. Décider s'ils vivent sur deux pages ou sur une seule avec deux sections.
- **La capture de l'environnement lit `.zshrc` ou `.bashrc`**, mais le shell est lancé en mode connexion, ce qui exécute aussi `.zprofile` ou `.bash_profile`. L'effet exact pour un utilisateur qui déclare ses variables dans `.zprofile` demande un essai réel avant d'écrire quoi que ce soit de définitif sur le site.
- **Le comportement du bac à sable sur Windows n'a pas été lu en détail** (profil dédié, listes de contrôle d'accès). Le tableau des mécanismes est juste, la description est volontairement sommaire. À compléter avant publication si le site veut détailler.
- Je n'ai **pas vérifié à l'écran** comment se présente une commande en cours dans la conversation : sortie en direct, bouton d'arrêt, affichage des fichiers modifiés. Description issue du code uniquement.
- La liste des motifs bloqués mérite d'être **rediscutée avec l'équipe produit** avant publication : la publier intégralement documente aussi ce qui n'est **pas** bloqué. Recommandation : donner les catégories et deux ou trois exemples, pas la liste exhaustive.
