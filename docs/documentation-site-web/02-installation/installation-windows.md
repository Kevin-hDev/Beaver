# Installation sur Windows

**Emplacement site** — Démarrage › Installation › Windows
**Répond à** — « Comment j'installe Beaver sur Windows, et pourquoi SmartScreen et Defender réagissent ? »
**Sources** — `install.ps1` (lignes 88-182), `CROSS-PLATFORM.md`, `src-tauri/tauri.conf.json` (bloc `nsis`)
**Vérification** — Vérifié dans le code : toutes les étapes sont lues dans `install.ps1`

---

## Plan de page proposé

1. Méthode recommandée — le script d'installation
2. Ce que fait le script
3. Méthode alternative — l'installeur
4. SmartScreen et l'absence de signature
5. Windows Defender et l'accès contrôlé aux dossiers
6. Choisir un autre dossier d'installation
7. Mettre à jour
8. Désinstaller

---

## Contenu

### 1. Méthode recommandée — le script d'installation

Commande PowerShell à publier telle quelle :

```powershell
$installer = Join-Path ([IO.Path]::GetTempPath()) "beaver-install-$([Guid]::NewGuid().ToString('N')).ps1"
try {
  Invoke-WebRequest https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 -OutFile $installer -ErrorAction Stop
  & $installer
} finally {
  Remove-Item -LiteralPath $installer -Force -ErrorAction SilentlyContinue
}
```

Préciser que la commande se colle dans **PowerShell**, pas dans l'invite de commandes classique.

### 2. Ce que fait le script

1. Vérifie que l'architecture du processeur est **AMD64**. Toute autre valeur arrête l'installation.
2. Force **TLS 1.2** et **désactive le suivi automatique des redirections** : chaque redirection est examinée avant d'être suivie.
3. Interroge l'API GitHub Releases, réponse plafonnée à **512 Kio**.
4. Exige l'asset nommé exactement `Beaver_<version>_x64-setup.exe`.
5. Télécharge `update-manifest.json`, plafonné à **64 Kio**, et vérifie que sa taille correspond à celle annoncée.
6. Vérifie que le manifeste porte bien la version attendue, et que chaque entrée d'asset a un nom conforme, une empreinte **SHA-256** de 64 caractères hexadécimaux et une taille.
7. Télécharge l'installeur, avec un délai maximum de **1800 secondes**, et vérifie que la taille reçue correspond exactement à celle attendue.
8. Calcule le **SHA-256** du fichier et le compare à celui du manifeste. En cas d'écart, l'installation s'arrête.
9. Propose le dossier d'installation, `%LOCALAPPDATA%\Beaver` par défaut.
10. Lance l'installeur NSIS en mode silencieux : `/S /D=<dossier>`, fenêtre masquée.
11. Vérifie que `cl-go-dash.exe` existe dans le dossier d'installation et **qu'il ne s'agit pas d'un point d'analyse** — une protection contre la substitution par lien.

### 3. Méthode alternative — l'installeur

- L'installeur `Beaver_X.Y.Z_x64-setup.exe` est attaché à chaque release GitHub.
- Double-clic et parcours classique.
- **Inconvénient** : SmartScreen s'interpose sur un exécutable non signé téléchargé par navigateur, et la vérification d'empreinte est à faire soi-même.

### 4. SmartScreen et l'absence de signature

À annoncer avant que l'utilisateur le rencontre :

- **L'application n'est pas signée par un certificat de signature de code.**
- Windows SmartScreen affiche donc un écran bleu « Windows a protégé votre ordinateur » à l'exécution de l'installeur.
- Contournement : cliquer sur « Informations complémentaires », puis « Exécuter quand même ».
- Ce comportement est normal pour un logiciel libre non signé — un certificat de signature de code est payant et nominatif.

### 5. Windows Defender et l'accès contrôlé aux dossiers

Point spécifique à Windows, à ne pas omettre :

- Si la protection **anti-rançongiciel** (« Accès contrôlé aux dossiers ») est active, `ollama.exe` peut être bloqué au premier lancement quand il tente d'écrire les modèles dans `~/.ollama/models/`.
- Une notification Windows apparaît. L'utilisateur doit cliquer **« Autoriser »**.
- La demande ne se répète pas ensuite.

Sans cette explication, le symptôme observé est « le téléchargement du modèle échoue sans raison ».

### 6. Choisir un autre dossier d'installation

- Défaut : `%LOCALAPPDATA%\Beaver`.
- Un autre chemin peut être saisi, avec des contrôles stricts : il doit commencer par une lettre de lecteur suivie de `:\`, ne pas dépasser 1024 caractères, ne contenir aucun des caractères `* ? < > | "`, aucun caractère de contrôle, aucun segment `..`, et aucun `:` au-delà des trois premiers caractères.
- Un chemin non conforme arrête l'installation.

### 7. Mettre à jour

**Différence notable avec macOS et Linux** : le script Windows ne contient pas de refus explicite d'installation existante — les scripts macOS et Linux, eux, s'arrêtent si une installation est détectée. Sur Windows, l'installeur NSIS gère lui-même le remplacement.

Cela dit, la voie normale reste la **mise à jour intégrée** à l'application. Renvoyer vers *Mise à jour*.

Voir *Points à confirmer* : le comportement exact d'une réinstallation par script par-dessus une installation existante n'a pas été vérifié.

### 8. Désinstaller

Voir *Points à confirmer*.

Ce qui est certain :
- L'installeur NSIS pose un désinstalleur ; la suppression passe par Paramètres › Applications, ou par le désinstalleur du dossier d'installation.
- Les données restent dans `C:\Users\<utilisateur>\.local\share\cl-go-dash\`.
- La clé maîtresse du coffre reste protégée par DPAPI.

---

## Tableaux

### Tableau — Contrôles effectués par le script

| Étape | Contrôle |
|---|---|
| Architecture | `AMD64` exigé |
| Transport | TLS 1.2, redirections non suivies automatiquement |
| Réponse API | 512 Kio maximum |
| Manifeste | 64 Kio maximum, taille et version vérifiées |
| Nom d'asset | `Beaver_<version>_x64-setup.exe` exactement |
| Empreinte | SHA-256 sur 64 caractères hexadécimaux, comparée au manifeste |
| Taille de l'asset | Doit correspondre exactement au manifeste |
| Binaire installé | `cl-go-dash.exe` présent et non-lien |

### Tableau — Contraintes sur le dossier d'installation

| Règle | Détail |
|---|---|
| Format | Commence par `X:\` |
| Longueur | 1024 caractères maximum |
| Caractères interdits | `*` `?` `<` `>` `\|` `"` et caractères de contrôle |
| Deux-points | Interdits au-delà des trois premiers caractères |
| Remontée | Aucun segment `..` |

---

## Encadrés

**Encadré « SmartScreen »** — information, section 4.
> Windows affichera « Windows a protégé votre ordinateur ». Cliquez sur « Informations complémentaires », puis « Exécuter quand même ». Beaver n'est pas signé par un certificat de signature de code.

**Encadré « Autorisez Ollama »** — avertissement, section 5.
> Si Windows affiche une notification de blocage au premier téléchargement de modèle, cliquez sur « Autoriser ». C'est la protection anti-rançongiciel qui empêche Ollama d'écrire les modèles sur le disque. La demande n'apparaît qu'une fois.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Le script s'arrête immédiatement | Architecture différente d'AMD64 | Aucune : pas de build ARM |
| « Windows a protégé votre ordinateur » | Exécutable non signé | Informations complémentaires › Exécuter quand même |
| Le téléchargement de modèle échoue sans message clair | Accès contrôlé aux dossiers bloque `ollama.exe` | Cliquer « Autoriser » dans la notification Windows |
| L'application ne démarre pas après installation | WebView2 ou runtime VC++ absent | Sujet ouvert, voir *Points à confirmer* |
| L'installation s'arrête sans explication | Un contrôle a échoué : empreinte, taille, chemin | Relancer ; si ça persiste, signaler le problème |
| Nom de fichier tronqué dans l'interface | Séparateur de chemin Windows mal découpé | Corrigé en v0.7.0 ; signaler si le symptôme réapparaît |

---

## Renvois

- *Prérequis* — compatibilité
- *Premier lancement* — la suite
- *Mise à jour* — la mise à jour intégrée
- *Dépannage › Installation*
- *Dépannage › Ollama* — GPU AMD et Vulkan

---

## Points à confirmer

- **WebView2 et le runtime Visual C++.** `CROSS-PLATFORM.md` signale que l'installeur NSIS ne les gère pas explicitement et que rien n'a été testé sur une machine vierge. Tant que ce n'est pas vérifié, ne pas écrire « aucune dépendance nécessaire ».
- **Réinstallation par-dessus une installation existante.** Les scripts macOS et Linux refusent ; le script Windows ne fait pas ce contrôle. Vérifier ce qui se passe réellement, et harmoniser le message si besoin.
- **La procédure de désinstallation complète.** Non documentée : désinstalleur, sort du dossier `.local\share\cl-go-dash`, nettoyage du secret DPAPI.
- **Le dossier de données sous Windows.** `CROSS-PLATFORM.md` indique `C:\Users\<user>\.local\share\cl-go-dash\` — un emplacement inhabituel sur Windows, hérité de l'unification des chemins. Confirmer, et l'expliquer sur le site : un utilisateur Windows cherchera dans `%APPDATA%`.
- **La migration depuis l'ancien emplacement.** `CROSS-PLATFORM.md` mentionne une copie automatique depuis `%APPDATA%\cl-go-dash`. Vérifier que le mécanisme est toujours actif en 1.1.2.
- **Le GPU AMD et Vulkan.** Fonctionne, mais avec une perte de performance importante liée à une version datée de llama.cpp dans Ollama. Un modèle est explicitement signalé comme produisant des réponses incohérentes sous Vulkan. Vérifier si c'est toujours vrai avant de le documenter, et sous quelle forme.
