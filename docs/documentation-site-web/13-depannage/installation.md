# Dépannage — Installation

**Emplacement site** — Référence › Dépannage › Installation
**Répond à** — « L'installation a échoué, ou l'application ne démarre pas : qu'est-ce que je fais ? »
**Sources** — `install.sh` (racine), `install.ps1` (racine), `src-tauri/tauri.conf.json`, `.github/workflows/release.yml:135-161`, `src-tauri/src/app_build.rs:114-126`, `src-tauri/src/storage_migration.rs`, `src-tauri/src/services/private_store.rs`, `src-tauri/src/services/paths.rs`, `src-tauri/src/services/paths/ollama.rs`, `src-tauri/src/commands/ollama_setup.rs`, `src-tauri/src/services/ollama_manager/` (`release_source.rs`, `release_fetch.rs`, `download.rs`, `download_stream.rs`, `error.rs`, `spawn_profile_paths.rs`, `spawn_gate_unix.rs`), `src-tauri/src/services/gpu_detect.rs`, `src-tauri/src/services/gpu_vram.rs`, `src-tauri/src/services/gpu_vram/macos.rs`, `src-tauri/src/services/vault.rs`, `src/hooks/use-startup-gate.ts`, `src/lib/ollama-setup-gate.ts`, `src/components/ollama/ollama-setup-screen.tsx`, `src/components/ollama/ollama-tab.tsx`, `src/i18n/fr.json`
**Vérification** — Vérifié dans le code pour tous les messages, chemins, limites et enchaînements cités. Les comportements de Gatekeeper et de SmartScreen relèvent du système d'exploitation et ne sont pas dans le code de Beaver : ils sont décrits comme conséquences de faits vérifiés (absence de certificat de signature), et les points d'écran non observés sont listés en fin de fichier.

---

## Avertissement au rédacteur

Cette page est organisée **par symptôme**, pas par système. Elle ne réexplique aucune procédure d'installation : les pages *Installation macOS*, *Installation Windows*, *Installation Linux*, *Premier lancement* et *Mise à jour* font autorité et cette page y renvoie.

Deux règles de rédaction pour toute cette page :

1. **Chaque entrée commence par ce que l'utilisateur voit**, mot pour mot quand c'est un message de l'application. Quelqu'un qui cherche un dépannage cherche sa phrase à l'écran, pas une catégorie.
2. **Aucune promesse de résolution qui n'existe pas.** Plusieurs échecs listés ici n'ont pas de solution — architecture non distribuée, dépendance non gérée. Le dire, et donner l'issue réelle (renoncer, signaler, contourner autrement).

**Le vocabulaire des scripts d'installation tutoie l'utilisateur** (« Utilise sa mise à jour intégrée », `install.sh:135`) alors que ces fichiers vouvoient. Le site doit trancher un ton unique, et **citer les messages tels qu'ils sortent du script**, sans les réécrire — sinon l'utilisateur ne reconnaît pas sa phrase.

---

## Plan de page proposé

1. Avant de chercher : les trois causes qui expliquent la moitié des cas
2. macOS — « Beaver ne peut pas être ouvert » ou « développeur non identifié »
3. macOS — le script s'arrête avec un message
4. Windows — « Windows a protégé votre ordinateur »
5. Windows — le script s'arrête sans explication
6. Linux — ma distribution n'est pas prise en charge
7. Linux — dépendances et paquet refusé
8. Le premier lancement échoue au téléchargement d'Ollama
9. Le téléchargement d'Ollama a été interrompu
10. La machine n'a pas de réseau
11. « Erreur d'initialisation des données » — droits et espace disque
12. Le GPU n'est pas détecté
13. Désinstaller, et ce qui reste après

---

## Contenu

### 1. Avant de chercher : les trois causes qui expliquent la moitié des cas

Ouvrir la page par ce tri, avant toute liste de symptômes. Il évite la lecture intégrale.

- **Beaver n'est signé par aucun certificat de signature de code.** Le fichier de configuration de l'empaquetage déclare une signature ad hoc, `"signingIdentity": "-"` (`src-tauri/tauri.conf.json:70`), et la chaîne d'intégration continue ne comporte **aucune étape de signature ni de notarisation** (`.github/workflows/release.yml:135-161`). C'est la cause de tout ce que macOS et Windows affichent comme mise en garde.
- **Une seule architecture est distribuée par système.** Trois cibles seulement sont construites : `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc` (`.github/workflows/release.yml:139-158`). Un Mac Intel, un PC ARM, un Linux ARM ou un Linux hors famille Debian n'ont pas de fichier à installer.
- **Le premier lancement n'a besoin d'Internet que si l'utilisateur accepte d'installer le moteur Ollama** — le téléchargement est proposé par un bouton de l'écran d'installation, jamais déclenché tout seul, et se rattrape depuis **Réglages › Ollama**. Il vient des releases GitHub du projet Ollama (`src-tauri/src/services/ollama_manager/release_fetch.rs:10` et `:104-108`). Tout le reste de l'installation est local.

### 2. macOS — « Beaver ne peut pas être ouvert » ou « développeur non identifié »

**Symptôme** — au double-clic sur l'application, macOS refuse de l'ouvrir et parle d'un développeur non identifié, ou d'une application dont l'origine ne peut pas être vérifiée.

**Cause** — l'application porte une signature ad hoc (`src-tauri/tauri.conf.json:70`) et non un certificat Apple Developer ID, et elle n'est pas notarisée : aucune étape de signature n'existe dans la chaîne de publication (`.github/workflows/release.yml:135-161`). Quand le fichier `.dmg` est téléchargé par un navigateur, macOS lui pose son marquage de quarantaine, et Gatekeeper bloque.

**Résolution, dans cet ordre :**

1. **Réinstaller par le script**, qui est la voie recommandée : il télécharge par `curl` (`install.sh:6` et `:28-31`), ce qui ne pose pas le marquage de quarantaine. La commande complète est sur la page *Installation macOS* — ne pas la dupliquer ici, y renvoyer.
2. **Ou débloquer l'application déjà installée** : clic droit sur `Beaver` dans le dossier Applications, puis « Ouvrir », et confirmer dans la boîte de dialogue.
3. **Ou passer par les Réglages Système** : Confidentialité et sécurité, faire défiler jusqu'au message concernant Beaver, puis « Ouvrir quand même ».

**Ne pas publier de commande `xattr`.** Elle circule sur les forums, elle désactive une protection du système pour un dossier entier selon la façon dont elle est tapée, et les deux méthodes ci-dessus suffisent.

### 3. macOS — le script s'arrête avec un message

Le script d'installation n'affiche qu'une poignée de messages, tous en français, tous précédés d'une croix rouge. Le tableau complet est en section Tableaux. Les quatre cas qui remontent le plus :

- **« Système non pris en charge. »** — le script n'accepte que les couples `Darwin:arm64` et `Darwin:aarch64` (`install.sh:167-171`). Sur un Mac à processeur Intel, **il n'y a pas de solution** : aucun fichier d'installation Intel n'est construit (`.github/workflows/release.yml:139-144`). Le dire sans détour.
- **« Droits administrateur requis. »** — le script écrit dans le dossier d'installation par élévation de privilèges et exige `sudo` quand il n'est pas lancé en tant qu'administrateur (`install.sh:96-101`). Sur une machine où le compte n'est pas administrateur, choisir un dossier d'installation dans le dossier personnel quand le script le demande — il accepte tout chemin absolu, et développe `~` (`install.sh:102-106`, `:121-124`).
- **« Une application est déjà installée. Utilise sa mise à jour intégrée. »** — le script est réservé à une première installation. Il détecte `Beaver.app` **et** `CL-GO.app`, l'ancien nom, y compris sous forme de raccourci (`install.sh:107`, `:134-135`). Passer par la mise à jour intégrée à l'application, ou supprimer l'installation précédente d'abord.
- **« curl est requis. »** — le script exige l'outil du système, à l'emplacement `/usr/bin/curl` (`install.sh:6`, `:165`). Sur un macOS standard il est présent ; son absence signale une machine modifiée.

### 4. Windows — « Windows a protégé votre ordinateur »

**Symptôme** — au lancement de l'installeur `Beaver_X.Y.Z_x64-setup.exe`, Windows affiche un écran bleu « Windows a protégé votre ordinateur » et ne propose que « Ne pas exécuter ».

**Cause** — la même que sur macOS : aucun certificat de signature de code (`src-tauri/tauri.conf.json:70`, `.github/workflows/release.yml:135-161`). SmartScreen se déclenche sur un exécutable non signé qu'il ne connaît pas encore.

**Résolution** — cliquer sur **« Informations complémentaires »**, puis sur **« Exécuter quand même »**. Le lien est discret et beaucoup d'utilisateurs ne le voient pas : la page doit décrire précisément où il se trouve.

**À écrire dans la page, franchement** : un certificat de signature de code est payant et nominatif ; le projet n'en a pas. Cet avertissement est donc permanent, pas un incident.

**Point à ne pas confondre** avec le blocage de la protection anti-rançongiciel de Windows Defender au premier téléchargement de modèle : celui-là relève de la page *Installation Windows* et de *Dépannage › Ollama*.

### 5. Windows — le script s'arrête sans explication

**Symptôme** — le script PowerShell affiche une seule ligne, « ERREUR Installation impossible. », et s'arrête.

**Cause** — c'est **volontairement le seul message d'erreur du script** : tous les contrôles échouent sur la même sortie générique (`install.ps1:9`, `:182-186`). Rien n'est affiché sur la nature du contrôle qui a échoué.

Les contrôles qui peuvent produire ce message, vérifiés dans le code :

- **architecture différente de `AMD64`** (`install.ps1:141`) — pas de version ARM, aucune solution ;
- **dossier d'installation refusé** : il doit commencer par une lettre de lecteur suivie de `:\`, faire au plus 1024 caractères, ne contenir ni `*` `?` `<` `>` `|` `"`, ni caractère de contrôle, ni segment `..`, ni deux-points au-delà des trois premiers caractères (`install.ps1:133-139`) ;
- **empreinte ou taille du fichier téléchargé non conformes** au manifeste de la release (`install.ps1:159-166`) ;
- **l'installeur silencieux retourne un code d'erreur**, ou `cl-go-dash.exe` est absent du dossier d'installation à la fin, ou c'est un lien et non un vrai fichier (`install.ps1:176-179`).

**Résolution pas à pas :**

1. Relancer la commande une fois : un téléchargement interrompu produit le même message et réussit souvent au deuxième essai.
2. Réessayer avec le dossier proposé par défaut, `%LOCALAPPDATA%\Beaver` (`install.ps1:167-173`), plutôt qu'un dossier personnalisé — c'est la cause la plus fréquente et la plus silencieuse.
3. Si l'échec persiste, **installer par l'installeur** téléchargé depuis la page des releases GitHub, qui affiche ses propres erreurs.

**Bon à savoir, et absent des pages d'installation** : l'installeur est configuré en mode « utilisateur courant » (`"installMode": "currentUser"`, `src-tauri/tauri.conf.json:64-67`). Il n'a **pas** besoin de droits administrateur, et il installe pour le compte Windows en cours seulement.

### 6. Linux — ma distribution n'est pas prise en charge

**Symptôme** — le script affiche « Système non pris en charge. », ou bien il n'y a aucun paquet correspondant à la distribution sur la page des releases.

**Cause, à énoncer sans ambiguïté** — Beaver ne distribue qu'un paquet Debian pour l'architecture x64. Le script n'accepte que `Linux:x86_64` et `Linux:amd64` (`install.sh:167-171`), ne construit que le nom de fichier `_amd64.deb` (`install.sh:169`), vérifie le paquet avec `dpkg-deb` et `dpkg-query` (`install.sh:108`, `:146-151`) et l'installe avec `apt-get install -y` (`install.sh:153`). **Aucun autre gestionnaire de paquets n'est appelé.**

**Résolution** — aucune sur Fedora, RHEL, Arch, openSUSE ou toute distribution hors famille Debian : il n'existe pas de paquet pour elles. Les options honnêtes à présenter :

- utiliser une distribution de la famille Debian (Debian, Ubuntu et dérivées) ;
- ou attendre, et suivre le dépôt.

⚠️ **Le document interne `CROSS-PLATFORM.md` annonce un support Fedora/RHEL par `dnf` qui n'existe pas dans le code.** Ne rien en reprendre. Ce constat est déjà porté par les briefs *Prérequis* et *Installation Linux* ; il est répété ici parce que c'est exactement la question que pose un utilisateur bloqué.

### 7. Linux — dépendances et paquet refusé

Deux symptômes distincts, souvent confondus.

**Symptôme A — « Paquet d'installation invalide. »**

*Cause* — avant d'installer, le script contrôle six métadonnées du paquet : `Package` vaut `beaver`, `Architecture` vaut `amd64`, `Version` est identique à celle annoncée par la release, et `Provides`, `Conflicts`, `Replaces` valent tous `cl-go` (`install.sh:146-152`). Les trois derniers champs viennent de la configuration d'empaquetage (`src-tauri/tauri.conf.json:56-61`). Un seul écart fait échouer l'installation.

*Résolution* — ce n'est **pas** un problème de machine : cela signifie que le fichier publié ne correspond pas à ce que le script attend. Signaler le problème au projet, en indiquant la version. Ne pas contourner en installant le paquet à la main.

**Symptôme B — l'application ne démarre pas après une installation manuelle par `dpkg -i`**

*Cause* — `dpkg` n'installe pas les dépendances manquantes ; il laisse un paquet configuré à moitié. Le script, lui, passe par `apt-get` (`install.sh:153`), qui les résout.

*Résolution* — dans un terminal :

```bash
sudo apt install -f
```

ou, mieux, réinstaller proprement :

```bash
sudo apt install ./Beaver_X.Y.Z_amd64.deb
```

en remplaçant `X.Y.Z` par le numéro de version du fichier téléchargé.

**Symptôme C — installation réussie, mais rien ne se lance**

*Cause possible et vérifiable* — le script contrôle après installation que `/usr/bin/cl-go-dash` existe et n'est pas un lien symbolique (`install.sh:154`). **Le binaire ne s'appelle pas `beaver`** : quelqu'un qui tape `beaver` dans un terminal n'obtient rien.

*Résolution* — lancer depuis le menu des applications, ou taper `cl-go-dash` dans un terminal.

### 8. Le premier lancement échoue au téléchargement d'Ollama

**Symptôme** — l'écran **« Configuration d'Ollama »** affiche un bloc d'erreur : le titre **« L'installation a échoué. Détails ci-dessous. »** suivi d'une phrase (`src/components/ollama/ollama-setup-screen.tsx:156-165`, libellés `src/i18n/fr.json:1549-1568`).

C'est la phrase du dessous qui identifie la cause. **Le site doit publier le tableau complet de ces phrases** — il est en section Tableaux, avec la cause réelle de chacune. Sans lui, l'utilisateur n'a qu'une phrase générique et aucune piste.

**Résolution commune à tous ces cas, dans cet ordre :**

1. **Cliquer de nouveau sur « Télécharger Ollama ».** Un échec réseau ne laisse aucun fichier partiel derrière lui : le fichier en cours d'écriture est supprimé dès qu'une erreur survient (`src-tauri/src/services/ollama_manager/download_stream.rs:112-116`).
2. **Cliquer sur « Passer »** pour entrer dans l'application tout de suite. Beaver reste pleinement utilisable avec des modèles distants : la seule chose qui manque est le moteur local.
3. **Reprendre plus tard** dans **Réglages › Ollama** : tant que le moteur n'est pas installé, cet onglet affiche exactement le même écran « Configuration d'Ollama » (`src/components/ollama/ollama-tab.tsx:56-67`).

**Point important à écrire** : passer l'étape est mémorisé (`src/lib/ollama-setup-gate.ts:1-18`, appliqué en `src/hooks/use-startup-gate.ts:39-50`). L'écran ne réapparaîtra **pas** au démarrage suivant. Quelqu'un qui a cliqué « Passer » par erreur doit savoir qu'il le retrouve dans Réglages › Ollama, et nulle part ailleurs.

### 9. Le téléchargement d'Ollama a été interrompu

**Symptôme** — l'ordinateur s'est éteint, l'application a été fermée, ou le bouton « Annuler » a été utilisé pendant la barre de progression. Au lancement suivant, Réglages › Ollama affiche soit une progression, soit un message d'erreur accompagné d'un bouton **« Réessayer »**.

**Cause** — l'installation d'Ollama est une opération en plusieurs phases, annonçées à l'écran dans cet ordre : préparation, téléchargement, vérification, installation, validation, finalisation, démarrage (`src-tauri/src/commands/ollama_setup.rs:108-121`, libellés `src/i18n/fr.json:1558-1566`). Chaque phase travaille dans un dossier distinct **à côté** de l'installation active, jamais dessus : `ollama-bundle-install-staging`, `ollama-bundle-install-staging-archives`, `ollama-bundle-backup`, et leurs dossiers de mise au rebut (`src-tauri/src/services/paths/ollama.rs:38-56`). L'installation utilisable, elle, s'appelle `ollama-bundle` et n'est remplacée qu'à la toute fin.

**Résolution :**

1. **Redémarrer Beaver.** Une reprise s'exécute automatiquement au démarrage, avant toute tentative de lancer le moteur (`src-tauri/src/runtime_startup.rs:44-49`). Dans la plupart des cas il n'y a rien d'autre à faire.
2. **Si l'écran affiche un message avec un bouton « Réessayer »** (Réglages › Ollama), cliquer dessus : il relance la réparation (`src/components/ollama/ollama-tab.tsx:71-79` et `:141-143`).
3. **Si le message est « La réparation d'Ollama est temporairement impossible. Réessaie. »**, attendre quelques secondes et réessayer : ce message signifie qu'une autre opération occupe le dossier.

**À dire clairement, c'est rassurant et c'est vrai** : une interruption ne laisse jamais une installation à moitié remplacée. Le nouvel état est construit à côté de l'ancien et ne prend sa place qu'une fois complet ; les restes d'une tentative abandonnée sont mis au rebut puis supprimés à la reprise (`src-tauri/src/services/ollama_manager/cleanup.rs:25-46`, `archive_recovery.rs:18-46`).

### 10. La machine n'a pas de réseau

**Symptôme** — l'installation de l'application réussit, mais l'écran « Configuration d'Ollama » échoue immédiatement avec **« Le téléchargement d'Ollama a échoué. Vérifie la connexion puis réessaie. »**

**Cause** — Ollama n'est pas embarqué dans l'application. Beaver interroge la dernière release publiée du projet Ollama sur `github.com` (`src-tauri/src/services/ollama_manager/release_fetch.rs:10`, délai de 10 secondes en `:13-18`), puis lit le fichier d'empreintes `sha256sum.txt` de cette release et interroge la taille de chaque archive (`release_fetch.rs:61-102`, délai de 15 secondes). Sans réseau, ces trois requêtes échouent.

**Ce qui fonctionne quand même**, et qu'il faut énoncer avant la résolution :

- l'application **s'installe et démarre** sans réseau ; seule l'étape Ollama échoue ;
- le dossier de données et toute sa structure sont créés localement (`src-tauri/src/storage_migration.rs:65-119`) ;
- le moteur de prévision local est installé depuis les fichiers livrés avec l'application, sans réseau (`storage_migration.rs:60`).

**Résolution :**

1. Cliquer sur **« Passer »** et utiliser l'application.
2. Se connecter à un réseau, puis reprendre dans **Réglages › Ollama**.
3. **Sur un réseau d'entreprise ou derrière un portail captif** : vérifier que `github.com` est joignable. Beaver n'accepte que des adresses `https` sur `github.com`, sans port, sans identifiant, sans paramètre (`src-tauri/src/services/ollama_manager/release_source.rs:11` et `:63-77`) — un proxy qui réécrit l'adresse ou renvoie une page d'authentification fait échouer l'étape.

**Encadré à prévoir** : si un moteur Ollama est déjà installé séparément sur la machine et qu'il tourne, Beaver le réutilise et ne télécharge rien. Le renvoi va vers *Premier lancement*, qui documente cette détection.

### 11. « Erreur d'initialisation des données » — droits et espace disque

**Symptôme** — l'application ne démarre pas, ou s'arrête au tout début du démarrage. Le message technique est **« Erreur d'initialisation des données »** (`src-tauri/src/storage_migration.rs:126-128`).

**Cause** — Beaver crée son dossier de données et sa structure au tout début du démarrage, avant l'affichage de la fenêtre (`src-tauri/src/app_build.rs:125`). Cette étape échoue si le dossier ne peut pas être créé, si les droits ne peuvent pas être posés, ou si l'écriture d'un fichier échoue.

**Le dossier de données est le même sur les trois systèmes**, `~/.local/share/cl-go-dash/` (`src-tauri/src/services/paths.rs:10-14`) — soit `C:\Users\<votre compte>\.local\share\cl-go-dash\` sur Windows. **Ce n'est pas `%APPDATA%`** : la page doit le dire, un utilisateur Windows cherchera au mauvais endroit.

Beaver y pose des droits stricts : **dossiers en `0700`, fichiers en `0600`** sur macOS et Linux (`src-tauri/src/services/private_store.rs:168-174`), et une liste de contrôle d'accès restreinte à l'utilisateur sur Windows (`private_store.rs:176-179`). Ces droits sont vérifiés et réparés à chaque démarrage sur le dossier racine, `agent-sessions/`, `forecast-notes/`, `logs/`, et sur six fichiers sensibles dont le coffre `secrets.enc` (`private_store.rs:91-115`).

**Résolution pas à pas :**

1. **Vérifier l'espace libre du disque de démarrage.** Il faut de la place pour le moteur Ollama, ses modèles et les conversations. Un disque plein produit exactement ce message.
2. **Vérifier que le dossier personnel est accessible en écriture.** Sur macOS et Linux, ouvrir un terminal et taper `ls -ld ~/.local/share/cl-go-dash` : le dossier doit appartenir à votre compte.
3. **Cas particulier du dossier personnel sur un disque réseau ou un volume chiffré** : les droits `0700`/`0600` ne peuvent pas toujours y être posés. Déplacer le dossier personnel n'est pas une option raisonnable pour un utilisateur ; c'est un cas à signaler au projet.
4. **Si le dossier de données a été copié depuis une autre machine ou restauré depuis une sauvegarde**, ses propriétaires peuvent être faux. Sur macOS et Linux : `sudo chown -R "$USER" ~/.local/share/cl-go-dash`.

⚠️ **Beaver ne vérifie pas l'espace disque disponible avant de télécharger Ollama.** Le seul contrôle d'espace libre du dépôt concerne l'installation des extensions (`src-tauri/src/services/extensions/install_jobs/disk_control.rs:4`). Un disque plein pendant le téléchargement se manifeste donc par un message générique d'échec, pas par un message d'espace insuffisant. Le dire dans un encadré.

### 12. Le GPU n'est pas détecté

**Symptôme** — l'installation réussit, mais les modèles locaux sont très lents, et Beaver ne prévient jamais qu'un modèle est trop gros pour la carte graphique.

**Cause, en deux mécanismes distincts qu'il ne faut pas confondre :**

**a) La détection du constructeur**, utilisée uniquement sous Linux pour choisir l'archive à télécharger. Beaver lit les identifiants constructeur dans `/sys/class/drm/*/device/vendor` : `0x1002` pour AMD, `0x10de` pour NVIDIA, `0x8086` pour Intel (`src-tauri/src/services/gpu_detect.rs:49-68`). Une carte AMD reconnue déclenche le téléchargement d'une archive supplémentaire, `ollama-linux-amd64-rocm.tar.zst` (`src-tauri/src/services/ollama_manager/release_source.rs:20-26`). **Si la détection échoue, seule l'archive de base est téléchargée** et l'accélération AMD n'est pas installée. Sur macOS la détection lit la marque du processeur (`gpu_detect.rs:31-47`), sur Windows elle interroge la liste des cartes vidéo par PowerShell (`gpu_detect.rs:70-97`).

**b) La mesure de la mémoire graphique**, qui décide de la taille de contexte par défaut. Trois paliers : **32 768 jetons** au-dessus de 24 000 Mo, **24 576 jetons** au-dessus de 12 000 Mo, **8 192 jetons** en dessous — et **8 192 également quand la mesure échoue** (`src-tauri/src/services/gpu_vram.rs:44-48` et `:138-144`). Conséquence directe : **une mémoire non mesurée ne se voit pas, elle se traduit par un contexte réduit au minimum.**

Deuxième conséquence, plus discrète : l'avertissement « ce modèle ne tient pas dans votre mémoire » **ne s'affiche jamais** quand la mesure a échoué, parce que le contrôle répond alors « ça tient » (`src-tauri/src/commands/ollama_setup.rs:203-210`).

**Résolution :**

1. **Sous Linux avec une carte AMD** : vérifier que la carte est bien vue par le système avec `ls /sys/class/drm/`. Si aucune entrée n'y correspond, le pilote graphique n'est pas chargé — c'est un problème de système, pas de Beaver.
2. **Sous Windows avec une carte NVIDIA** : le pilote doit être installé ; c'est lui qui fournit le nom de la carte à la détection.
3. **Sous Linux avec une carte NVIDIA** : pilote **531 ou supérieur** (voir *Prérequis*). CUDA est fourni dans l'archive Ollama, rien d'autre à installer.
4. **Si le contexte reste à 8 192 jetons** alors que la machine a plus de 12 Go de mémoire graphique, c'est que la mesure a échoué. Ce n'est pas réglable depuis l'application : signaler le problème avec le modèle exact de la carte.

⚠️ **Sur un Mac à processeur Intel, la mémoire n'est jamais mesurée** : la mesure sort immédiatement hors architecture Apple Silicon (`src-tauri/src/services/gpu_vram/macos.rs:3-6`), et le contexte tombe donc au palier de 8 192 jetons. Le cas est théorique — un Mac Intel ne peut pas installer Beaver (section 3) — mais il faut le savoir avant de promettre quoi que ce soit sur cette plateforme.

**Le moteur Ollama n'écrit plus aucun fichier de traces.** Sa sortie est envoyée vers `/dev/null` sur macOS et Linux (`src-tauri/src/services/ollama_manager/spawn_gate_unix.rs:63-66`). Ne pas publier de conseil « consultez le journal du moteur » : ce fichier n'existe pas.

### 13. Désinstaller, et ce qui reste après

**Symptôme** — « J'ai supprimé l'application et il reste des choses », ou « je réinstalle et il retrouve mes anciennes conversations ».

**Cause** — c'est le comportement voulu : **le dossier de données est indépendant de l'application installée**. Aucune procédure de désinstallation n'est documentée dans le dépôt et **aucun code ne supprime le dossier de données** ; c'est un choix à assumer sur le site, pas un oubli à masquer.

**Retirer l'application :**

| Système | Comment |
|---|---|
| macOS | Jeter `Beaver` du dossier d'installation choisi (`/Applications` par défaut, `install.sh:118-124`) |
| Linux | `sudo apt remove beaver` — le nom du paquet est `beaver` (`install.sh:146`) |
| Windows | Paramètres › Applications, ou le désinstalleur posé dans le dossier d'installation. L'installation est faite pour l'utilisateur courant (`src-tauri/tauri.conf.json:64-67`) |

**Ce qui reste sur le disque après cette suppression, et qu'il faut lister nommément :**

1. **Le dossier de données**, `~/.local/share/cl-go-dash/` (`src-tauri/src/services/paths.rs:10-14`) : conversations, mémoire, réglages, projets, coffre chiffré `secrets.enc`, et l'installation d'Ollama dans `ollama-bundle` (`src-tauri/src/services/paths/ollama.rs:38`).
2. **Les modèles Ollama téléchargés**, dans `~/.ollama/models` (`src-tauri/src/services/ollama_manager/spawn_profile_paths.rs:33`). C'est de loin le plus volumineux, et il est **en dehors** du dossier de données de Beaver.
3. **Les entrées du gestionnaire de secrets du système** : la clé maîtresse du coffre, sous le service `cl-go-dash` et le compte `master-key` (`src-tauri/src/services/vault.rs:16-18`), plus une entrée par fournisseur configuré et une entrée `brave_api_key` (`vault.rs:160`, `:169`). Trousseau d'accès sur macOS, Secret Service sur Linux, gestionnaire d'identification protégé par DPAPI sur Windows.

**Résolution — désinstallation complète, dans cet ordre :**

1. Retirer l'application (tableau ci-dessus).
2. Supprimer le dossier de données. Sur macOS et Linux : `rm -rf ~/.local/share/cl-go-dash`. Sur Windows, supprimer le dossier `.local\share\cl-go-dash` dans le dossier de l'utilisateur.
3. Supprimer les modèles si l'on ne s'en sert plus ailleurs : `rm -rf ~/.ollama`. **Attention** : ce dossier est partagé avec toute autre installation d'Ollama sur la machine.
4. Supprimer les entrées du gestionnaire de secrets, à la main. Chercher `cl-go-dash` dans Trousseau d'accès (macOS), dans le gestionnaire de mots de passe du bureau (Linux), dans le Gestionnaire d'identification (Windows).

**Encadré indispensable** : sauter l'étape 4 laisse la **clé maîtresse du coffre** dans le gestionnaire de secrets du système. Elle ne déchiffre rien sans le fichier `secrets.enc` supprimé à l'étape 2, mais elle n'a plus aucune raison d'être là.

**Réinstaller après une suppression partielle** : les conversations, réglages et clés sont repris tels quels par la nouvelle installation. C'est voulu, et c'est ce qui permet aux mises à jour de ne rien perdre.

---

## Tableaux

### Tableau — Les messages du script d'installation macOS et Linux

Tous vérifiés dans `install.sh`. Les citer mot pour mot sur le site.

| Message affiché | Ligne | Cause | Résolution |
|---|---|---|---|
| « Système non pris en charge. » | `:170` | Architecture hors `Darwin:arm64`, `Darwin:aarch64`, `Linux:x86_64`, `Linux:amd64` | Aucune : pas de fichier d'installation pour cette machine |
| « curl est requis. » | `:165` | `/usr/bin/curl` absent ou non exécutable | Installer `curl` (Linux) ; sur macOS, machine modifiée |
| « Droits administrateur requis. » | `:98` | Pas de `sudo` et compte non administrateur | Choisir un dossier d'installation dans le dossier personnel |
| « Impossible de récupérer la version. » | `:175` | L'API GitHub n'a pas répondu | Vérifier la connexion, le proxy, le pare-feu |
| « Version de Beaver invalide. » / « Version de Beaver incomplète. » | `:176`, `:181` | La release publiée ne contient pas les fichiers attendus | Signaler le problème ; ne pas contourner |
| « Manifeste de mise à jour invalide. » | `:184`, `:186` | Le fichier d'empreintes de la release est absent ou malformé | Signaler le problème |
| « Téléchargement impossible. » | `:190` | Le téléchargement a échoué ou dépassé 1800 secondes | Relancer ; vérifier le débit |
| « Téléchargement invalide. » | `:191`, `:192`, `:193` | Taille ou empreinte SHA-256 différente de celle annoncée | Relancer une fois ; si ça persiste, signaler |
| « Une application est déjà installée. Utilise sa mise à jour intégrée. » | `:135`, `:145` | `Beaver.app`, `CL-GO.app`, ou le paquet `beaver` ou `cl-go` déjà présent | Mise à jour intégrée, ou retirer l'ancienne installation |
| « Répertoire d'installation invalide. » | `:124` | Chemin non absolu, trop long, avec caractère de contrôle ou segment `..` | Saisir un chemin absolu simple |
| « Paquet d'installation invalide. » | `:152` | Une des six métadonnées du `.deb` ne correspond pas | Signaler : le fichier publié ne correspond pas au script |
| « Installation impossible. » | `:125`, `:127`, `:128`, `:133`, `:138`, `:153`, `:154` | Montage, vérification du contenu, copie ou installation du paquet en échec | Relancer ; vérifier espace disque et droits sur le dossier choisi |

### Tableau — Les messages d'erreur d'Ollama à l'installation

Ces phrases s'affichent sous « L'installation a échoué. Détails ci-dessous. ». Textes vérifiés dans `src/i18n/fr.json:687-708`, codes dans `src-tauri/src/services/ollama_manager/error.rs:60-82`.

| Phrase affichée | Ce qui s'est réellement passé | Ce qu'il faut faire |
|---|---|---|
| « Le téléchargement d'Ollama a échoué. Vérifie la connexion puis réessaie. » | Réseau injoignable, réponse inattendue, ou taille reçue différente de celle annoncée | Relancer ; vérifier réseau et proxy |
| « Le fichier téléchargé n'a pas passé la vérification d'intégrité. » | L'empreinte SHA-256 du fichier reçu ne correspond pas à celle publiée par le projet Ollama | Relancer une fois ; si ça persiste, suspecter un proxy qui modifie les téléchargements |
| « L'extraction d'Ollama a échoué. » | L'archive n'a pas pu être décompressée | Vérifier l'espace disque, puis relancer |
| « Le stockage d'Ollama est inaccessible. Vérifie l'espace libre et les autorisations. » | Écriture impossible dans le dossier de données | Voir la section « Erreur d'initialisation des données » |
| « L'installation d'Ollama est endommagée ou incompatible. » | Le contenu installé n'est pas celui attendu | Relancer l'installation depuis Réglages › Ollama |
| « L'installation d'Ollama doit être réparée avant de continuer. » | Une installation précédente s'est interrompue à un point qui exige une reprise explicite | Bouton « Réessayer » dans Réglages › Ollama |
| « La réparation d'Ollama est temporairement impossible. Réessaie. » | Une autre opération occupe le dossier | Attendre quelques secondes, réessayer |
| « Une autre opération Ollama est déjà en cours. » | Deux installations lancées en même temps | Attendre la fin de la première |
| « L'opération Ollama a été annulée. » | Bouton « Annuler » utilisé | Relancer quand vous voulez |
| « Ollama n'a pas répondu avant la fin du délai. » | Le moteur a été installé mais n'a pas répondu dans les **45 secondes** (`src-tauri/src/commands/ollama_setup.rs:166-186`) | Relancer ; sur une machine lente c'est fréquent au premier démarrage |
| « Ollama n'a pas pu démarrer. » | Le processus n'a pas pu être lancé | Redémarrer Beaver ; voir *Dépannage › Ollama* |
| « Ollama n'est pas installé par Beaver. » | Aucune installation présente | Lancer le téléchargement depuis Réglages › Ollama |

### Tableau — Où ça bloque, et est-ce que c'est réparable

| Symptôme | Réparable par l'utilisateur | Issue |
|---|---|---|
| Mac Intel, PC ARM, Linux ARM | **Non** | Aucun fichier d'installation construit pour ces machines |
| Distribution hors famille Debian | **Non** | Seul un paquet `.deb` x64 est publié |
| Avertissement Gatekeeper | Oui | Clic droit › Ouvrir, ou installer par le script |
| Avertissement SmartScreen | Oui | Informations complémentaires › Exécuter quand même |
| Téléchargement d'Ollama échoué | Oui | Relancer, ou passer et reprendre plus tard |
| Installation d'Ollama interrompue | Oui | Redémarrer Beaver, puis « Réessayer » si demandé |
| « Erreur d'initialisation des données » | Souvent | Espace disque, droits sur le dossier personnel |
| Mémoire graphique non mesurée | **Non** | Contexte réduit à 8 192 jetons, aucun avertissement de taille de modèle |
| « Paquet d'installation invalide. » | **Non** | À signaler au projet |

---

## Encadrés

**Encadré « Aucune signature, et c'est permanent »** — avertissement, en tête de page.
> Beaver n'est signé par aucun certificat de signature de code. macOS et Windows afficheront donc une mise en garde à la première ouverture. Ce n'est pas un incident : c'est le cas de tous les logiciels libres distribués sans certificat, qui est payant et nominatif. Les deux sections ci-dessous expliquent quoi cliquer.

**Encadré « L'espace disque n'est pas vérifié »** — avertissement, section 11.
> Beaver ne contrôle pas l'espace libre avant de télécharger le moteur local. Sur un disque presque plein, l'échec se présente comme un problème de téléchargement ou d'extraction. Vérifiez votre espace libre avant de relancer.

**Encadré « Passer n'est pas définitif — mais ça ne revient pas tout seul »** — information, section 8.
> Si vous passez l'installation du moteur local, l'écran ne réapparaîtra pas au démarrage suivant. Vous le retrouvez à tout moment dans **Réglages › Ollama**.

**Encadré « Vos données survivent à la désinstallation »** — avertissement, section 13.
> Supprimer l'application ne supprime ni vos conversations, ni vos clés, ni vos modèles. Ils vivent dans un dossier séparé. La marche complète est décrite ci-dessous ; l'étape du gestionnaire de secrets est celle qu'on oublie.

**Encadré « Le moteur local n'écrit pas de journal »** — information, section 12.
> Contrairement à ce que suggèrent d'anciennes documentations, le moteur Ollama lancé par Beaver n'écrit aucun fichier de traces. Il n'y a pas de journal à consulter en cas de lenteur.

---

## Pièges et erreurs fréquentes

**Chercher le dossier de données dans `%APPDATA%` sous Windows.** Il est dans `C:\Users\<votre compte>\.local\share\cl-go-dash\` (`src-tauri/src/services/paths.rs:10-14`), un emplacement inhabituel sur Windows, hérité de l'unification des chemins entre les trois systèmes.

**Taper `beaver` dans un terminal sous Linux.** Le binaire s'appelle `cl-go-dash` (`install.sh:154`). Le paquet, lui, s'appelle bien `beaver`.

**Utiliser le script d'installation pour mettre à jour.** Il refuse, par conception (`install.sh:134-135`, `:144-145`). La mise à jour passe par l'application.

**Croire que « Passer » désinstalle ou refuse Ollama définitivement.** Ça n'écrit qu'un réglage, et l'écran reste accessible dans Réglages › Ollama (`src/lib/ollama-setup-gate.ts:16-18`, `src/components/ollama/ollama-tab.tsx:56-67`).

**Installer le `.deb` avec `dpkg -i`.** Les dépendances ne sont pas résolues et l'application ne démarre pas. Utiliser `apt`.

**Supprimer `~/.ollama` sans réfléchir.** Ce dossier est partagé avec toute autre installation d'Ollama sur la machine, y compris une installation faite avant Beaver.

**Lire un message d'échec Windows comme une piste.** Le script PowerShell n'a qu'un seul message pour tous ses contrôles (`install.ps1:9`, `:182-186`). Il ne dit jamais lequel a échoué.

---

## Renvois

- *Installation macOS*, *Installation Windows*, *Installation Linux* — les procédures nominales, à ne pas répéter ici
- *Prérequis* — compatibilité matérielle, architectures, dépendances
- *Premier lancement* — ce qui est créé sur le disque, la détection d'un Ollama déjà présent
- *Parcours d'accueil* — l'étape Ollama et la façon de la passer
- *Mise à jour* — les échecs de mise à jour, qui ne relèvent pas de cette page
- *Dépannage › Ollama* — moteur indisponible, port occupé, modèle trop lourd après une installation réussie
- *Référence › Stockage local* — le contenu complet du dossier de données
- *Modèles › Matériel et VRAM* — quelle taille de modèle pour quelle machine

---

## Points à confirmer

- **La procédure de désinstallation n'est validée sur aucune des trois plateformes.** Elle est reconstituée ici à partir des chemins et des noms trouvés dans le code — dossier de données, `~/.ollama`, entrées de gestionnaire de secrets — mais personne ne l'a exécutée de bout en bout. **À faire avant publication**, sur les trois systèmes, en vérifiant qu'il ne reste rien d'autre. C'est la section la plus coûteuse en cas d'erreur : elle demande à l'utilisateur de supprimer des fichiers.
- **Le nom exact de l'entrée dans le Gestionnaire d'identification Windows et dans le magasin de secrets Linux.** Le service est `cl-go-dash` et le compte `master-key` côté code (`vault.rs:16-18`), mais la façon dont chaque système les affiche n'a pas été observée. Sans ce libellé, l'étape 4 de la désinstallation n'est pas exécutable par quelqu'un qui ne programme pas.
- **Ce que voit l'utilisateur quand l'initialisation du stockage échoue.** Le message technique est « Erreur d'initialisation des données » (`storage_migration.rs:126-128`) et il remonte comme erreur de démarrage (`app_build.rs:125`), mais il n'a pas été observé à l'écran : boîte de dialogue, fenêtre vide, ou rien du tout. **À provoquer et à capturer** — c'est le seul écran de cette page qu'un utilisateur peut rencontrer sans jamais avoir vu l'application fonctionner.
- **Le contournement de Gatekeeper sur les versions récentes de macOS.** Le clic droit puis « Ouvrir » est décrit ici d'après l'usage établi ; Apple a modifié ce parcours sur les versions récentes du système, où le passage par Réglages Système devient obligatoire. **À vérifier sur la version de macOS visée**, et à écrire pour celle-là, pas pour les deux à la fois.
- **La version minimale de macOS.** `tauri.conf.json` ne définit pas `minimumSystemVersion` ; la valeur par défaut de Tauri s'applique. Un prérequis sans numéro reste incomplet, et cette page ne peut pas dire « votre système est trop ancien » tant qu'il n'existe pas.
- **WebView2 et le runtime Visual C++ sous Windows.** Toujours non vérifié sur une machine vierge — le brief *Installation Windows* le signale déjà. Tant que ce n'est pas tranché, le symptôme « l'application ne démarre pas après installation, sans message » n'a pas de résolution publiable et n'apparaît volontairement pas dans le corps de cette page.
- **Le comportement d'une réinstallation par script par-dessus une installation existante sous Windows.** Les scripts macOS et Linux refusent explicitement ; `install.ps1` ne fait pas ce contrôle et laisse l'installeur décider. À vérifier et à harmoniser.
- **Le comportement sur un dossier personnel monté en réseau.** Les droits `0700`/`0600` (`private_store.rs:168-174`) ne sont pas toujours applicables sur ces volumes. La section 11 le mentionne comme cas à signaler, faute d'avoir été reproduit.
- **La taille du téléchargement d'Ollama par système**, à annoncer avant de lancer l'opération. Toujours inconnue — le point est déjà ouvert dans *Premier lancement*. Il est plus gênant ici : quelqu'un en connexion limitée doit pouvoir décider avant de cliquer.
- **Deux briefs de la section 02 sont à corriger, et l'écart a été constaté en écrivant cette page :**
  - *Premier lancement*, section 5, annonce une « taille minimale de 10 Mo » et un « rejet si le type de contenu est `text/html` » pour le téléchargement d'Ollama. **Ces deux contrôles n'existent pas dans le code actuel.** Les contrôles réels sont : taille exacte obtenue par une requête préalable et vérifiée à l'octet près (`download_stream.rs:99-105` et `:161`), plafond de **3 Gio** (`download.rs:10`), délai de **1800 secondes** (`download_stream.rs:12`), et empreinte SHA-256 comparée en temps constant à celle publiée par le projet Ollama (`download.rs:126-148`).
  - *Installation Linux*, tableau des pièges, conseille de consulter `~/.local/share/cl-go-dash/logs/ollama-sidecar.log`. **Ce fichier n'est plus écrit** : la sortie du moteur va vers `/dev/null` (`spawn_gate_unix.rs:63-66`).
- **Aucun écran de cette page n'a été observé en fonctionnement.** Les libellés cités viennent des fichiers de traduction et des composants ; leur disposition, leur visibilité et le comportement du bloc d'erreur de l'écran « Configuration d'Ollama » relèvent de la passe d'interface de fin de parcours.
