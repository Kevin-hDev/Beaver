# Diagnostics et erreurs

**Emplacement site** — Agent › Diagnostics et erreurs (ou Dépannage › Agent et outils)
**Répond à** — « Quelque chose a échoué. Comment je comprends quoi, et est-ce que je peux réessayer ? »
**Sources** — `src-tauri/src/services/agent_local/tool_result_contract.rs` (lignes 36-56), `types_diagnostics.rs`, `diagnostic_args.rs` (lignes 4-5), `diagnostic_redaction.rs` (lignes 3-4), `circuit_breaker.rs`, `permission_gate.rs` (journal), `services/app_log.rs`
**Vérification** — Vérifié dans le code : catégories d'erreurs, indicateur de reprise, mécanisme de masquage

---

## Plan de page proposé

1. Comment lire une erreur
2. Les dix catégories
3. Peut-on réessayer ?
4. Les diagnostics
5. Ce qui est masqué dans les diagnostics
6. Les journaux

---

## Contenu

### 1. Comment lire une erreur

Chaque échec d'outil porte trois informations :

- un **code** — identifiant précis de la cause ;
- une **catégorie** — la nature du problème ;
- un **indicateur de reprise** — dit si réessayer est sûr.

L'intérêt pour l'utilisateur est le troisième : il évite de relancer une action qui échouera à l'identique, ou pire, qui aurait déjà partiellement abouti.

### 2. Les dix catégories

| Catégorie | Signification | Que faire |
|---|---|---|
| **Validation** | Les arguments sont invalides | Reformuler la demande |
| **Permission** | L'action a été refusée | Vérifier le mode de permission et la portée d'accès |
| **Introuvable** | Le fichier ou la ressource n'existe pas | Vérifier le chemin |
| **Conflit** | L'état a changé entre-temps | Inspecter la situation avant de relancer |
| **Délai dépassé** | L'opération a été trop longue | Réessayer, ou réduire la portée |
| **Annulé** | Vous avez interrompu | Rien |
| **Indisponible** | Le service ou l'outil n'est pas joignable | Vérifier la configuration |
| **Externe** | Un service tiers a échoué | Vérifier le fournisseur ou le connecteur |
| **Exécution** | L'action a échoué à l'exécution | Lire le message |
| **Interne** | Défaut de Beaver | Signaler le problème |

Cette classification n'est pas décorative : chaque catégorie appelle une réaction différente, et la colonne de droite est ce que l'utilisateur cherche.

### 3. Peut-on réessayer ?

L'indicateur de reprise est vrai **uniquement quand une nouvelle tentative est sûre sans vérifier l'état extérieur**.

La nuance compte. Un délai dépassé sur une écriture n'est pas rejouable à l'aveugle : l'écriture a peut-être abouti. Une lecture qui échoue, elle, se rejoue sans risque.

### 4. Les diagnostics

Beaver conserve des relevés de diagnostic par conversation : ce qui a été tenté, avec quels arguments résumés, et ce qui a échoué.

Ils servent à comprendre après coup une conversation qui a mal tourné, sans relire tout l'historique.

Les diagnostics ne sont pas repris dans un clone de conversation.

### 5. Ce qui est masqué dans les diagnostics

Point de confiance à mettre en avant.

Les arguments enregistrés sont **résumés et expurgés** avant écriture :

- le texte est tronqué à **200 caractères** ;
- les structures complètes à **1 000 caractères** ;
- les valeurs sensibles sont remplacées par **`[redacted]`** ;
- les chemins sont remplacés par **`[path]`**.

Autrement dit : un diagnostic permet de comprendre ce qui s'est passé sans conserver le contenu de vos fichiers ni vos secrets.

### 6. Les journaux

Plusieurs journaux existent dans le dossier de données, sous `logs/`. Ceux repérés dans le code lu jusqu'ici :

| Journal | Contenu | Rotation |
|---|---|---|
| `permission-diagnostics.jsonl` | Décisions de permission | À 2 Mo |
| `wakeups.jsonl` | Exécutions des réveils | 500 lignes |
| `gateway-audit.jsonl` | Échanges des canaux externes | — |
| `ollama-sidecar.log` | Sortie d'erreur d'Ollama | Écrasé à chaque démarrage |

Le dernier est le plus utile en pratique : c'est là qu'on regarde quand un modèle local ne se comporte pas comme attendu, en particulier sur les questions de carte graphique.

Renvoyer vers *Référence › Journaux* pour la liste complète.

---

## Encadrés

**Encadré « Vos données ne sont pas dans les diagnostics »**
> Les relevés de diagnostic tronquent les textes et remplacent les valeurs sensibles et les chemins par des marqueurs. Ils décrivent ce qui s'est passé sans conserver le contenu de vos fichiers.

**Encadré « Toutes les erreurs ne se rejouent pas »**
> Une opération d'écriture interrompue par un délai dépassé a peut-être abouti. Beaver indique explicitement quand une nouvelle tentative est sûre.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause probable | Résolution |
|---|---|---|
| Erreur de permission alors que le mode est Accès complet | Chemin hors de la portée d'accès, ou chemin sensible | Voir *Répertoire de travail* et *Permissions* |
| Erreur « conflit » sur des changements de sous-agent | Le dossier a changé pendant son travail | Inspecter avant d'appliquer |
| Erreur « externe » répétée | Fournisseur ou connecteur en difficulté | Vérifier la clé, le quota, l'état du service |
| Erreur « indisponible » sur un modèle local | Ollama non démarré, ou modèle absent | Voir *Dépannage › Ollama* |
| Erreur « interne » | Défaut de Beaver | Signaler avec le code affiché |
| La conversation s'arrête sans message d'erreur | Garde-fou anti-boucle, ou 200 tours | Voir *Fonctionnement* |

---

## Renvois

- *Agent › Fonctionnement* — les conditions d'arrêt
- *Agent › Permissions*
- *Agent › Répertoire de travail*
- *Référence › Journaux*
- *Dépannage* — les pages par domaine

---

## Points à confirmer

- **Les libellés français des dix catégories** tels qu'ils s'affichent. Ceux du tableau sont des traductions des identifiants du code.
- **Où l'utilisateur voit les diagnostics** : dans la conversation, dans un écran dédié, dans les réglages ? Non relevé.
- **La liste des codes d'erreur les plus fréquents.** La catégorie oriente, mais le code est précis. Les recenser rendrait la page nettement plus utile — c'est ce que fait une bonne page d'erreurs de référence.
- **Ce qui déclenche l'enregistrement d'un diagnostic** : tout échec, ou seulement certains ?
- **La liste complète des journaux** et leur rotation. Quatre sont identifiés ; le code mentionne aussi un journal applicatif général et un journal d'outils.
- **Les messages visibles sont-ils tous traduits ?** La consigne du projet impose des clés de traduction plutôt que des messages bruts. Vérifier sur les erreurs d'outils, où plusieurs messages apparaissent en dur dans le code Rust.
