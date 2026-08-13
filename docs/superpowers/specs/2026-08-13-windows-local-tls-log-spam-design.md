# Suppression du bruit TLS du détecteur local Windows

## Objectif

Tu empêches le détecteur automatique de sites locaux de produire toutes les
cinq secondes une erreur `rustls_platform_verifier` lorsqu'un serveur HTTPS
local présente un certificat non approuvé. Tu conserves le refus strict de ce
certificat et tu ne modifies pas la navigation CEF ni les autres clients HTTPS
de Beaver.

## Cause racine vérifiée

Trois hypothèses ont été départagées :

1. Le détecteur local est la source. Le hook `useLocalSites` appelle
   `browser_detect_local_sites` toutes les cinq secondes tant que l'accueil du
   navigateur est visible. Le scanner essaie HTTP puis HTTPS sur chaque port
   local. `reqwest` utilise alors `rustls_platform_verifier`, qui écrit une
   erreur avant de retourner `UnknownIssuer`. La cadence des traces et la
   chaîne d'appels confirment cette hypothèse.
2. CEF produit l'erreur. Cette hypothèse est écartée : le chemin CEF ne dépend
   pas de `reqwest` ou de `rustls` pour cette opération, et la cible de la trace
   appartient au vérificateur TLS Rust du processus principal.
3. Un autre service périodique produit l'erreur. Cette hypothèse est écartée :
   le seul chemin relié à l'accueil visible, au délai de cinq secondes et à la
   double tentative HTTP/HTTPS est le détecteur local.

## Classement de l'erreur

Un certificat local non approuvé est un résultat attendu de la découverte. Le
site n'est pas ajouté à la liste et le scan continue. Beaver n'a ni action à
demander ni état incohérent à signaler, donc cette trace n'est pas une erreur
applicative.

## Approches étudiées

### 1. Filtre limité à la sonde HTTPS locale — retenu

Le propriétaire du journal expose une portée asynchrone temporaire. Pendant
cette portée seulement, il ignore la cible Windows exacte de
`rustls_platform_verifier`. Le probe place uniquement l'envoi HTTPS local dans
cette portée. La vérification TLS s'exécute normalement et le certificat reste
refusé.

La portée suit la tâche asynchrone même si son exécution change de fil. Une
requête concurrente d'un autre service ne se trouve pas dans cette portée et
sa trace TLS reste visible.

### 2. Moteur TLS natif Reqwest — écarté après vérification

Les fonctionnalités Cargo sont additives. Activer `native-tls` sur la même
version de `reqwest` ferait du moteur natif le choix par défaut des nombreux
clients construits directement dans Beaver sous Windows. Sélectionner Rustls
partout ailleurs demanderait de modifier plusieurs dizaines de chemins et
créerait deux autorités sur la politique TLS. Cette portée dépasse le bug.

### 3. Cache négatif ou filtre global — écarté

Un cache réduirait les tentatives sans supprimer la première trace et pourrait
retarder la découverte d'un serveur réparé. Un filtre permanent sur la cible
masquerait aussi les véritables erreurs TLS des autres services. Le filtre
retenu est donc conditionné à la tâche et à la cible exacte.

## Architecture retenue

`services::app_log` reste l'autorité unique du filtrage des traces. Il possède :

- la cible Windows exacte à ignorer ;
- la portée asynchrone réservée à l'échec TLS attendu de la découverte locale ;
- la décision pure qui autorise ou refuse une métadonnée de journal.

`local_site_probe` ne connaît pas la règle du logger. Il demande seulement à
exécuter l'envoi HTTPS dans la portée nommée. Le flux reste :

1. l'interface demande un scan toutes les cinq secondes quand l'accueil est
   visible ;
2. le backend énumère une liste bornée de ports locaux ;
3. le probe essaie HTTP puis HTTPS avec les délais existants ;
4. l'envoi HTTPS local est exécuté dans la portée de résultat attendu ;
5. le certificat non approuvé est refusé et le probe retourne `Err(())` ;
6. le logger ignore uniquement la trace Windows produite dans cette portée.

Aucun état persistant, cache ou collection n'est ajouté. Le moteur TLS et le
magasin de confiance ne changent sur aucun système.

## Tests obligatoires

Tu écris chaque test avant son correctif et tu observes son échec pour la raison
attendue.

Le test du journal prouve séparément que :

- la cible Windows du vérificateur est conservée hors de la portée ;
- cette cible est ignorée dans la portée ;
- une autre cible reste conservée dans la même portée.

Un test Windows lance un vrai serveur HTTPS local avec un certificat
auto-signé. Il exécute le probe dans un sous-processus qui possède son logger,
afin de ne pas modifier le logger global des autres tests. Il prouve que :

- le certificat auto-signé est refusé ;
- le site n'est pas retourné ;
- la trace réelle du vérificateur a rencontré le filtre mais n'a pas été émise ;
- le sous-processus se termine dans une limite de temps stricte.

Tu conserves et exécutes les tests HTTP existants. Tu exécutes ensuite le
profil Windows complet, le formatage et le lint Rust strict.

## Sécurité

Tu n'utilises ni `danger_accept_invalid_certs`, ni vérificateur permissif, ni
autorité auto-signée dans le client de production. Le test génère une clé
éphémère qui n'est ni persistée ni écrite dans les traces.

Les limites existantes restent inchangées : candidats, concurrence,
redirections, taille de réponse, délai de connexion et délai total.

## Critères d'acceptation

- Un serveur HTTPS local auto-signé n'apparaît pas dans les sites détectés.
- Son scan ne produit plus la trace répétée
  `rustls_platform_verifier::verification::windows`.
- La même cible reste journalisée hors du probe local.
- Un serveur HTTP local valide reste détecté.
- La navigation CEF, le moteur TLS et les autres clients HTTPS ne changent pas.
- Aucun filtre permanent, cache négatif ou assouplissement TLS n'est ajouté.
- Chaque fichier de code garde une responsabilité unique et moins de 230 lignes.

## Hors périmètre

Tu ne changes pas l'intervalle de cinq secondes, l'interface du panneau, la
détection des ports, la navigation volontaire vers un site ni la politique TLS
générale de Beaver.
