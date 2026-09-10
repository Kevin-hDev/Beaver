# Questions fréquentes

**Emplacement site** — Référence › Dépannage › Questions fréquentes
**Répond à** — « Les questions courtes qui n'ont pas de page à elles : où sont mes fichiers, que se passe-t-il hors ligne, l'application est-elle libre, comment je la désinstalle. »
**Sources** — `src-tauri/src/services/paths.rs` ; `src-tauri/src/services/vault.rs` ; `src-tauri/src/services/api_keys.rs` ; `src-tauri/src/app_events.rs` ; `src-tauri/src/models/config.rs` ; `src-tauri/src/commands/app_update_source.rs` ; `src-tauri/src/services/forecast/sidecar_process_env.rs` ; `src/i18n/index.ts` ; `src/hooks/use-theme.ts` ; `src/hooks/use-update-checker.ts` ; `src/components/settings/about-settings.tsx` ; `src/components/settings/session-workspace-settings.tsx` ; `src/lib/brand.ts` ; `src/i18n/fr.json` ; `LICENSE`, `CLA.md`, `README.md`
**Vérification** — Vérifié dans le code et dans les fichiers du dépôt le 10 septembre 2026. Chaque réponse porte sa source. Les questions sans réponse vérifiable sont en fin de fichier, dans « Points à confirmer », et ne doivent pas être publiées telles quelles.

---

## Avertissement au rédacteur

**Une question n'entre dans cette page que si elle a une réponse sourcée.** Une FAQ inventée est pire qu'une FAQ absente : le lecteur y croit et ne vérifie pas.

**Cette page ne double aucune autre.** Les questions qui relèvent d'une page existante figurent en fin de fichier, dans un tableau de renvois d'une ligne chacun — à reprendre tel quel sur le site, parce qu'un lecteur cherche d'abord dans la FAQ.

**Le ton des questions doit être celui de l'utilisateur**, pas celui de la documentation : « Où sont mes conversations ? », pas « Emplacement des données de session ».

---

## Plan de page proposé

1. Où sont mes données ?
2. Comment je sauvegarde tout — et qu'est-ce qui ne se restaure pas ?
3. Pourquoi Beaver demande-t-il mon trousseau au démarrage ?
4. Est-ce que Beaver fonctionne sans Internet ?
5. Est-ce que Beaver contacte Internet tout seul ?
6. Fermer la fenêtre, est-ce que ça quitte l'application ?
7. Est-ce que Beaver démarre avec ma machine ?
8. L'application est-elle gratuite ? libre ? puis-je m'en servir au travail ?
9. Où est le numéro de version ?
10. Comment désinstaller proprement ?
11. Les questions qui ont leur propre page

---

## Contenu

### 1. Où sont mes données ?

**Dans un seul dossier**, identique sur macOS, Windows et Linux :

**`~/.local/share/cl-go-dash/`** (`services/paths.rs:10-14` — le chemin est calculé à un seul endroit et n'est écrit en dur nulle part ailleurs).

Sous Windows, cela correspond à `C:\Users\<votre compte>\.local\share\cl-go-dash\` : un emplacement inhabituel pour ce système, hérité du choix d'un chemin unique sur les trois.

**Le raccourci depuis l'application** : Réglages › Avancé › **Données Beaver** › bouton **Ouvrir le dossier** (`components/settings/session-workspace-settings.tsx:28`, `:67-75` ; libellés `fr.json:1177-1179`, dont la description « Ouvrir le dossier qui contient les sessions, la mémoire et les réglages locaux »).

**Ce dossier ne se déplace pas.** Aucun réglage, aucune variable d'environnement ne permet de le changer : le chemin est fixe dans le code.

Deux choses vivent **en dehors** de ce dossier, et c'est important pour la question suivante :

- **la clé qui ouvre vos clés API**, confiée au gestionnaire de secrets du système (`services/vault.rs:15-17` : service `cl-go-dash`, entrée `master-key`) ;
- **la langue et le thème de l'interface**, rangés dans le stockage local du composant d'affichage sous les noms `clgo-language` (`src/i18n/index.ts:33`), `clgo-theme` et `clgo-theme-base` (`src/hooks/use-theme.ts:27-28`).

Les modèles locaux d'Ollama, eux, sont ailleurs encore — voir `06-modeles/ollama-modeles.md`.

### 2. Comment je sauvegarde tout — et qu'est-ce qui ne se restaure pas ?

**Il n'existe aucune fonction de sauvegarde ni de restauration dans l'application.** Rien dans les commandes, rien dans les réglages : la seule action prévue est le bouton qui ouvre le dossier.

**La sauvegarde, c'est donc copier `~/.local/share/cl-go-dash/`**, Beaver fermé.

**Ce que la copie restaure**, sur la même machine ou sur une autre : les conversations, les projets, la mémoire, les réglages, les analyses de prévision, les notes, les connecteurs configurés.

**Ce que la copie ne restaure pas**, et qu'il faut écrire sans l'adoucir :

| Élément | Pourquoi |
|---|---|
| **Vos clés API** | Le fichier `secrets.enc` est chiffré par une clé restée dans le trousseau de l'ancienne machine. Beaver refuse volontairement d'en fabriquer une nouvelle (`vault.rs:66-71`) : les clés sont à ressaisir |
| **La langue de l'interface** | Elle est dans le stockage local, pas dans le dossier de données (`i18n/index.ts:33`) : l'application redémarre en anglais |
| **Le thème** | Même raison (`use-theme.ts:27-28`) : il repart sur « système » |
| **Les modèles locaux** | Ils vivent hors de ce dossier et pèsent plusieurs gigaoctets |

**Un détail que la page doit relever** : plusieurs messages d'erreur de l'application vous demandent de « restaurer depuis une sauvegarde » — par exemple « Le fichier de personnalisation Ollama est illisible. Restaure « ollama-custom-models.json » depuis une sauvegarde dans Réglages > Avancé > Données Beaver. » (`fr.json:1352`, et cinq messages voisins `:1353-1357`). L'application suppose donc une sauvegarde qu'elle n'a jamais proposé de faire. **C'est une raison suffisante pour recommander la copie du dossier**, et pour le dire tôt sur le site.

### 3. Pourquoi Beaver demande-t-il mon trousseau au démarrage ?

Parce que vos clés API sont chiffrées, et que la clé qui les déchiffre est confiée au gestionnaire de secrets de votre système — Trousseau d'accès sur macOS, Gestionnaire d'identifiants sur Windows, service de secrets du bureau sur Linux.

**Cela n'arrive qu'une fois par lancement.** La fonction qui lit cette clé n'a qu'un seul appelant dans tout le code : la routine d'initialisation exécutée au démarrage (`services/vault.rs:49-50`, appelée uniquement depuis `services/api_keys.rs:46-47`). Vous n'êtes pas sollicité à chaque requête.

**Au tout premier lancement**, il n'y a rien à lire : Beaver tire **32 octets** au hasard et les dépose dans le trousseau (`vault.rs:72-81`). C'est à ce moment que le système peut demander une autorisation.

**Si vous refusez, ou si aucun service de secrets ne tourne** — cas courant sur une session Linux minimale — l'application démarre quand même, affiche un bandeau, et refuse toute opération sur les clés API. Les modèles locaux, qui n'ont besoin d'aucune clé, continuent de fonctionner.

Le détail complet est dans `11-securite/vault-et-cles-api.md`.

### 4. Est-ce que Beaver fonctionne sans Internet ?

**Oui, pour l'essentiel — à condition d'avoir déjà installé ce dont vous vous servez.** C'est la ligne à faire passer.

**Ce qui fonctionne hors ligne :**

- **les modèles locaux**, une fois téléchargés ;
- **les modèles de prévision locaux**, une fois installés : leur moteur est démarré en mode hors ligne forcé, avec `HF_HUB_OFFLINE` et `TRANSFORMERS_OFFLINE` posés à chaque lancement, et un test verrouille ce comportement (`services/forecast/sidecar_process_env.rs:28-36`, test `:45-56`) ;
- tout ce qui touche à vos fichiers, au terminal, à Git, aux conversations déjà enregistrées.

**Ce qui ne fonctionne pas :**

- les modèles distants, quels qu'ils soient ;
- la recherche web, les connecteurs externes, les canaux de messagerie ;
- **l'installation** de quoi que ce soit : un modèle local se télécharge, un modèle de prévision télécharge aussi des bibliothèques Python.

**Le cas particulier du tout premier lancement** : Beaver a besoin d'Internet une fois, pour récupérer le moteur Ollama. Voir `13-depannage/installation.md`, section 8.

**La vérification des mises à jour échoue silencieusement** quand il n'y a pas de réseau : rien ne s'affiche, sauf si vous l'avez déclenchée vous-même (voir la question suivante).

### 5. Est-ce que Beaver contacte Internet tout seul ?

**Oui, pour une seule chose : chercher les mises à jour.** C'est vérifiable et il faut le dire simplement.

La vérification part **au lancement de l'application, puis toutes les heures** (`src/hooks/use-update-checker.ts:13` — une heure en millisecondes — et `:110-112`). Elle porte sur trois choses à la fois : la version de Beaver, la version du moteur Ollama, et les mises à jour de vos modèles Ollama installés (`:78-84`).

Pour l'application, l'adresse interrogée est **`https://api.github.com/repos/Kevin-hDev/Beaver/releases/latest`** (`commands/app_update_source.rs:15`, `:29-33`, `:43-49`). Aucune donnée personnelle n'y est envoyée : c'est une lecture publique de la dernière version publiée.

**En cas d'échec, rien ne s'affiche.** Le message « La recherche des mises à jour a échoué. Réessaie dans un instant. » (`fr.json:1448`) n'apparaît que si vous avez lancé la vérification vous-même (`use-update-checker.ts:97-99`) — et, dans l'état actuel du code, seulement si c'est la vérification du moteur Ollama qui a échoué. Voir « Anomalies relevées ».

**Sur la télémétrie**, ce qui est vérifié : Beaver **désactive** celle des composants qu'il lance — `DO_NOT_TRACK=1`, `HF_HUB_DISABLE_TELEMETRY=1` et `TABPFN_DISABLE_TELEMETRY=1` sont posés sur chaque processus de prévision (`services/forecast/sidecar_process_env.rs:29-36` ; `model_manager/smoke.rs:64`). L'absence de toute télémétrie propre à Beaver n'a pas été auditée ligne à ligne : voir « Points à confirmer », point 1, et renvoyer à `11-securite/confidentialite-des-donnees.md`.

### 6. Fermer la fenêtre, est-ce que ça quitte l'application ?

**Ça dépend du système, et la différence est voulue** (`src-tauri/src/app_events.rs:13-18`, `:29-37`) :

| Système | Ce que fait la croix de fermeture |
|---|---|
| **macOS** | **La fenêtre est masquée, l'application continue de tourner.** Elle se rouvre en cliquant sur l'icône du Dock (`app_events.rs:42-48`) |
| **Windows et Linux** | **L'application se ferme** |

Sur les trois systèmes, une icône dans la barre système est affichée par défaut (`models/config.rs:43`), et le réglage se trouve dans Réglages › Avancé (`fr.json:1053-1054`).

### 7. Est-ce que Beaver démarre avec ma machine ?

**Non, pas par défaut.** Le réglage **Lancer au démarrage** existe mais est désactivé à l'installation (`src-tauri/src/models/config.rs:41`), tout comme **Démarrage masqué**, qui lance l'application en arrière-plan sans ouvrir la fenêtre (`:42`).

Les deux se trouvent dans Réglages › Avancé (`fr.json:1049-1052`).

### 8. L'application est-elle gratuite ? libre ? puis-je m'en servir au travail ?

**Beaver est publié sous la licence GNU Affero General Public License version 3** (fichier `LICENSE` à la racine du dépôt ; annonce dans `README.md:404-425`). Le droit d'auteur appartient à Kevin Huynh (`README.md:408`).

Ce que cela veut dire, en une phrase chacun :

- **Vous pouvez l'utiliser, l'étudier, le modifier et le redistribuer**, y compris en entreprise, sans rien payer.
- **En contrepartie**, toute version que vous distribuez ou que vous mettez à disposition sur un réseau — modifiée ou non — doit être publiée sous la même licence, avec son code source complet (`README.md:410-412`).
- **Une licence commerciale existe** pour qui veut être dispensé de cette obligation ; elle s'obtient auprès de l'auteur (`README.md:417-418`).
- **Contribuer au projet demande de signer un accord de contribution** (`CLA.md`, référencé depuis `README.md:414-415`).
- **Les versions jusqu'à la 1.1.2 incluse** avaient été publiées sous licence Apache 2.0 ; elles ne sont plus distribuées (`README.md:423-425`).

Les composants tiers gardent leurs propres licences, listées dans `THIRD_PARTY_NOTICES.md`.

**Un point à signaler à l'équipe du site : l'application n'affiche nulle part sa licence.** L'écran **À propos** ne montre que la version de Beaver, la version de son cadre applicatif, le système d'exploitation, et un bouton vers le dépôt (`components/settings/about-settings.tsx:41-54`). Le lecteur qui se pose la question ne peut y répondre que depuis le site ou le dépôt.

### 9. Où est le numéro de version ?

**Réglages › À propos.** L'écran affiche la version de Beaver, celle de son cadre applicatif Tauri, et le système d'exploitation détecté (`components/settings/about-settings.tsx:41-43`), plus un bouton **Voir sur GitHub** qui ouvre `https://github.com/Kevin-hDev/Beaver` (`:13`, `:46-54` ; identifiant du dépôt dans `src/lib/brand.ts:3`).

C'est le numéro à donner quand vous signalez un problème.

### 10. Comment désinstaller proprement ?

Retirer l'application ne suffit pas : trois choses restent.

1. **Le dossier de données** `~/.local/share/cl-go-dash/` — conversations, réglages, modèles de prévision, coffre chiffré.
2. **L'entrée du gestionnaire de secrets** de votre système, au nom de service **`cl-go-dash`**, identifiant **`master-key`** (`services/vault.rs:15-17`). Tant qu'elle est là, elle ne sert plus à rien ; supprimée, le coffre devient définitivement illisible.
3. **Les modèles Ollama**, qui vivent dans un dossier partagé avec toute autre installation d'Ollama sur la machine — à ne pas supprimer sans vérifier.

**La procédure système par système est dans `13-depannage/installation.md`, section 13**, qui fait autorité. Cette page ne la duplique pas ; elle rappelle seulement les trois restes, parce que c'est la question posée.

### 11. Les questions qui ont leur propre page

Tableau de renvois, à reprendre tel quel : un lecteur cherche d'abord dans la FAQ.

| Question | Où est la réponse |
|---|---|
| Comment je change la langue de l'interface ? | `03-interface/langues.md` |
| Comment je change le thème, la police, la taille du texte ? | `03-interface/themes-et-apparence.md` |
| Où je mets ma clé API, et qui peut la relire ? | `11-securite/vault-et-cles-api.md` |
| Est-ce que mes conversations partent quelque part ? | `11-securite/confidentialite-des-donnees.md` |
| Quel modèle ma machine peut-elle faire tourner ? | `06-modeles/materiel-et-vram.md` |
| Local ou distant : lequel choisir ? | `01-decouverte/local-vs-cloud.md` |
| Comment je mets Beaver à jour ? | `02-installation/mise-a-jour.md` |
| L'installation a échoué, l'application ne démarre pas | `13-depannage/installation.md` |
| Mon modèle de prévision refuse de s'installer ou de calculer | `13-depannage/forecast.md` |
| Quels fichiers l'agent peut-il lire ou modifier ? | `04-agent/permissions.md` |

---

## Encadrés

> **ℹ À placer près de la question sur la sauvegarde — Sauvegarder, c'est copier un dossier.**
> Beaver n'a pas de fonction de sauvegarde. Fermez l'application, copiez `~/.local/share/cl-go-dash/`, et sachez que vos clés API ne s'y restaureront pas : elles sont chiffrées par une clé restée dans le trousseau de la machine d'origine.

> **⚠ À placer près de la question sur la sauvegarde — Vider le trousseau de votre système rend vos clés illisibles.**
> Beaver refuse volontairement de fabriquer une nouvelle clé maîtresse quand l'ancienne a disparu : cela écraserait un coffre encore valable. Il faut alors ressaisir toutes vos clés.

> **ℹ À placer près de la question sur Internet — Une seule connexion automatique.**
> Au lancement puis toutes les heures, Beaver demande à GitHub s'il existe une version plus récente, et à Ollama si vos modèles ont bougé. Rien d'autre ne part de la machine sans que vous l'ayez demandé.

> **ℹ À placer près de la question sur macOS — Sur Mac, la croix ne quitte pas l'application.**
> Elle masque la fenêtre. Pour quitter réellement, utilisez le menu de l'application ou l'icône de la barre système. Sur Windows et Linux, la croix ferme bien Beaver.

> **ℹ À placer près de la question sur la licence — Libre, et gratuit.**
> Beaver est publié sous licence AGPL v3 : vous pouvez l'utiliser, l'étudier, le modifier et le redistribuer. La contrepartie ne concerne que ceux qui en distribuent une version : ils doivent publier leur code source.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| J'ai copié mon dossier de données sur un autre ordinateur, et aucune clé API ne marche | La clé maîtresse est restée dans le trousseau de l'ancienne machine | Ressaisir les clés ; copier `secrets.enc` seul ne sert à rien |
| Après restauration, l'application est en anglais | La langue est dans le stockage local, pas dans le dossier de données | La rechoisir dans Réglages › Général |
| Je ne trouve pas mon dossier de données dans `%APPDATA%` sous Windows | Le chemin est unifié sur les trois systèmes | Chercher `C:\Users\<compte>\.local\share\cl-go-dash\` |
| Je ferme la fenêtre sur Mac et Beaver tourne toujours | Comportement voulu sur macOS | Quitter par le menu de l'application |
| Beaver n'annonce jamais de mise à jour | La vérification échoue en silence quand le réseau manque | La déclencher à la main depuis l'indicateur de mises à jour |
| Un message me dit de restaurer un fichier « depuis une sauvegarde » que je n'ai jamais faite | L'application n'a pas de fonction de sauvegarde | Supprimer le fichier nommé et relancer pour repartir des valeurs par défaut, comme le message le propose en second recours |
| J'ai désinstallé Beaver et il reste des fichiers | Le dossier de données, l'entrée du trousseau et les modèles ne sont pas supprimés | Voir `13-depannage/installation.md`, section 13 |

---

## Renvois

- `13-depannage/installation.md` — installation, premier lancement, désinstallation complète
- `13-depannage/forecast.md` — les pannes du module de prévision
- `11-securite/vault-et-cles-api.md` — le coffre, la clé maîtresse, le trousseau du système
- `11-securite/confidentialite-des-donnees.md` — ce qui sort de la machine et ce qui n'en sort pas
- `03-interface/langues.md` et `03-interface/themes-et-apparence.md` — langue, thème, apparence
- `02-installation/mise-a-jour.md` — comment Beaver se met à jour
- `10-reglages/application.md` — les réglages avancés cités ici : démarrage, barre système, dossier de données

---

## Points à confirmer

1. **L'absence de télémétrie propre à Beaver n'a pas été auditée.** Ce qui est vérifié, c'est que Beaver désactive celle des composants qu'il lance, et qu'il interroge GitHub pour les mises à jour. Une revue complète des appels sortants relève de `11-securite/confidentialite-des-donnees.md` ; **ne pas écrire « Beaver n'envoie aucune donnée » sur la foi de cette page.**
2. **La boîte de dialogue du Trousseau macOS au premier lancement** n'a pas été observée. Le code ne la décrit pas : elle vient du système. À voir sur une machine avant de rédiger la question 3.
3. **La restauration d'une copie du dossier de données** n'a été essayée sur aucune machine. Les quatre éléments non restaurés sont déduits du code, pas d'un essai. À vérifier avant de publier la question 2, qui est celle où une erreur coûterait le plus cher.
4. **La taille occupée par une installation de Beaver** — application, dossier de données, moteur Ollama, modèles — n'est pas mesurée. Question fréquente, sans réponse chiffrée : ne pas l'inventer.
5. **Le comportement de la barre système sur Linux** selon l'environnement de bureau n'est pas vérifié : certains n'affichent pas d'icône de barre système, ce qui rendrait l'application difficile à retrouver après fermeture. Sans conséquence sur Windows et Linux, où la croix quitte réellement ; à vérifier si le réglage « Démarrage masqué » est activé.
6. **La formulation dans les six autres langues** des messages cités ici (`fr.json:1177-1179`, `:1352-1357`, `:1448`) n'a pas été relue.
7. **Une question fréquente sans réponse vérifiable, écartée volontairement** : « puis-je ouvrir plusieurs fenêtres de Beaver ? ». Le code ne l'a pas été examiné sur ce point ; à trancher avant d'ajouter la question.

---

## Anomalies relevées

Constatées dans le code ou les fichiers du dépôt, **non corrigées** — à transmettre à l'équipe.

1. **Six messages d'erreur demandent à l'utilisateur de restaurer un fichier « depuis une sauvegarde »** (`fr.json:1352-1357`) alors qu'aucune fonction de sauvegarde n'existe dans l'application. Le second recours proposé — supprimer le fichier et relancer — est le seul réellement disponible pour la plupart des utilisateurs.

2. **Le message « La recherche des mises à jour a échoué » ne se déclenche que sur l'échec de la vérification du moteur Ollama.** La condition testée porte sur le troisième résultat de la liste (`use-update-checker.ts:97-99`), qui est `check_ollama_binary_update` (`:81`). Si c'est la vérification de la version de Beaver qui échoue et que celle d'Ollama réussit, l'utilisateur qui a cliqué ne voit rien.

3. **L'application n'affiche nulle part sa licence.** L'écran À propos donne la version, le cadre applicatif et le système, mais aucune mention de l'AGPL v3 ni du droit d'auteur (`about-settings.tsx:41-54`), alors que la licence impose de transmettre ces informations aux destinataires du logiciel. Point à faire trancher, éventuellement avec un juriste.

4. **La langue et le thème ne suivent pas le dossier de données.** Ils sont rangés dans le stockage local du composant d'affichage (`i18n/index.ts:33` ; `use-theme.ts:27-28`), donc perdus lors d'un déplacement vers une autre machine, et invisibles pour toute sauvegarde du dossier de données. Choix cohérent en soi, mais qui prend l'utilisateur au dépourvu.

5. **La langue par défaut est l'anglais, sans tenir compte de la langue du système** (`i18n/index.ts:31-36` : en l'absence de valeur enregistrée, `"en"`). Le thème, lui, suit bien le réglage du système par défaut (`use-theme.ts:54-62`). Les deux réglages voisins n'ont pas la même politique.
