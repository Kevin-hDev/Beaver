# Intégrations : Providers, Connecteurs, Canaux, Extensions

**Emplacement site** — Réglages › Intégrations
**Répond à** — « Où branche-t-on Beaver sur le reste : fournisseurs de modèles, services externes, messageries, extensions ? »
**Sources** — `src/components/api-keys/api-keys-tab.tsx`, `src/components/providers/providers-shell.tsx`, `oauth-providers.tsx` ; `src/components/connectors/connectors-tab.tsx`, `connectors-detail.tsx` ; `src/components/channels/channels-tab.tsx` ; `src/components/extensions/extensions-tab.tsx`, `extension-sections.ts` ; `src/components/settings/settings-child-slots.tsx` ; `src/types/navigation.ts`, `src/types/extension-ui-contract.generated.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Cette page est une carte, pas un mode d'emploi.** Chacun des quatre onglets a sa page de fond : les clés API dans `06-modeles/providers-api.md`, les comptes web dans `06-modeles/providers-comptes-web.md`, les connecteurs dans `07-integrations/mcp-connecteurs.md`, les canaux dans `07-integrations/channels-gateway.md`, les extensions dans `07-integrations/extensions-centre.md`. Ici : comment les quatre écrans sont organisés, et lequel ouvrir pour quoi.

---

## Plan de page proposé

1. Ce que contient la section Intégrations
2. Une organisation commune aux quatre onglets
3. Onglet Providers
4. Onglet Connecteurs
5. Onglet Canaux
6. Onglet Extensions
7. Ce que les quatre onglets ont en commun côté sécurité

---

## Contenu

### 1. Ce que contient la section Intégrations

Quatre onglets (`src/features/extension-ui/core-occupants.tsx:51-58`), qui répondent à quatre questions distinctes :

| Onglet | La question à laquelle il répond |
|---|---|
| **Providers** | Avec quoi Beaver produit-il ses réponses ? |
| **Connecteurs** | À quels services extérieurs l'agent peut-il parler ? |
| **Canaux** | Par où peut-on écrire à Beaver depuis l'extérieur ? |
| **Extensions** | Quel code tiers tourne dans Beaver ? |

La distinction entre **Connecteurs** et **Canaux** est celle qu'on comprend le moins vite, et le site doit la poser dès le début. Elle tient au **sens de la conversation** :

- un **connecteur** est un service que **l'agent va consulter** — « Permettez aux LLM de se connecter à d'autres applications et services » (`fr.json`, clé `connectors.main.subtitle`) ;
- un **canal** est une messagerie depuis laquelle **on écrit à Beaver** — « Connectez des bots de messagerie pour communiquer avec les LLM. » (`fr.json`, clé `channels.main.subtitle`).

### 2. Une organisation commune aux quatre onglets

Les quatre écrans suivent le même schéma, ce qui rend la section prévisible et mérite d'être dit une fois pour toutes :

1. **une liste** de ce qui est déjà configuré ;
2. **un bouton d'ajout** en haut à droite du panneau, qui ouvre un catalogue à parcourir ;
3. **une fiche de détail** quand on sélectionne un élément, avec un lien de retour vers la liste ;
4. **un message d'état vide** quand rien n'est configuré.

C'est vérifiable sur les trois premiers onglets, construits sur les mêmes composants (`api-keys-tab.tsx:104-116`, `connectors-tab.tsx:104-131`, `channels-tab.tsx:89-115`). Les boutons d'ajout portent des libellés parallèles : **« Connecteurs API »** pour les fournisseurs, **« Parcourir les connecteurs »**, **« Parcourir les canaux »**.

Conséquence pratique : **l'écran vide n'est pas une impasse.** Le bouton d'ajout est présent même quand la liste est vide, et c'est par lui qu'on commence.

### 3. Onglet Providers

Le seul onglet de la section à porter deux sous-onglets (`providers-shell.tsx:18-21`) :

| Sous-onglet | Libellé affiché | Ce qu'on y fait |
|---|---|---|
| Clés | **Clés API** | Enregistrer une clé chez un fournisseur |
| Comptes | **OAuth** | Se connecter avec un compte web existant |

Ce sont deux façons de se connecter au même genre de service, et le choix entre les deux est un vrai sujet — traité dans `06-modeles/providers-api.md` et `06-modeles/providers-comptes-web.md`.

**Le sous-onglet Clés API** liste les fournisseurs déjà configurés, chacun avec son icône et son nom. Un bouton **« Connecteurs API »** ouvre le catalogue complet des fournisseurs disponibles ; en choisir un ouvre une fenêtre de configuration où l'on colle la clé (`api-keys-tab.tsx:118-154`).

La fiche d'un fournisseur configuré propose de **modifier** la clé ou de la **supprimer**.

**Le point à répéter ici**, parce que c'est la question que tout le monde se pose : **une clé enregistrée ne peut plus être relue**, y compris par Beaver lui-même. Aucune commande de l'application ne permet de l'afficher. La démonstration complète est dans `11-securite/vault-et-cles-api.md`, qui est la page à lier depuis cet écran.

**Le sous-onglet OAuth** liste les comptes web connectés et permet de s'y connecter ou de s'en déconnecter.

### 4. Onglet Connecteurs

Liste des connecteurs MCP configurés, avec un bouton **« Parcourir les connecteurs »** qui ouvre un catalogue (`connectors-tab.tsx:99-102`, `:132-133`).

La fiche d'un connecteur propose trois actions (`connectors-tab.tsx:107-119`) :

- **connecter ou déconnecter** — la déconnexion passe par une **confirmation** ;
- **supprimer** le connecteur ;
- revenir à la liste.

Quand la liste est vide, le message dépend de la cause : « aucun connecteur » si tout va bien, un message d'erreur de chargement si la lecture a échoué (`connectors-tab.tsx:126`). C'est une distinction utile : une liste vide et une liste illisible ne demandent pas la même action.

**Le lien avec l'onglet Outils** est à écrire : l'agent n'utilise les connecteurs que si le groupe d'outils **Connecteurs externes** est actif — et celui-là est **toujours actif**, il fait partie des cinq groupes verrouillés. Sa description le dit d'ailleurs mot pour mot : « Permet à l'agent d'utiliser les connecteurs externes que tu as configurés dans l'onglet Connecteurs. » Configurer un connecteur ici suffit donc à le rendre disponible.

### 5. Onglet Canaux

Même structure : liste des canaux configurés, bouton **« Parcourir les canaux »**, fiche de détail avec configuration et suppression (`channels-tab.tsx:82-115`).

Une particularité par rapport aux deux onglets précédents : la fiche affiche **l'état de santé** du canal, relevé en continu (`channels-tab.tsx:97`). Un canal peut être configuré et hors service — c'est une information que ni les connecteurs ni les fournisseurs n'exposent de la même façon.

Un canal se configure **par compte** : la clé d'un canal dans l'état de navigation combine l'identifiant du canal et celui du compte (`channels-tab.tsx:95-96`). Plusieurs comptes sur la même messagerie sont donc possibles.

### 6. Onglet Extensions

Le plus riche des quatre, et le seul qui soit **protégé par le contrat des extensions** : aucune extension ne peut le retirer ni le remplacer (`extension-ui-contract.generated.ts:12`). Le raisonnement mérite d'être écrit : on ne peut pas installer une extension qui vous priverait du moyen de la désinstaller.

**Trois sections** (`extension-sections.ts:16-20`) :

| Section | Libellé affiché | Contenu |
|---|---|---|
| `plugins` | **Plugins** | « Les plugins officiels de Beaver, distincts des Tools internes. » |
| `custom` | **Extensions** | « Ajoutez du code local ou installez une extension depuis Git ou npm. » |
| `host` | **Hôte** | L'état du processus qui exécute les extensions |

Chaque section a son état vide écrit : « La suite officielle de plugins apparaîtra ici lorsqu'elle sera disponible. » pour les plugins, « Aucune extension personnalisée. » pour les extensions (`fr.json`, clés `extensions.pages.*`).

Les actions disponibles sur une extension, lues dans le code qui les câble (`extensions-tab.tsx:44-59`) : **activer ou désactiver**, **afficher ou non dans le chat**, **ouvrir sa source**, **mettre à jour**, **supprimer**, **recharger**, et pour celles qui ont échoué à se charger, **réessayer** ou **rester désactivée**.

Deux mécanismes de reprise existent et méritent d'être signalés, parce qu'ils traitent un cas réel — une extension qui empêche l'application de fonctionner : une **récupération** de l'hôte et la **restauration d'un instantané** antérieur (`extensions-tab.tsx:56-59`). Le détail est dans `07-integrations/extensions-centre.md`.

Certaines extensions sont **protégées contre la suppression** (`extensions-tab.tsx:41`, `protectedPluginIds`). Le critère exact reste à confirmer — voir « Points à confirmer ».

### 7. Ce que les quatre onglets ont en commun côté sécurité

Une section courte mais nécessaire, à écrire une fois ici et à lier depuis les quatre pages de fond.

**Tout ce qui est secret dans ces quatre écrans va au même endroit** : le coffre chiffré de Beaver. Clés API des fournisseurs, jetons des comptes web, jetons OAuth des connecteurs, variables d'environnement secrètes des connecteurs, jetons des canaux — chacun a son entrée dans `secrets.enc`, et aucune n'est relisible par l'interface. L'inventaire complet est dans `11-securite/vault-et-cles-api.md`.

**Une exception, et elle est délibérée** : une **extension approuvée** peut demander au coffre une clé de fournisseur, un jeton de connecteur ou un jeton de canal. Le coffre protège vos secrets du reste du système, pas du code que vous avez vous-même autorisé. C'est pourquoi l'activation d'une extension demande une confirmation explicite.

Cette phrase doit figurer sur la page du site, parce qu'elle explique pourquoi l'onglet Extensions n'est pas un onglet comme les trois autres.

---

## Encadrés

> **ℹ Connecteur ou canal ? La différence tient au sens de la conversation.**
> Un **connecteur** est un service que l'agent va consulter pour vous. Un **canal** est une messagerie depuis laquelle vous écrivez à Beaver. Les deux se configurent dans cette section, mais ils ne servent pas du tout à la même chose.

> **⚠ Une clé API enregistrée ne peut plus être affichée.**
> Y compris par Beaver. Il n'existe aucune commande capable de la relire. Notez-la ailleurs si vous en avez besoin, ou générez-en une nouvelle chez le fournisseur.

> **⚠ Une extension activée peut demander vos secrets.**
> Clés de fournisseurs, jetons de connecteurs, jetons de canaux : le coffre les remet au code que vous avez approuvé. C'est la raison de la confirmation demandée à l'activation, et la raison pour laquelle l'onglet Extensions ne peut jamais être retiré par une extension.

> **ℹ Une liste vide n'est pas une impasse.**
> Le bouton d'ajout est en haut à droite du panneau, présent même quand rien n'est configuré. C'est par lui qu'on parcourt le catalogue.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Je ne trouve pas où saisir ma clé API » | Ce n'est plus un onglet : c'est le sous-onglet **Clés API** de **Providers** | Ouvrir Providers, puis Clés API |
| « Je ne retrouve plus ma clé enregistrée » | Aucune commande ne permet de relire une clé | Générer une nouvelle clé chez le fournisseur |
| « J'ai supprimé ma clé dans Beaver, est-elle révoquée ? » | Non : elle reste valable chez le fournisseur | La supprimer sur le site du fournisseur |
| « Mon connecteur est configuré mais l'agent ne l'utilise pas » | Un connecteur configuré doit aussi être **connecté** | Ouvrir sa fiche et le connecter |
| « Mon canal est configuré mais rien n'arrive » | Un canal peut être configuré et hors service ; sa fiche affiche son état de santé | Voir `07-integrations/channels-gateway.md` |
| « La liste des connecteurs est vide alors que j'en avais » | Le message distingue une liste vide d'une erreur de lecture | Lire le message affiché ; en cas d'erreur, voir `13-depannage/` |
| « Je veux supprimer une extension et le bouton n'est pas là » | Certaines extensions sont protégées contre la suppression | Voir `07-integrations/extensions-centre.md` |
| « Une extension empêche Beaver de fonctionner » | Des mécanismes de récupération existent dans la section **Hôte** | Même page |

---

## Renvois

- `10-reglages/reference-complete.md` — l'arborescence complète des réglages
- `06-modeles/providers-api.md` — les fournisseurs, où récupérer une clé et comment la saisir
- `06-modeles/providers-comptes-web.md` — se connecter avec un compte plutôt qu'une clé
- `07-integrations/mcp-connecteurs.md` — les connecteurs MCP en détail
- `07-integrations/mcp-oauth.md` — l'autorisation d'un connecteur par OAuth
- `07-integrations/channels-gateway.md` — les canaux externes et leur passerelle
- `07-integrations/extensions-centre.md` — installer, activer et surveiller une extension
- `07-integrations/extensions-ecrire.md` — écrire sa propre extension
- `11-securite/vault-et-cles-api.md` — où vont les secrets de ces quatre écrans
- `10-reglages/agent.md` — le groupe d'outils **Connecteurs externes**, toujours actif
- `05-outils/mcp.md` — comment l'agent se sert d'un connecteur

---

## Points à confirmer

1. **Cette page ne détaille volontairement aucun parcours.** Les quatre sujets ont déjà des briefs de fond, dont deux très développés (`extensions-centre.md`, `vault-et-cles-api.md`). **Le risque à surveiller à la rédaction est la duplication** : cette page doit rester une carte et renvoyer, sous peine de créer une deuxième source de vérité sur des sujets où elle divergera.
2. **Le critère des « extensions protégées » n'a pas été établi.** Le code passe une liste d'identifiants protégés à l'écran (`extensions-tab.tsx:41`) sans que la lecture de cet écran dise d'où elle vient. À vérifier dans `07-integrations/extensions-centre.md`, qui a traité le sujet de plus près, avant d'écrire quoi que ce soit.
3. **Les libellés des sous-onglets Providers sont en partie en anglais** : l'onglet lui-même s'affiche **« Providers »** et son second sous-onglet **« OAuth »**. Voir `10-reglages/reference-complete.md`, point 3 : la décision de traduire ou de reprendre les mots affichés vaut pour tout le site.
4. **La distinction connecteur / canal proposée ici** est ma reformulation des deux sous-titres de l'application. Elle est fidèle au code, mais c'est une formulation de documentation, pas un texte du produit. À valider avec l'équipe, puisqu'elle sera reprise partout.
5. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la barre des deux sous-onglets de Providers ; les états vides des quatre onglets, qui sont ce que voit un nouvel utilisateur ; le catalogue ouvert par chacun des trois boutons de parcours ; l'affichage de l'état de santé dans la fiche d'un canal ; la section **Hôte** de l'onglet Extensions, qui n'a d'équivalent nulle part ailleurs dans les réglages.
