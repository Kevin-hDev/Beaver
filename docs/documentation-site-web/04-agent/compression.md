# La compression du contexte

**Emplacement site** — Agent › La compression du contexte
**Répond à** — « Ma conversation devient longue. Qu'est-ce que Beaver fait quand elle approche de la limite du modèle, qu'est-ce qu'il garde, qu'est-ce qu'il perd, et est-ce que je peux le régler ? »
**Sources** — `src-tauri/src/services/compress/` en entier, en particulier `orchestrator.rs`, `orchestrator_started.rs`, `orchestrator_summary.rs`, `orchestrator_sections.rs`, `automatic_guard.rs`, `checkpoint_document.rs`, `checkpoint_selection.rs`, `checkpoint_candidate.rs`, `checkpoint_target.rs`, `checkpoint_candidate_budget.rs`, `checkpoint_evidence.rs`, `checkpoint_transaction.rs`, `summary_contract.rs`, `summary_request.rs`, `prompt.rs`, `compression_redaction.rs`, `profile_types.rs`, `profile_defaults.rs`, `profile_limits.rs`, `profile_resolve.rs`, `profile_budget.rs`, `profile_store.rs`, `profile_store_document.rs`, `profile_validation.rs`, `realtime_budget.rs`, `token_estimate.rs`, `context_resolve.rs`, `session_capabilities.rs`, `metrics.rs`, `timeouts.rs`, `command.rs` ; `services/agent_local/compress_hook.rs`, `agent_loop_compression.rs` ; `services/llm/compress_hook.rs` ; `services/config_compression_migration.rs` ; `src-tauri/src/commands/agent_chat_task.rs`, `commands/agent_chat_task/compress.rs`, `commands/agent_chat_task/conversation.rs`, `commands/compression_profiles_undo.rs` ; `src-tauri/src/models/compression_profile_contract.rs` ; `src-tauri/src/invoke_handler.rs` ; côté interface `src/components/agent-local/compression-indicator.tsx`, `context-compression-marker.tsx`, `context-compression-help-popover.tsx`, `context-progress.tsx`, `chat-plus-compression-menu.tsx`, `src/components/settings/compression/*`, `src/hooks/use-compression-profiles.ts`, `use-session-compression-profile.ts`, `use-slash-commands.ts`, `agent-chat-stream-callbacks.ts`, `agent-stream-compression-complete.ts`, `src/lib/context-messages.ts`, `src/lib/agent-error-codes.ts`, `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, après la refonte livrée en **1.2.0** (`CHANGELOG.md:81-96`), sur la version courante **1.2.2** (`package.json:3`). Rien n'a été observé à l'écran : tous les points d'affichage sont listés en fin de fichier.

> **Cette page décrit un mécanisme automatique, pas une manipulation.** L'utilisateur n'a presque rien à faire : la compression se déclenche seule. La page doit donc répondre à une question de confiance — *qu'est-ce que Beaver garde de ma conversation ?* — avant de décrire les réglages. Les profils de compression et leur éditeur avancé sont la seconde moitié de la page, pas la première.

---

## Plan de page proposé

1. Le problème : la fenêtre de contexte est finie
2. Ce que fait la compression, en une phrase
3. Quand elle se déclenche
4. Ce que l'utilisateur voit
5. Ce qui est conservé, ce qui est résumé
6. Le résumé lui-même : neuf sections imposées
7. Ce que la conversation devient après
8. Les protections : boucle, échec, secrets
9. Les trois tailles de fenêtre, et le cas des petits modèles
10. Les profils de compression
11. La compression à la demande : `/compress`
12. Où tout cela est rangé

---

## Contenu

### 1. Le problème : la fenêtre de contexte est finie

Un modèle de langage ne lit pas une conversation, il relit **toute** la conversation à chaque réponse. Cette relecture a une taille maximale — la **fenêtre de contexte** — comptée en jetons. Une conversation longue finit par ne plus tenir dedans.

La fenêtre effective n'est pas la même pour tout le monde, et Beaver la détermine différemment selon le type de modèle (`services/compress/context_resolve.rs:44-50`) :

- **modèle distant** — la longueur déclarée par le catalogue de fournisseurs (`context_resolve.rs:35-42`, `:135-139`) ;
- **modèle local via Ollama** — la première valeur disponible dans cet ordre : la fenêtre du modèle déjà chargé, puis celle écrite dans son Modelfile (`num_ctx`), puis la plus petite entre la fenêtre native du modèle et le palier déduit du matériel (`context_resolve.rs:99-116`).

Autrement dit : sur un modèle local, la fenêtre réellement utilisée est souvent **plus petite** que celle annoncée par le modèle, parce que le matériel commande. C'est le point qui surprend le plus, et il mérite une phrase sur le site.

### 2. Ce que fait la compression, en une phrase

Beaver demande à un modèle de **résumer la conversation dans un format imposé**, range ce résumé avec quelques éléments récents dans un **point de reprise**, puis remplace l'historique par ce point de reprise. La conversation continue au même endroit, dans le même fil, avec beaucoup moins de jetons.

Deux propriétés à énoncer d'entrée :

- **la conversation enregistrée est réellement remplacée** — ce n'est pas un affichage replié : le fichier de session ne contient plus les anciens messages (`services/compress/checkpoint_transaction.rs:66`) ;
- **le remplacement est tout ou rien** — Beaver construit le point de reprise complet, le valide, vérifie qu'il tient dans la cible, et ne remplace la conversation qu'à la fin, en une seule écriture ; si quoi que ce soit échoue en chemin, **la conversation reste exactement telle qu'elle était** (`checkpoint_transaction.rs:51-86`).

### 3. Quand elle se déclenche

Deux déclencheurs, et un seul moteur derrière (`services/compress/profile_types.rs:17-20` ; `orchestrator.rs:28`).

**Automatique.** Beaver vérifie après chaque réponse et après chaque série d'outils si la conversation a atteint le **seuil** du profil actif (`services/agent_local/agent_loop_compression.rs:97-128` pour les modèles locaux, `services/agent_local/tool_executor_compression.rs:38` et `:60` pour les deux familles). La condition exacte est « jetons estimés ≥ fenêtre × seuil ÷ 100 » (`token_estimate.rs:73-79`).

**Le seuil du profil livré avec Beaver est de 90 %** (`profile_defaults.rs:13`). Il est réglable de **1 % à 90 %** (`profile_limits.rs:14-15`), et le moteur le replafonne à 90 % à l'exécution quoi qu'il arrive (`orchestrator.rs:170`).

Une troisième vérification existe, **pendant** la génération d'une réponse : Beaver surveille le total toutes les **32 jetons produits** et interrompt la réponse en cours dès que le seuil est franchi, plutôt que d'attendre la fin (`realtime_budget.rs:1`, `:48-54`). C'est ce qui explique qu'une réponse puisse s'arrêter net et être suivie d'une compression.

**À la demande.** L'utilisateur envoie le message `/compress`. La commande est reconnue seulement si le message ne contient **rien d'autre** — les espaces autour sont tolérés, un mot ajouté ne l'est pas (`services/compress/command.rs:1-5`, test `:10-13`). Elle court-circuite entièrement le tour de conversation habituel : ni journal de tour, ni appel au modèle pour répondre (`commands/agent_chat_task.rs:89-107` ; `commands/agent_chat_task/conversation.rs:73-79`).

**Quatre différences entre les deux déclencheurs**, toutes vérifiées :

| | Automatique | `/compress` |
|---|---|---|
| Respecte le seuil | Oui | **Non — s'exécute quel que soit le remplissage** (`orchestrator.rs:162-172`) |
| Respecte l'interrupteur « Compression automatique » | Oui (`orchestrator.rs:165`) | **Non** |
| Si un tour est resté ouvert | Renonce en silence (`orchestrator.rs:148`) | **Échoue avec un message** (`orchestrator.rs:149`) |
| Si la compression est indisponible sous 64K | Renonce en silence (`orchestrator.rs:50`) | **Échoue avec un message** (`orchestrator.rs:51`) |

C'est le point le plus utile de la section : **`/compress` n'est pas « la même chose, en avance ». C'est la version qui parle quand ça ne marche pas.**

### 4. Ce que l'utilisateur voit

**Pendant.** Un bandeau discret remplace l'indicateur de travail habituel, avec deux traits horizontaux qui pulsent de part et d'autre du texte **« Compression du contexte »** (`src/components/agent-local/compression-indicator.tsx:12-16` ; `src/i18n/fr.json:328` ; animation `compression-indicator.css:1-39`, désactivée si le système demande moins d'animations). Il n'affiche ni pourcentage ni durée, et n'est pas cliquable.

Le bouton d'envoi devient un bouton d'arrêt pendant l'opération, comme pour une réponse normale (`src/components/agent-local/chat-input.tsx:149-151`). Le champ de saisie, lui, reste éditable (`chat-input.tsx:175`).

**Après.** À l'endroit exact de la coupure, le fil affiche une icône d'archive suivie du texte **« Contexte compressée »** (`context-compression-marker.tsx:6-11` ; `fr.json:339`). Ce marqueur n'est **pas dépliable** : le résumé généré n'est consultable nulle part dans l'interface (voir « Anomalies relevées »).

**L'anneau de contexte**, près du champ de saisie, indique le profil de compression actif de la conversation sous le libellé **« Compression »**, ou **« Compression désactivée »** quand elle ne peut pas s'appliquer (`context-progress.tsx:189-210` ; `fr.json:439-440`). Dans ce second cas, un bouton d'aide ouvre une explication (`context-compression-help-popover.tsx:64-78`), reproduite au point 9.

**En cas d'échec**, cinq messages distincts selon la cause (`src/lib/agent-error-codes.ts:36-40` ; `fr.json:1426-1430`) — voir le tableau des pièges.

### 5. Ce qui est conservé, ce qui est résumé

C'est le cœur de la page. Tout ce qui n'est pas dans la liste ci-dessous **disparaît de la conversation** et ne survit que sous la forme du résumé.

**Ce qui est repris tel quel, message par message :**

| Élément | Comment il est choisi | Source |
|---|---|---|
| **Messages récents** | Un quota par rôle — la moitié arrondie au supérieur pour l'utilisateur, la moitié pour l'assistant — puis les emplacements restants sont comblés du plus récent au plus ancien | `checkpoint_selection.rs:66-93`, `:95-117` |
| **Chaînes d'outils** | Une chaîne appel + résultats est prise **entière ou pas du tout** ; un résultat trop long est remplacé par un extrait borné qui renvoie au résultat complet conservé sur disque | `checkpoint_selection.rs:119-153` ; `checkpoint_tools.rs:59-74` |
| **Le tour en cours** | Toujours conservé intégralement, hors quota | `checkpoint_selection.rs:39-42` |
| **Images** | Les images des messages retenus, dans la limite du profil et de celle du fournisseur | `orchestrator_started.rs:82-95` ; `orchestrator_support.rs:8-15` |

**Ce qui est reconstruit et rangé dans le point de reprise**, dans cet ordre et tant qu'il reste de la place (`orchestrator_sections.rs:13-49`) :

1. **l'état du travail** — état Git, plan et tâches en cours, échecs non résolus, rapports de sous-agents (`orchestrator_sections.rs:51-73`) ;
2. **les fichiers récents, relus depuis le disque** — c'est-à-dire leur contenu **actuel**, pas celui qu'avait vu l'agent (`checkpoint_files.rs:12-18`, champ `current_content` `:9`) ;
3. **les pièces jointes texte**, au plus 8 (`orchestrator_sections.rs:33-38`) ;
4. **les références critiques**, au plus 32 (`orchestrator_sections.rs:39-44`).

Le point 2 mérite une phrase explicite sur le site : **un fichier modifié entre-temps entre dans le point de reprise dans sa version d'aujourd'hui.** C'est voulu — c'est ce qui permet à l'agent de reprendre sur l'état réel — mais ce n'est pas ce qu'on suppose spontanément d'un « résumé ».

Enfin, le message `/compress` lui-même est exclu partout : il n'entre ni dans les messages conservés (`checkpoint_document.rs:91-95`), ni dans l'historique envoyé au modèle qui rédige le résumé (`summary_request.rs:49-56`).

### 6. Le résumé lui-même : neuf sections imposées

Beaver envoie la conversation à un modèle avec un **contrat système fixe** que rien ne peut modifier (`services/compress/prompt.rs:1-13`). Ce contrat impose exactement neuf sections, dans cet ordre (`summary_contract.rs:72-84`) :

1. Primary Request and Intent — la demande initiale et l'intention
2. Key Technical Concepts — les notions techniques en jeu
3. Files and Code Sections — les fichiers et portions de code
4. Errors and Fixes — les erreurs rencontrées et leurs corrections
5. Problem Solving — la résolution
6. User Intent and Corrections — les intentions et corrections de l'utilisateur
7. Pending Tasks — ce qui reste à faire
8. Current Work — le travail en cours
9. Next Step — l'action suivante

*(Les titres sont en anglais dans le résumé produit ; les traductions ci-dessus sont pour la page, pas pour l'application.)*

**Le résumé est vérifié avant d'être accepté**, et un seul défaut le fait rejeter (`summary_contract.rs:17-70`) : un appel d'outil, une réponse tronquée, une balise `<summary>` manquante ou mal fermée, du texte après la fermeture, un contenu vide, **une des neuf sections absente ou dans le mauvais ordre**, ou un dépassement du plafond de jetons du profil.

**Trois protections de sécurité valent d'être mentionnées sur le site**, parce qu'elles répondent à une inquiétude légitime :

- **l'historique est présenté au modèle comme des données non fiables**, enveloppé dans un marqueur explicite avec la consigne de ne jamais suivre les instructions qu'il contient (`summary_request.rs:69-71`) ;
- **le prompt personnalisé d'un profil ne peut pas primer sur le contrat** : il est introduit comme un objectif supplémentaire qui ne peut pas l'écraser (`summary_request.rs:65-68`, `:72-75`) ;
- **le contrat interdit explicitement de révéler des secrets et de mentionner les modes de permission** (`prompt.rs:2`).

Et la protection se double d'un filtrage mécanique : le résumé, les sections et les messages envoyés au modèle passent tous par la fonction de masquage des données sensibles de Beaver (`compression_redaction.rs:3-38`, appelée en `summary_contract.rs:58` et `checkpoint_document.rs:146`, `:188`).

### 7. Ce que la conversation devient après

Le point de reprise prend la forme de **deux messages techniques** portant le même identifiant de tour (`checkpoint_document.rs:140-169`) :

- un message d'utilisateur contenant le document JSON du point de reprise — numéro de format **1**, identifiant, résumé, et les sections rangées par nom (`checkpoint_document.rs:9`, `:27-33`, `:147-152`) ;
- un message d'assistant portant le texte fixe `[Compression boundary — previous messages have been summarized]` (`checkpoint_boundary.rs:1`).

Dans l'interface, le premier devient le marqueur « Contexte compressée » et le second est **entièrement masqué** (`src/lib/context-messages.ts:7-17`).

**La cible visée.** Beaver ne cherche pas à remplir un pourcentage de la fenêtre : il vise **20 % de ce qui est compressible**, c'est-à-dire tout sauf les instructions système et les définitions d'outils, plus ces dernières telles quelles (`checkpoint_target.rs:11-15`). Ce résultat est en plus plafonné selon la taille de la fenêtre : **10 000 jetons** sous 64K, **20 000** entre 64K et 128K, **28 000** au-delà (`checkpoint_target.rs:36-42`).

**Deux garde-fous à la sortie** (`checkpoint_candidate.rs:189-218`) :

- si le point de reprise construit dépasse la cible, la compression **échoue** au lieu de publier un résultat trop gros ;
- si la compression était automatique et que le résultat est **encore au-dessus du seuil**, elle échoue également — Beaver refuse de publier une compression qui n'aurait rien réglé.

### 8. Les protections : boucle, échec, secrets

**La garde anti-boucle.** Chaque tentative automatique est identifiée par un ensemble de repères — dernier tour, dernier message, nombre de messages, dernier point de reprise, fournisseur, modèle, fenêtre, profil et sa révision (`automatic_guard.rs:129-157`). Trois règles en découlent (`automatic_guard.rs:165-200`) :

- une tentative identique à la précédente **n'est pas rejouée** ;
- une compression réussie **ne peut pas se relancer dans le même tour principal**, même si elle a changé tous les identifiants ;
- **tout changement d'environnement remet le compteur à zéro** — changer de modèle ou de profil débloque une conversation coincée.

**Après trois échecs consécutifs, la compression automatique se met en pause pour cette conversation** (`automatic_guard.rs:202-205`). Le message affiché le dit et donne la sortie : *« La compression automatique est suspendue pour cette session après plusieurs échecs. Vous pouvez réessayer avec /compress. »* (`fr.json:1430` ; émis en `orchestrator.rs:70-73`).

**L'écriture est transactionnelle.** Avant de remplacer quoi que ce soit, Beaver vérifie que la conversation sur disque est toujours **identique octet pour octet** à celle qu'il a photographiée au début (`checkpoint_transaction.rs:63-65` ; comparaison `checkpoint_candidate.rs:220-225`). Si un message est arrivé entre-temps, la compression est abandonnée plutôt qu'appliquée sur un état périmé. La même vérification est faite une seconde fois, plus tôt, au moment de poser la garde (`automatic_guard.rs:25-30`).

**Le délai d'attente** de la requête de résumé est de **600 secondes**, avec la même valeur en délai d'inactivité (`timeouts.rs:5-11` ; test `timeouts_tests.rs:6-7`).

### 9. Les trois tailles de fenêtre, et le cas des petits modèles

Un profil ne porte pas un réglage mais **trois**, un par taille de fenêtre (`profile_types.rs:7-13`, `:44-46`). Les bornes sont fixes (`profile_limits.rs:16-17` ; classement `profile_budget.rs:3-13`) :

| Plage | Fenêtre du modèle | Libellé affiché |
|---|---|---|
| Petite | moins de **64 000** jetons | « moins de 64K » (`fr.json:1105`) |
| Moyenne | de 64 000 à moins de **128 000** | « 64K – moins de 128K » (`fr.json:1106`) |
| Grande | **128 000** et plus | « 128K et plus » (`fr.json:1107`) |

**Sous 64 000 jetons, la compression est désactivée par défaut** (`profile_defaults.rs:14`, champ `allow_under_64k` à `false`). Ce n'est pas un oubli : la raison est donnée à l'utilisateur dans l'application, et le site peut la reprendre telle quelle (`fr.json:442`) :

> « La compression est désactivée parce que cette fenêtre de contexte est inférieure à 64K. Ces petites fenêtres peuvent ne pas laisser assez de place pour générer puis réinjecter un résumé, les instructions, les outils et les échanges récents. Vous pouvez l’autoriser dans le profil de compression avancé. »

L'autorisation existe donc, et elle est accompagnée d'un avertissement dans l'éditeur (`fr.json:1111`) : *« Pour les modèles de moins de 64K, une configuration trop volumineuse peut saturer la fenêtre de contexte. La session peut rencontrer des problèmes et la compression peut échouer si elle réinjecte trop de contexte. »*

**Conséquence à écrire clairement sur le site** : sur un petit modèle local avec la compression laissée désactivée, une conversation longue **n'est pas compressée et n'avertit de rien** — l'automatique renonce en silence (`orchestrator.rs:49-50`). L'utilisateur qui veut savoir doit regarder l'anneau de contexte, qui affiche alors « Compression désactivée ».

### 10. Les profils de compression

Un **profil** rassemble tout ce qui est réglable : le seuil, l'autorisation sous 64K, les deux prompts, et les trois jeux de réglages par plage (`profile_types.rs:33-47`).

**Beaver en livre un seul, nommé « Beaver »** (`profile_defaults.rs:3`, `:11`). Ses valeurs, à reproduire telles quelles sur le site (`profile_defaults.rs:13-19`) :

| Réglage | moins de 64K | 64K – 128K | 128K et plus |
|---|---|---|---|
| Messages récents conservés | **2** | **4** | **4** |
| Taille maximale du résumé | **2 000** jetons | **4 000** | **6 000** |
| Résultats d'outils | **5** | **10** | **10** |
| Fichiers récents relus | **3** | **5** | **5** |
| Images conservées | **2** | **4** | **4** |
| État du travail inclus | Oui | Oui | Oui |

Seuil : **90 %**. Compression sous 64K : **désactivée**.

**Ce que l'utilisateur peut faire.** Créer jusqu'à **20 profils** (`profile_limits.rs:1`), les renommer, les modifier, les supprimer, revenir aux valeurs Beaver, et rétablir les seuls prompts sans toucher au reste (`compression-profile-bar.tsx:88-111` ; `compression-summary-prompts.tsx:24-61`). **Le profil « Beaver » ne peut être ni renommé ni supprimé** (`compression-profile-bar.tsx:54`) ; il peut seulement être remis à ses valeurs d'origine.

**La suppression est annulable pendant 30 secondes**, par un bouton dans une notification (`commands/compression_profiles_undo.rs:7` ; `compression-profile-undo.tsx:3-12`). Le jeton d'annulation est comparé en temps constant et effacé de la mémoire après usage (`compression_profiles_undo.rs:74-90`) — un détail à ne pas mettre sur le site, mais qui explique pourquoi l'annulation est fiable.

**Deux niveaux de choix.** Le profil global se choisit dans Réglages › Avancé › Compression (`compression-settings-card.tsx:111-138`). Une conversation peut en utiliser un autre, choisi depuis le menu « + » du champ de saisie, entrée « Compression » (`chat-plus-menu.tsx:89-100` ; `chat-plus-compression-menu.tsx:34-58`). Le choix par conversation l'emporte, **tant que la sélection globale n'a pas changé** : Beaver mémorise le numéro de révision de la sélection globale au moment du choix, et retombe sur le profil global si celui-ci a bougé depuis (`profile_resolve.rs:60-73`).

**Les bornes de saisie**, toutes vérifiées à l'enregistrement (`profile_limits.rs:5-15` ; `profile_validation.rs:37-52`, `:79-90`) :

| Réglage | Borne |
|---|---|
| Nom d'un profil | **48 caractères**, jamais deux fois le même nom |
| Prompt personnalisé | **32 000 caractères** |
| Messages récents | **8** au maximum |
| Résultats d'outils | **50** |
| Fichiers récents | **15** |
| Images | **16** |
| Taille du résumé | de **1 000** à **8 000** jetons |
| Seuil | de **1 %** à **90 %** |
| Nombre de profils | **20** |

**L'aperçu de budget.** Le pied de l'éditeur estime, avant tout usage, ce que le profil enverrait : la part système et outils, la part réinjectée par le profil, la cible estimée et la **réduction attendue en pourcentage** (`compression-budget-preview.tsx:21-124` ; `fr.json:1139-1148`). Le calcul est fait sur une conversation de démonstration de **96 000 jetons** dont **12 000** de tête système, pas sur la conversation réelle (`profile_limits.rs:19-20`), et le résultat est arrondi par tranches de 8 000 jetons (`checkpoint_target.rs:4`, `:29-34`). Le site doit le dire : **c'est une estimation de comparaison entre profils, pas une prédiction.**

### 11. La compression à la demande : `/compress`

Le chemin exact, à décrire pas à pas :

1. taper `/` dans le champ de saisie ouvre la liste des commandes ;
2. `compress` y figure comme commande intégrée (`src/hooks/use-slash-commands.ts:40-47`) ;
3. la sélectionner **écrit seulement le texte `/compress` dans le champ** (`src/hooks/use-active-skills.ts:28-29`) ;
4. il faut ensuite l'envoyer comme un message normal.

Il n'existe aucun bouton ni raccourci clavier dédié.

À l'envoi, Beaver ne répond pas : il compresse et s'arrête là (`commands/agent_chat_task.rs:89-107`). C'est le geste à recommander dans trois situations : avant de changer de sujet dans une longue conversation, quand la compression automatique s'est suspendue, et quand on veut compresser sans attendre le seuil.

### 12. Où tout cela est rangé

| Ce qui est rangé | Où | Source |
|---|---|---|
| Profils, profil global, interrupteur automatique | `~/.local/share/cl-go-dash/compression-profiles.json` | `profile_store.rs:143-145` |
| Copie de sauvegarde avant migration de format | `compression-profiles.v1.bak`, à côté | `profile_store_migration.rs:119` |
| Copie de sauvegarde de l'ancien `config.json` | `config.json.compression-v1.bak` | `services/config_compression_migration.rs:103` |
| Points de reprise et compteur de compressions | Dans le fichier de la conversation, `agent-sessions/*.json` | `checkpoint_transaction.rs:66-68` |

Le fichier de profils **porte un numéro de version, actuellement 2** (`profile_store_document.rs:8`), est borné à **1 Mio** (`profile_store.rs:7`), est écrit de façon atomique (`profile_store.rs:118`), et suit la règle de lecture tolérante du projet :

- un fichier écrit par une **version plus récente** de Beaver est **refusé, jamais réécrit** (`profile_store.rs:126-133`) ;
- un fichier **illisible** est remplacé par les valeurs d'usine, mais la sauvegarde de l'ancien format est **conservée durablement** parce qu'une réparation d'usine ne prouve pas que les profils ont été récupérés (`profile_store.rs:72-82`, commentaire `:76-77`).

Les réglages de compression vivaient auparavant dans `config.json` (champs `compression_enabled` et `compression_threshold`). Ils sont repris automatiquement au premier accès, puis retirés de `config.json` après sauvegarde (`config_compression_migration.rs:24-55`).

---

## Tableaux

### Les cinq messages d'erreur, mot pour mot

| Cause | Message affiché | Sources |
|---|---|---|
| Le moteur de compression est indisponible | « La compression est actuellement indisponible. » | `agent-error-codes.ts:36` ; `fr.json:1426` |
| Le fichier de profils est illisible ou d'une version future | « Les réglages de compression sont actuellement indisponibles. » | `agent-error-codes.ts:37` ; `fr.json:1427` |
| Fenêtre sous 64K et autorisation non donnée | « La compression est désactivée pour les fenêtres de contexte inférieures à 64K. Vous pouvez l’autoriser dans le profil de compression avancé. » | `agent-error-codes.ts:38` ; `fr.json:1428` |
| Trois échecs automatiques d'affilée | « La compression automatique est suspendue pour cette session après plusieurs échecs. Vous pouvez réessayer avec /compress. » | `agent-error-codes.ts:39` ; `fr.json:1430` |
| Toutes les autres causes | « Le contexte n’a pas pu être compressé. Votre conversation n’a pas été modifiée. » | `agent-error-codes.ts:40` ; `fr.json:1429` |

Le dernier message est un **fourre-tout** : quatorze causes internes distinctes existent dans le code (`checkpoint_transaction.rs:7-22`) et dix d'entre elles aboutissent au même texte (`checkpoint_transaction.rs:33-40`). Sa dernière phrase — « Votre conversation n'a pas été modifiée » — est **exacte**, et c'est l'information qui compte pour l'utilisateur.

### Ce que la compression garde et ce qu'elle perd

| | Devient | Retrouvable ensuite |
|---|---|---|
| Messages anciens | Le résumé en neuf sections | **Non** |
| Résultats d'outils anciens | Une mention dans le résumé | Les gros résultats restent sur disque dans `tool-results/`, référencés par les extraits (`checkpoint_tools.rs:64-72`) |
| Fichiers lus | Leur contenu **actuel**, relu du disque | Oui, c'est le fichier réel |
| Images anciennes | Rien au-delà du quota du profil | **Non** |
| État Git, plan, tâches, sous-agents | Une section compacte du point de reprise | Reflète l'état au moment de la compression |
| Messages récents, tour en cours | Repris tels quels | Oui |

---

## Encadrés

> **ℹ À placer en tête de page — Le résumé en quatre phrases.**
> Quand une conversation atteint 90 % de la fenêtre du modèle, Beaver la fait résumer en neuf sections imposées, garde les derniers échanges et l'état du travail en cours, puis remplace l'historique par ce point de reprise. La conversation continue au même endroit, sans que vous ayez rien à faire. Si quoi que ce soit échoue, la conversation reste intacte. Tout est réglable dans Réglages › Avancé › Compression, et vous pouvez déclencher l'opération à tout moment en envoyant `/compress`.

> **⚠ À placer à la section « Ce qui est conservé » — La compression supprime réellement les anciens messages.**
> Ce n'est pas un repli d'affichage : le fichier de votre conversation ne les contient plus après l'opération, et il n'existe aucun moyen de les retrouver. Ce qui subsiste est le résumé, les derniers échanges et l'état du travail. Si un échange ancien vous importe, copiez-le ailleurs avant que la conversation ne s'allonge.

> **ℹ À placer à la section « Ce qui est conservé » — Les fichiers sont relus, pas recopiés.**
> Le point de reprise contient le contenu **actuel** des fichiers récemment consultés, pas celui que l'agent avait vu. C'est voulu : cela lui permet de reprendre sur l'état réel du projet. Mais si un fichier a changé entre-temps, c'est bien la nouvelle version qui entre dans le point de reprise.

> **⚠ À placer à la section « Petits modèles » — Sous 64K, rien ne se passe et rien ne le dit.**
> Sur un modèle dont la fenêtre effective est inférieure à 64 000 jetons, la compression est désactivée par défaut et l'automatique renonce **sans message**. La conversation continue de grossir jusqu'à saturer la fenêtre. Le seul endroit où cela se voit est l'anneau de contexte, qui affiche alors « Compression désactivée ». L'autorisation existe dans l'éditeur avancé, avec son propre avertissement.

> **ℹ À placer à la section « Le résumé » — La conversation est traitée comme des données, jamais comme des instructions.**
> Le modèle qui rédige le résumé reçoit un contrat qu'aucun contenu de la conversation ne peut modifier, et l'historique lui est présenté explicitement comme des données non fiables. Il lui est interdit de révéler des secrets et de mentionner vos réglages de permission — et un filtrage automatique retire de toute façon les données sensibles du résumé avant qu'il soit rangé.

> **ℹ À placer à la section « /compress » — La compression à la demande ignore vos réglages.**
> `/compress` ne regarde ni le seuil, ni l'interrupteur « Compression automatique ». Il compresse tout de suite. Et contrairement à l'automatique, il vous **dit** pourquoi quand il n'y arrive pas — c'est le bon réflexe quand une conversation semble bloquée.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Ma conversation n'est jamais compressée » | Fenêtre sous 64 000 jetons et autorisation non donnée — l'automatique renonce en silence | Vérifier l'anneau de contexte ; activer « Autoriser la compression sous 64K », ou passer à un modèle à plus grande fenêtre |
| « La compression automatique ne se déclenche plus du tout » | Trois échecs d'affilée l'ont suspendue pour cette conversation | Envoyer `/compress` — ou changer de modèle ou de profil, ce qui remet le compteur à zéro |
| « Une réponse s'est arrêtée en plein milieu, suivie d'une compression » | Comportement voulu : le seuil est surveillé pendant la génération, toutes les 32 jetons | Rien à faire, la réponse reprend après |
| « Le contexte n'a pas pu être compressé » | Message générique couvrant dix causes internes | Réessayer avec `/compress`, qui donne parfois un message plus précis ; la conversation est intacte |
| `/compress` répond que la compression est désactivée | Fenêtre sous 64K, autorisation non donnée | Activer l'autorisation dans l'éditeur avancé, en acceptant son avertissement |
| `/compress` ne fait rien | Le message contenait autre chose que `/compress` seul | Envoyer `/compress` seul, sans mot ajouté |
| « J'ai perdu un détail dont l'agent avait besoin » | Il n'entrait ni dans le résumé ni dans les éléments conservés | Le redonner dans un message ; augmenter les réglages du profil pour la suite |
| « J'ai choisi un profil pour cette conversation et il est revenu au profil global » | Le profil global a été changé depuis, ce qui annule les choix par conversation | Le rechoisir depuis le menu « + » du champ de saisie |
| « Mes profils ont disparu après une mise à jour ratée » | Fichier illisible, remplacé par les valeurs d'usine | La sauvegarde `compression-profiles.v1.bak` est conservée à côté |
| « La réduction annoncée dans l'aperçu ne correspond pas » | L'aperçu se calcule sur une conversation de démonstration de 96 000 jetons | C'est un outil de comparaison entre profils, pas une prédiction |

---

## Renvois

- `04-agent/fonctionnement.md` — la boucle de l'agent, où la compression s'insère
- `04-agent/sous-agents.md` — pourquoi leurs rapports entrent dans le point de reprise
- `06-modeles/materiel-et-vram.md` — pourquoi la fenêtre effective d'un modèle local dépend de la machine
- `06-modeles/ollama-runtime.md` — la fenêtre `num_ctx` et son effet sur la plage de compression
- `10-reglages/avance.md` — l'emplacement du panneau de compression dans les réglages
- `11-securite/vault-et-cles-api.md` — le masquage des données sensibles, appliqué ici aussi
- `12-reference/emplacement-des-donnees.md` — `compression-profiles.json` et ses sauvegardes
- `03-interface/conversations-et-onglets.md` — l'anneau de contexte et le menu « + »

---

## Anomalies relevées

Constatées dans le code, **non corrigées** : à signaler à l'équipe, pas à documenter comme des fonctionnalités.

1. **Faute d'accord dans le marqueur du fil.** Le texte affiché est **« Contexte compressée »** (`fr.json:339`) — « contexte » est masculin. C'est la chaîne la plus visible de toute la fonctionnalité, présente dans chaque conversation compressée. Correction à répercuter sur les sept langues si le libellé est retouché.
2. **Le résumé généré n'est consultable nulle part.** Le marqueur du fil n'a aucun gestionnaire de clic (`context-compression-marker.tsx:1-12`) et le contenu du message de point de reprise n'est jamais rendu (`src/lib/context-messages.ts:7-17`). L'utilisateur ne peut donc pas vérifier ce que Beaver a retenu de sa conversation, alors que le texte existe dans le fichier de session. C'est l'écart le plus important entre ce que la fonctionnalité fait et ce qu'elle montre.
3. **Aucune nouvelle tentative n'est faite en cas d'échec réseau du résumé.** Le module de reprise existe et définit ses délais (`summary_retry.rs:3-8`), la mécanique de reprise est écrite (`summary_request.rs:103-107`), mais le seul appel de production passe `0` en nombre de tentatives (`orchestrator_summary.rs:122`). Une coupure réseau passagère fait donc échouer la compression du premier coup, et trois de ces échecs suspendent l'automatique.
4. **La description de `/compress` dans la liste des commandes n'est pas traduite.** Elle est écrite en anglais, en dur, sans clé i18n : *« Compress conversation context manually »* (`src/hooks/use-slash-commands.ts:40-47`).
5. **Deux clés de traduction paraissent orphelines** : `settings.advanced.compressionProfileTitle` et `compressionProfileDesc` (`fr.json:1093-1094`) n'ont été retrouvées dans aucun composant de production, seulement dans un test.
6. **Code résiduel dans la collecte des fichiers** : l'énumération `FileKind` n'a qu'une seule variante et son paramètre est explicitement ignoré (`checkpoint_files.rs:20-23`, `:49`).
7. **`compression-profiles.json` ne figure pas dans l'inventaire des données du projet** (`CLAUDE.md`, section « Data sources »), alors que c'est un fichier persistant, versionné et porteur de réglages utilisateur. La page `12-reference/emplacement-des-donnees.md` doit l'ajouter.

---

## Points à confirmer

**À trancher avec l'équipe avant publication**

1. **Faut-il documenter que les anciens messages sont réellement supprimés ?** C'est le fait le plus important de la page et le plus susceptible d'inquiéter. Ma recommandation est oui, franchement, dans un encadré — la même logique que celle retenue pour le coffre : la transparence sur une contrainte réelle vaut mieux que la découverte après coup. **Décision de Kevin nécessaire.**
   *Décision du 10 septembre 2026 : documenter franchement la suppression des anciens messages ; une seule page.*
2. **Quel vocabulaire français pour « checkpoint » ?** J'ai employé « point de reprise » partout dans ce fichier, mais **l'application n'emploie aucun terme** : elle dit seulement « Contexte compressée ». Le mot choisi doit être le même sur tout le site et, idéalement, apparaître aussi dans l'application. Trois candidats : « point de reprise », « point de reprise du contexte », « repère de compression ».
3. **Faut-il présenter l'éditeur avancé de profils sur la page principale, ou lui donner sa propre page ?** Il compte à lui seul six sections, trois plages, une vingtaine de réglages et un aperçu de budget. Ma recommandation : une page « La compression du contexte » qui s'arrête au point 9, et une page « Régler la compression » pour les points 10 à 12.
   *Décision du 10 septembre 2026 : documenter franchement la suppression des anciens messages ; une seule page.*
4. **Documente-t-on l'anomalie 3 (aucune nouvelle tentative) sur le site ?** Elle explique un comportement réellement observable — une compression qui échoue au premier problème réseau — mais elle sera peut-être corrigée avant la mise en ligne. À revérifier juste avant publication.

**Non vérifié — hors de portée d'une lecture du code**

5. **La qualité réelle des résumés produits par de petits modèles locaux.** Le contrat impose neuf sections et un format strict ; un modèle de 3 ou 7 milliards de paramètres peut échouer à le respecter, ce qui fait rejeter le résumé et échouer la compression. Non mesuré. À rapprocher de ce qui est déjà connu par ailleurs sur les petites fenêtres.
6. **La durée réelle d'une compression.** Le délai maximal est de 600 secondes, mais la durée usuelle n'a pas été mesurée. Le bandeau n'affiche ni progression ni estimation ; savoir si l'attente est de trois secondes ou de trente change ce qu'il faut écrire sur le site.
7. **Le comportement quand l'utilisateur arrête la compression** avec le bouton d'arrêt pendant qu'elle tourne. Le code prévoit l'annulation (`orchestrator_started.rs:15-17` ; `summary_contract.rs:24-26`) et la conversation doit rester intacte, mais l'enchaînement visible n'a pas été provoqué.
8. **Le comportement du choix de profil par conversation après un clonage de conversation.** Le code prévoit de reporter la sélection (`profile_resolve.rs:83-88`), non observé.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

9. **Le bandeau « Compression du contexte »** : sa position dans le fil, la lisibilité des deux traits qui pulsent, et son rendu dans les deux thèmes.
10. **Le marqueur « Contexte compressée »** dans le fil : sa visibilité au milieu d'une longue conversation, et le fait qu'il ne soit pas cliquable — un élément qui ressemble à un bouton sans en être un est à vérifier à l'œil.
11. **L'anneau de contexte** dans ses deux états, « Compression · *nom du profil* » et « Compression désactivée », ainsi que l'infobulle d'aide et sa fermeture au clavier (`context-compression-help-popover.tsx:41-54`).
12. **Le panneau de compression des réglages** : c'est une fenêtre modale contenant une barre de profils, un éditeur à six sections, trois onglets de plage et un aperçu graphique. Sa hauteur, son défilement et le comportement des menus déroulants à l'intérieur sont à vérifier dans les deux thèmes.
13. **L'aperçu de budget** : la jauge à deux segments, ses états « Calcul de la projection… » et « Projection temporairement indisponible » (`fr.json:1147-1148`), et la lisibilité de ses deux couleurs de segment.
14. **La notification d'annulation après suppression d'un profil** et sa fenêtre de 30 secondes : est-ce que le compte à rebours est visible, et que se passe-t-il si on clique après expiration ?
15. **Les cinq messages d'erreur** ne recevront pas tous de capture : provoquer une suspension après trois échecs ou un dépassement de capacité suppose de forcer des pannes. Ils sont documentés d'après le code, comme le prévoit la convention de ce dossier.
