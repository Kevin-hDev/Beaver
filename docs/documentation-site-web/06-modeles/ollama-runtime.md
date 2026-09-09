# Le moteur local — comment Beaver fait tourner Ollama

**Emplacement site** — Modèles › Moteur local
**Répond à** — « Beaver installe-t-il Ollama ? Et si j'ai déjà Ollama sur ma machine ? »
**Sources** — `services/ollama_manager/` : `manager_process.rs` (démarrage et détection), `manager_stop.rs` (arrêt), `port.rs` (attribution et sonde du port), `constants.rs` (les délais), `spawn_profile.rs` et `spawn_profile_paths.rs` (dossier des modèles, exécutable), `spawn_settings.rs` et `compute_mode.rs` (les variables réellement posées), `spawn_gate_unix.rs` / `spawn_gate_windows.rs` (le lancement lui-même), `process_receipt.rs` et `process_receipt_recovery.rs` (le reçu de processus et la reprise), `startup_recovery.rs` (le démarrage), `polling.rs` (la surveillance), `release_source.rs` (l'origine du téléchargement), `ollama_tree_job.rs` (arbre de processus Windows). Plus `services/paths/ollama.rs`, `services/gpu_detect.rs`, `services/gpu_vram.rs`, `models/config.rs`, `services/agent_local/agent_loop_support.rs`
**Vérification** — Vérifié dans le code. **Le module a été entièrement réécrit** ; ce brief a été refait de zéro le 9 septembre 2026 à partir du code, sans reprendre la version précédente

---

## Plan de page proposé

1. Ce que Beaver installe
2. Si vous avez déjà Ollama
3. Le port choisi
4. Les réglages appliqués au moteur
5. Où vivent les modèles
6. La taille de contexte s'adapte au matériel
7. Le démarrage et la reprise après panne
8. L'arrêt et le nettoyage
9. Les traces

---

## Contenu

### Ce que Beaver installe

Beaver embarque son propre moteur Ollama, téléchargé dans son dossier de données, sous `~/.local/share/cl-go-dash/ollama-bundle/` — le même chemin sur les trois systèmes.

Deux conséquences à énoncer clairement :

- **Aucune installation système n'est nécessaire.** L'utilisateur n'a pas à installer Ollama séparément, ni à connaître son existence.
- **Le moteur de Beaver n'écrase pas une installation existante.** Le programme du moteur vit dans le dossier de Beaver, avec ses propres réglages. Ses **modèles**, en revanche, sont bien ceux d'Ollama — voir plus bas.

Le téléchargement ne peut venir que **d'une seule adresse** : les publications officielles d'Ollama sur `github.com`, en `https` uniquement. Toute autre origine — autre domaine, autre port, adresse portant des identifiants ou des paramètres — est refusée avant même la connexion. **L'archive téléchargée est ensuite vérifiée par son empreinte SHA-256** ; si elle ne correspond pas, elle est rejetée et rien n'est installé.

Le nom de l'archive attendue dépend de la plateforme :

| Plateforme | Archive attendue |
|---|---|
| macOS | `ollama-darwin.tgz` |
| Windows | `ollama-windows-amd64.zip` |
| Linux | `ollama-linux-amd64.tar.zst` |
| Linux avec carte AMD détectée | la même, plus `ollama-linux-amd64-rocm.tar.zst` |

Une fois installé, le moteur laisse un **reçu** dans son dossier : la version installée et l'empreinte de son exécutable. **À chaque démarrage, Beaver recalcule cette empreinte et la compare au reçu.** Si les deux ne correspondent pas, il refuse de lancer le moteur et passe en reprise plutôt que d'exécuter un binaire dont il ne sait rien.

Le binaire est cherché à deux endroits dans le dossier du moteur : dans un sous-dossier `bin/` d'abord, puis à la racine — cette seconde position n'existe que pour les installations héritées des versions 1.1.x. S'il est absent des deux, Beaver signale que le moteur n'est pas installé.

### Si vous avez déjà Ollama

C'est le comportement le plus utile de cette page, et il est invisible pour qui ne le cherche pas.

**Avant de lancer quoi que ce soit, Beaver regarde si un moteur Ollama répond déjà sur le port habituel — `127.0.0.1:11434`.** Si c'est le cas, il l'adopte et **ne lance pas de second moteur**.

Comment la vérification se fait, exactement — et c'est à formuler sans exagérer :

1. **À la détection**, Beaver se contente d'ouvrir une connexion sur le port habituel, avec un délai de **250 millisecondes**. Ce qui répond est considéré comme un moteur existant.
2. **Ensuite, la surveillance vérifie réellement.** Toutes les **2 secondes**, Beaver interroge l'adresse de version du moteur. Si elle ne répond pas, le moteur est marqué indisponible et la fonctionnalité disparaît de l'application plutôt que d'échouer plus tard.

Conséquences pratiques :

- Les modèles déjà téléchargés par l'application Ollama officielle sont **immédiatement disponibles** dans Beaver. Rien à retélécharger.
- Il n'y a **jamais deux moteurs en mémoire** — donc pas de modèle chargé deux fois, ni de mémoire vidéo occupée en double.
- **Les réglages de Beaver ne s'appliquent pas** à un moteur qu'il n'a pas lancé. Le moteur existant garde sa propre configuration. Ce point doit figurer sur le site : un utilisateur qui règle l'accélération matérielle ou le multi-modèle dans Beaver ne verra aucun effet si Beaver réutilise le moteur du système.
- Certaines opérations sont **réservées au moteur possédé** : créer un modèle personnalisé, par exemple, est refusé quand Beaver n'a pas lancé le moteur lui-même.

### Le port choisi

Beaver ne s'impose jamais sur le port habituel d'Ollama.

- **Le port du moteur lancé par Beaver est attribué par le système d'exploitation.** Beaver demande « un port libre, n'importe lequel », retient celui qu'on lui donne, et recommence au plus **3 fois** si le port obtenu ne convient pas. Il n'y a **aucune plage de ports fixée** — ni dans le code, ni dans la configuration.
- **Le port 11434 ne sert qu'à la détection** d'un moteur déjà en cours d'exécution. Le moteur lancé par Beaver ne l'utilise pas.

Le moteur n'écoute que sur `127.0.0.1`. **Il n'est joignable depuis aucun autre appareil du réseau.**

### Les réglages appliqués au moteur

Quand Beaver lance son propre moteur, il lui impose une configuration. **Elle tient en cinq valeurs** — le tableau complet est plus bas. Les points qui méritent une explication sur le site :

- **Les modèles distants d'Ollama sont désactivés.** Ollama propose d'exécuter certains modèles sur ses propres serveurs ; Beaver coupe cette possibilité. Quand une conversation utilise un modèle local dans Beaver, **elle est réellement locale** — rien ne part sur le réseau.
- **Un seul modèle est chargé en mémoire à la fois**, sauf si l'utilisateur active le multi-modèle dans les réglages avancés. Sur une machine ordinaire, deux modèles chargés simultanément saturent la mémoire vidéo.
- **Le mode processeur seul se force explicitement.** Par défaut, Beaver laisse Ollama détecter le matériel tout seul ; le réglage « processeur » court-circuite cette détection.

**Combien de temps un modèle reste en mémoire** après usage se règle aussi, mais pas de la même façon : ce n'est pas une consigne donnée au moteur au lancement, c'est une valeur **transmise à chaque requête**. Elle vaut **5 minutes** par défaut, et deux valeurs méritent d'être expliquées sur le site :

- **« indéfiniment »** — le modèle ne quitte plus la mémoire. Il répond instantanément à la question suivante, au prix de la mémoire occupée en permanence.
- **« aucune persistance »** — le modèle est déchargé, et Beaver demande en plus explicitement au moteur de libérer la mémoire à la fin du tour de conversation.

Comme cette valeur voyage avec chaque requête et non avec le moteur, **c'est le seul réglage de moteur qui fonctionne aussi quand Beaver réutilise un moteur système.**

### Où vivent les modèles

Le dossier des modèles est **celui d'Ollama, pas un dossier propre à Beaver** :

- si la variable d'environnement `OLLAMA_MODELS` est définie sur la machine, Beaver la respecte ;
- sinon, il utilise l'emplacement habituel d'Ollama, `~/.ollama/models`.

Conséquence à écrire sur le site : **les modèles sont partagés avec l'application Ollama officielle**, que Beaver lance son propre moteur ou réutilise celui du système. Rien n'est dupliqué — et une suppression vaut des deux côtés.

Beaver refuse de démarrer si ce dossier chevauche l'un de ses propres dossiers de travail : sans ce garde-fou, une mise à jour du moteur pourrait effacer des modèles.

### La taille de contexte s'adapte au matériel

Beaver mesure la mémoire disponible et en déduit la taille de contexte des modèles locaux :

| Mémoire détectée | Contexte accordé |
|---|---|
| **24 Go ou plus** | **32 768 jetons** |
| **12 Go à 24 Go** | **24 576 jetons** |
| Moins de 12 Go, ou indétectable | **8 192 jetons** |

Quand la mémoire ne peut pas être mesurée, Beaver prend la valeur la plus basse. C'est le bon réflexe : mieux vaut un contexte modeste qui fonctionne qu'un contexte ambitieux qui fait échouer le chargement.

Sur les Mac à puce Apple, la mémoire mesurée est la **mémoire unifiée** — celle que le processeur et la partie graphique se partagent. Un Mac de 16 Go entre donc dans le palier intermédiaire. **Sur un Mac à processeur Intel, la mémoire n'est pas mesurée du tout** : le contexte tombe au palier de 8 192 jetons.

La mesure est rafraîchie toutes les **10 secondes** en arrière-plan, ce qui alimente aussi l'indicateur de mémoire de l'application.

### Le démarrage et la reprise après panne

L'ordre des opérations, à décrire simplement sur le site — il explique pourquoi Beaver refuse parfois de démarrer le moteur au lieu de le faire mal :

1. **Une passe de reprise d'abord.** Si une installation ou une mise à jour du moteur a été interrompue, elle est reprise ou annulée avant toute autre chose. Tant qu'elle ne se termine pas, le moteur reste bloqué et Beaver le dit plutôt que de lancer un moteur à moitié installé.
2. **Le reçu de processus est relu.** Si un moteur lancé par une session précédente tourne encore, il est arrêté et récupéré ; s'il a disparu, le reçu est supprimé.
3. **La détection d'un moteur existant.**
4. **Le lancement**, si aucun moteur n'a été trouvé.
5. **La vérification.** Beaver accorde **10 secondes** au moteur pour se mettre à écouter **et** pour répondre la version attendue. **Si l'une des deux échoue, le processus est arrêté et le démarrage est déclaré en échec** — il n'est jamais publié comme disponible « au cas où ».

Le reçu de processus est le mécanisme qui empêche les moteurs orphelins de s'accumuler. Il est écrit **avant que le moteur ne soit autorisé à démarrer réellement** : le processus est créé puis retenu, bloqué, tant que Beaver n'a pas enregistré son reçu sur le disque. Il n'existe donc aucun instant où un moteur tourne sans que Beaver sache le reconnaître. Le reçu contient de quoi l'identifier sans ambiguïté : identifiant du processus, instant exact de son démarrage, portée système, et empreinte du moteur installé. **Ces quatre éléments doivent tous correspondre pour que Beaver arrête un processus.** Un identifiant de processus est réutilisé par le système : sans cette vérification, Beaver risquerait de tuer un programme sans rapport qui aurait hérité du même numéro.

### L'arrêt et le nettoyage

- À la fermeture de Beaver, le moteur qu'il a lancé est arrêté **avec tout son arbre de processus** — groupe de processus sous macOS et Linux, objet de travail système sous Windows.
- Un moteur **réutilisé n'est jamais arrêté** : Beaver ne coupe pas un processus qu'il n'a pas démarré.
- La fermeture réserve **3 secondes** à cette opération dans son budget global. Un moteur qui ne rend pas la main dans ce délai n'empêche pas l'application de se fermer.
- Sous Linux, le moteur lancé reçoit en plus une consigne du noyau : **si Beaver disparaît brutalement, le système tue le moteur avec lui.**

### Les traces

**Le moteur n'écrit aucun fichier de traces.** Sa sortie n'est redirigée nulle part : elle est jetée. Il n'existe plus de fichier `logs/ollama-sidecar.log` — c'était le cas d'une version précédente du module, ce ne l'est plus.

Ce qui reste disponible pour comprendre une panne, ce sont **les traces de Beaver lui-même** : chaque étape du moteur y est consignée avec un **code d'erreur nommé** — moteur absent, reprise nécessaire, stockage indisponible, démarrage échoué, arrêt échoué, conflit de dossier de modèles. C'est ce code qu'il faut relever pour un signalement, pas un extrait de journal du moteur.

**Ce point est à faire remonter à l'équipe** : sans la sortie du moteur, un modèle qui refuse de se charger ne laisse aucune trace exploitable côté utilisateur.

---

## Tableaux

### Les réglages imposés au moteur lancé par Beaver

**Cinq valeurs, et rien d'autre.** Ce tableau est exhaustif : le code n'en pose aucune autre.

| Réglage | Valeur | Pourquoi |
|---|---|---|
| Adresse d'écoute | `127.0.0.1:<port attribué par le système>` | Rien n'est exposé au réseau |
| Dossier des modèles | Le dossier Ollama de la machine, vérifié avant lancement | Modèles partagés, jamais dupliqués |
| Modèles distants d'Ollama | **Désactivés** | Le local reste local |
| Modèles chargés en même temps | **1**, ou libre si le multi-modèle est activé | Éviter la saturation mémoire |
| Bibliothèque de calcul | **Processeur** en mode processeur seul, **détection automatique** sinon | Machines sans carte graphique utilisable |

**Pour la revérification, pas pour le site** — les cinq lignes ci-dessus correspondent exactement à cinq variables d'environnement, et à rien d'autre : `OLLAMA_HOST` (`127.0.0.1:<port>`), `OLLAMA_MODELS`, `OLLAMA_NO_CLOUD` (`1`), `OLLAMA_MAX_LOADED_MODELS` (`1`, ou `0` si le multi-modèle est activé), `OLLAMA_LLM_LIBRARY` (`cpu` en mode processeur, valeur vide sinon). L'environnement transmis est par ailleurs borné : **256 entrées** au maximum, **256 unités** par nom, **8 192** par valeur, et un total de **65 536 octets** sous Unix ou **32 767 unités** sous Windows.

**Ne sont posés nulle part**, contrairement à ce qu'affirmait une version antérieure de ce brief : l'attention optimisée, la compression du cache, une limite de requêtes simultanées, un délai de chargement de dix minutes, l'accélération Vulkan sous Windows, et une réserve de mémoire vidéo pour le système. Aucune de ces valeurs n'existe dans le code.

### Ce qui se règle ailleurs

| Réglage | Où il agit | Valeur par défaut |
|---|---|---|
| Persistance d'un modèle en mémoire | Transmis à chaque requête | **5 minutes** |
| Taille de contexte | Transmise à chaque requête, déduite de la mémoire | **8 192 à 32 768 jetons** |

### Les délais du moteur

| Étape | Délai |
|---|---|
| Sonde de détection d'un moteur existant | **250 ms** |
| Attente qu'un moteur lancé soit prêt et réponde la bonne version | **10 secondes** |
| Intervalle de surveillance | **2 secondes** |
| Délai d'une vérification de santé | **2 secondes** |
| Réserve d'arrêt à la fermeture | **3 secondes** |
| Rafraîchissement de la mesure de mémoire | **10 secondes** |
| Tentatives d'obtention d'un port | **3** |

### La détection de la carte graphique

| Plateforme | Méthode | Fabricants reconnus |
|---|---|---|
| macOS | Interrogation du processeur | Apple |
| Linux | Lecture des identifiants matériels du système | AMD, NVIDIA, Intel |
| Windows | Interrogation du gestionnaire de périphériques | NVIDIA, AMD, Intel |

Le fabricant détecté sert aussi au téléchargement : une carte AMD sous Linux fait récupérer l'archive ROCm en plus de l'archive standard.

### La détection de la mémoire

| Plateforme | Source, par ordre de préférence |
|---|---|
| macOS (puce Apple) | Mémoire unifiée du système |
| macOS (processeur Intel) | **Aucune** — la mémoire n'est pas mesurée |
| Linux | Outil NVIDIA, puis informations mémoire du pilote graphique |
| Windows | Outil NVIDIA, puis registre système, puis compteurs de performance |

---

## Encadrés

> **Vous n'avez pas à installer Ollama.**
> Beaver télécharge son propre moteur. Aucune manipulation, aucun terminal.

> **Si vous avez déjà Ollama, Beaver le réutilise.**
> Vos modèles déjà téléchargés sont immédiatement disponibles, et il n'y a jamais deux moteurs en mémoire. En contrepartie, les réglages de moteur de Beaver ne s'appliquent pas à cette installation-là.

> **Un modèle local est vraiment local.**
> Beaver désactive les modèles distants proposés par Ollama. Quand une conversation utilise un modèle local, rien ne quitte la machine.

> **Le moteur n'est joignable que depuis votre machine.**
> Il écoute sur `127.0.0.1`, sur un port que le système lui attribue. Aucun autre appareil ne peut s'y connecter.

> **Vos modèles ne sont pas dupliqués.**
> Beaver utilise le dossier de modèles habituel d'Ollama. Un modèle téléchargé d'un côté est disponible de l'autre — et une suppression vaut des deux côtés.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le moteur n'est pas installé » | Téléchargement interrompu, ou dossier du moteur absent | Relancer l'installation depuis les réglages |
| « Le moteur refuse de démarrer après un plantage » | Une installation ou une mise à jour interrompue doit être reprise avant tout démarrage | Relancer l'application ; la reprise se fait au démarrage |
| « Mes réglages de moteur ne changent rien » | Beaver réutilise un moteur système | Quitter l'application Ollama officielle, puis relancer Beaver |
| « Je ne peux pas créer de modèle personnalisé » | L'opération demande un moteur lancé par Beaver | Quitter l'application Ollama officielle, puis relancer Beaver |
| « Le chargement du modèle échoue » | Mémoire insuffisante pour le modèle choisi | Modèle plus petit, ou version plus compressée |
| « Le premier message est très lent » | Chargement du modèle en mémoire | Normal ; augmenter la persistance en mémoire pour éviter le rechargement |
| « Ma carte graphique n'est pas utilisée » | Fabricant non reconnu, ou pilotes absents | Forcer le mode processeur si besoin ; il n'y a pas de journal du moteur à consulter |
| « Beaver a laissé un processus en arrière-plan » | Arrêt brutal | Il est reconnu par son reçu et arrêté au démarrage suivant |
| « Deux modèles saturent ma mémoire » | Multi-modèle activé | Le désactiver dans les réglages avancés |

---

## Renvois

- `06-modeles/ollama-modeles.md` — installer et supprimer des modèles
- `06-modeles/ollama-personnalisation.md` — les réglages par modèle
- `06-modeles/materiel-et-vram.md` — choisir une taille de modèle
- `02-installation/premier-lancement.md` — le téléchargement initial
- `10-reglages/modeles.md` — les réglages avancés du moteur
- `12-reference/journaux.md` — les traces de l'application
- `13-depannage/ollama.md`

---

## Points à confirmer

- **Le moment exact du téléchargement du moteur** — au tout premier lancement, à la fin de l'accueil guidé, ou à la première utilisation d'un modèle local — n'a pas été tracé jusqu'à l'interface. Le mécanisme d'installation est vérifié ; son déclencheur ne l'est pas. À recouper avec `02-installation/premier-lancement.md` pour que les deux pages ne divergent pas.
- **La taille et la durée du téléchargement** ne sont pas mesurables depuis le code. À relever à l'usage.
- **Le comportement quand un moteur système est réutilisé** mérite une décision produit : rien dans le code ne montre que Beaver l'affiche à l'utilisateur. Sans indication visible, personne ne peut comprendre pourquoi les réglages de moteur restent sans effet et pourquoi la création de modèle personnalisé est refusée. **Recommandation à l'équipe : le signaler dans l'interface.**
- **L'absence de fichier de traces du moteur** est vérifiée dans le code et devrait être un point d'attention produit, pas seulement de documentation. Aucune sortie du moteur n'est conservée sur aucune des trois plateformes.
- **Les libellés exacts des réglages avancés** — accélération matérielle, multi-modèle, persistance en mémoire — n'ont pas été relevés dans les fichiers de traduction. À compléter en écrivant `10-reglages/modeles.md`.
- Sur **Mac Intel**, la mémoire n'est pas mesurée et le contexte tombe au palier minimal. Le comportement du code est confirmé ; reste à confirmer qu'il est voulu.
- Affichage à vérifier lors de la passe d'interface : ce que montre l'indicateur d'état du moteur, et ce qui s'affiche pendant une reprise après panne.
