# Navigateur intégré

**Emplacement site** — Outils › Navigateur intégré (page prévue au sommaire du mockup)
**Répond à** — « Qu'est-ce que ce navigateur dans Beaver, qu'est-ce qu'il sait faire, et est-ce qu'il est sûr ? »
**Sources** — `src-tauri/src/services/browser/session_types.rs` (ligne 3), `url_policy.rs`, `live_session_registry.rs`, `session_store.rs`, `local_site_types.rs`, `local_site_scanner.rs`, `cef_permission_handler.rs`, `cef_cookie_gate*.rs`, `cef_download_handler.rs`, `src/components/internal-browser/`
**Vérification** — Vérifié dans le code : limites, politique d'URL et mécanismes de restriction

---

## Plan de page proposé

1. À quoi il sert
2. Disponibilité par système
3. Les onglets
4. Les sessions connectées
5. La détection des serveurs de développement
6. Ce qui est bloqué
7. Les limites

---

## Contenu

### 1. À quoi il sert

Un navigateur web complet, dans le panneau latéral de la conversation. Il repose sur un moteur Chromium embarqué.

Trois usages :

- consulter une page sans quitter l'application ;
- rester connecté à un service d'une conversation à l'autre ;
- voir le rendu d'un serveur de développement local que l'agent vient de démarrer.

### 2. Disponibilité par système

**Disponible sur macOS et Windows. Pas sur Linux.**

À répéter sur cette page même si c'est déjà dit ailleurs : c'est la seule fonctionnalité majeure absente d'un système.

### 3. Les onglets

- **Dix onglets au maximum par conversation.**
- Les titres sont tronqués à **80 caractères**.
- Les onglets sont rattachés à la conversation : chacune a les siens.

### 4. Les sessions connectées

- Les sessions restaurées sont **chiffrées** sur le disque.
- Le navigateur utilise un **profil séparé**, distinct de votre navigateur habituel : vos cookies personnels ne sont pas exposés à l'agent.
- Taille d'un fichier de session : **128 Ko** au maximum.

### 5. La détection des serveurs de développement

Beaver examine les ports locaux pour repérer un serveur de développement en cours d'exécution et le proposer directement.

Bornes du balayage :

| | Valeur |
|---|---|
| Ports candidats examinés | 128 |
| Résultats retenus | 32 |
| Sondages simultanés | 8 |
| Contenu lu par site | 64 Ko |
| Longueur du titre | 80 caractères |
| Redirections suivies | 3 |

Les résultats sont classés par numéro de port.

### 6. Ce qui est bloqué

Le navigateur n'est pas un navigateur ordinaire : plusieurs capacités sont volontairement refusées.

- **Seuls `http` et `https` sont acceptés.** Tout autre schéma d'URL est rejeté.
- **Longueur d'URL plafonnée à 2 048 caractères.**
- **Les demandes d'accès aux périphériques** — caméra, microphone — sont interceptées par un gestionnaire dédié.
- Les téléchargements et les cookies passent par des contrôles spécifiques.
- Quand une fonctionnalité est refusée, l'utilisateur en est informé.

Formulation à retenir : ces restrictions ne sont pas des manques, ce sont des choix. Un navigateur piloté par un agent qui aurait accès au micro et à la caméra poserait un problème évident.

### 7. Les limites

| Limite | Valeur |
|---|---|
| Onglets par conversation | 10 |
| Sessions actives | 64 |
| Longueur d'URL | 2 048 caractères |
| Fichier de session | 128 Ko |
| Magasin de cookies | 256 Mo |

---

## Encadrés

**Encadré « macOS et Windows uniquement »** — avertissement.
> Le navigateur intégré n'est pas disponible sur Linux.

**Encadré « Un profil séparé »**
> Le navigateur intégré utilise son propre profil, indépendant de votre navigateur habituel. Vos cookies et vos sessions personnels ne lui sont pas accessibles.

**Encadré « Ce qui est refusé »**
> Le navigateur n'accepte que les adresses `http` et `https`, et refuse les demandes d'accès à la caméra et au microphone.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Le navigateur est absent de l'interface | Système Linux | Aucune : non disponible |
| Une URL est refusée | Schéma autre que `http`/`https`, ou plus de 2 048 caractères | Utiliser une adresse web classique |
| Impossible d'ouvrir un onzième onglet | Plafond de dix par conversation | Fermer un onglet |
| Un site demande la caméra et ne l'obtient pas | Refus volontaire | Utiliser votre navigateur habituel |
| Un serveur local n'est pas détecté | Hors des ports examinés, ou plus de 32 résultats | Saisir l'adresse manuellement |
| La session de connexion est perdue | Fichier de session au-delà de 128 Ko | Se reconnecter |

---

## Renvois

- *Interface › Panneau latéral* — le partage de l'espace avec les prévisualisations et Forecast
- *Sécurité › Durcissement* — le détail des restrictions
- *Prérequis* — la disponibilité par système

---

## Points à confirmer

- **La limite de 64 sessions actives** — globale, ou par conversation ? Le nom du registre suggère un compte global. Formulation à ajuster.
- **La plage de ports examinée** pour la détection de serveurs locaux. Le nombre de candidats est connu (128), mais pas lesquels. Utile : un utilisateur dont le serveur tourne sur un port exotique doit savoir pourquoi il n'est pas détecté.
- **Le comportement exact du contrôle des cookies.** Plusieurs fichiers y sont consacrés, dont un dédié au nettoyage. Décrire ce qui est conservé, ce qui est effacé et quand.
- **Les téléchargements.** Un gestionnaire dédié existe ; son comportement — autorisé, refusé, vers quel dossier — n'a pas été vérifié.
- **L'agent peut-il piloter ce navigateur ?** Aucun outil de navigation n'apparaît dans le catalogue. À confirmer : c'est la première question que se posera un utilisateur venant d'un agent doté de capacités de navigation.
- **Le comportement sur Linux.** L'interface masque-t-elle proprement la fonctionnalité, ou affiche-t-elle une erreur ? Détermine ce qu'on écrit dans le tableau de dépannage.
