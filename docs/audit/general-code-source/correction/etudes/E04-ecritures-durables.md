# E04 — Contrats des écritures durables

Date du relevé : 17 septembre 2026
Commit examiné : `c8a918b6`

## Décision

Conserver deux autorités : `private_store` pour les documents privés de Beaver et `ollama_manager::durable_fs` pour la transaction d'installation d'Ollama. Elles utilisent le même principe système, mais leurs garanties après interruption, leurs erreurs et la propriété des fichiers temporaires diffèrent.

Aucune nouvelle primitive commune n'est créée. La seule partie réellement identique et sans politique, le nouvel essai borné des erreurs de partage Windows, passe déjà par `services::windows_fs_retry::bounded`. Extraire davantage obligerait à paramétrer l'annulation, le remplacement, les droits privés, le nettoyage, les erreurs et la reprise ; cette couche serait plus complexe que les appels système qu'elle remplacerait.

## Comparaison

| Garantie | `private_store` | `ollama_manager::durable_fs` | Décision |
|---|---|---|---|
| Destination | Remplace une génération de document privée. | Écrit sans remplacer ou remplace selon l'étape de transaction. | Garder deux opérations Ollama explicites. |
| Nom temporaire | Aléatoire et créé par l'autorité de stockage. | Fourni par le plan d'installation et connu de la reprise. | Ne pas partager la propriété du temporaire. |
| Résidu après échec | Supprimé par `TempCleanup`. | Conservé tant que la publication n'a pas consommé son nom afin que la reprise puisse le classer. | Garder les comportements opposés. |
| Droits | Dossiers et fichiers privés, mode Unix ou ACL Windows vérifiée avant publication. | Layout du bundle, avec vérifications propres aux arbres et aux liens. | Ne pas déplacer les ACL dans Ollama. |
| Erreurs | Code utilisateur générique, sans chemin ni détail ; durabilité publiée mais non confirmée représentée séparément. | Erreur typée : absence, existence, partage, permission, entrée invalide, annulation et opération native. | Garder les modèles d'erreur. |
| Synchronisation Unix | Fichier, remplacement, puis dossier parent. Un échec du dernier `fsync` n'annule pas une publication déjà visible. | Fichier, renommage, puis parents source et destination. Tout échec reste une erreur de transaction traitée par la reprise. | Ne pas uniformiser la règle après publication. |
| Windows | `MoveFileExW` avec écriture immédiate, remplacement et verrou de métadonnées de sécurité. | Même primitive native, avec ou sans remplacement, annulation et erreur Windows conservée. | Garder les enveloppes ; réutiliser seulement le retry commun déjà présent. |
| Synchronisation entre appels | Verrous propres aux documents qui font lecture-modification-écriture. | Journal et machine de reprise de l'installation. | Ne pas ajouter de verrou transversal. |

## Pourquoi le sens de l'échec diffère

Pour un document de profil, le remplacement rend immédiatement la nouvelle génération lisible. Si la synchronisation du dossier échoue ensuite, relancer automatiquement une lecture-modification-écriture pourrait appliquer deux fois la modification. `private_store` rend donc `PublishedDurabilityUnconfirmed` à l'appelant spécialisé et journalise ce cas pour les appelants simples.

Pour Ollama, chaque renommage fait partie d'une séquence durable décrite par le journal d'installation. Un échec de synchronisation doit rester visible comme une erreur afin que la reprise reclasse les dossiers `active`, `backup`, `staging` ou `failed`. Le masquer changerait la décision de récupération.

## Frontière commune retenue

Une primitive commune reste acceptable uniquement si elle est sans politique : par exemple une classification d'erreur native ou une boucle bornée qui réessaie une opération. `windows_fs_retry::bounded` remplit déjà ce rôle. Elle ne connaît ni le coffre, ni Ollama, ni le type d'erreur public.

Les opérations `MoveFileExW`, `rename` et `fsync` ne justifient pas seules une abstraction. Le code qui les entoure porte la garantie utile. Un troisième domaine ne changera cette décision que s'il a exactement l'un des deux contrats existants ; il devra alors appeler directement cette autorité plutôt que créer une troisième variante.

## Synchronisation et reprise à conserver

- les verrous lecture-modification-écriture restent dans chaque stockage de document ;
- le verrou Windows des métadonnées privées reste dans `private_store` ;
- l'annulation des nouveaux essais Windows reste dans le système Ollama ;
- le temporaire privé est supprimé lors de toute sortie, tandis que le temporaire Ollama reste disponible pour la reprise ;
- les deux parents Unix sont synchronisés après un renommage Ollama entre dossiers différents ;
- les écritures du coffre restent indépendantes de tout module ou type Ollama.

## Preuves requises lors d'un futur changement

- interruption avant et après chaque étape : création, écriture, synchronisation du fichier, droits, publication et synchronisation du parent ;
- ancienne ou nouvelle génération complète pour les documents privés, jamais un mélange ;
- résidu Ollama classable par la reprise et résidu privé nettoyé sans bloquer l'écriture suivante ;
- Windows : verrou transitoire, annulation, remplacement et refus d'écraser pour l'écriture nouvelle ;
- Unix : parents identiques et différents, avec ordre de synchronisation vérifié ;
- coffre : droits privés vérifiés avant publication et aucune erreur détaillée exposée.

## Conséquences pour le plan

- E04 ne crée aucune correction autonome.
- Les futurs nettoyages Ollama doivent garder `OllamaDurableFs` et ses erreurs typées.
- Les documents privés continuent d'appeler `private_store::atomic_write`; aucun appelant ne doit dépendre d'Ollama.
- M30 et M31 concernent les conversations et ne dépendent pas de cette étude.
- Si une divergence apparaît dans la boucle de nouvel essai Windows, la correction doit être faite dans `windows_fs_retry::bounded`, qui est déjà l'autorité commune.

## Validation de l'étude

La comparaison couvre les implémentations Unix et Windows, les deux modes Ollama avec et sans remplacement, la synchronisation des parents, les erreurs typées, l'annulation et les tests d'interruption de `private_store`. Le doublon système du constat 02-C5 est confirmé, mais sa recommandation de faire envelopper le coffre par l'implémentation Ollama est rejetée : elle inverserait la dépendance entre un service général de stockage privé et un domaine produit spécialisé.
