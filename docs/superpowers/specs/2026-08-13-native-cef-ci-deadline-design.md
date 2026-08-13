# Stabilisation du parcours CEF natif macOS

## Constat

Le parcours `native-cef-shutdown` possède une limite Mocha de 60 secondes,
alors que ses attentes locales peuvent dépasser cette durée. Plusieurs appels
IPC ne sont pas bornés et aucune trace ne nomme l'étape active. Un échec finit
donc par un message global qui ne distingue pas un blocage du démarrage, du
chargement CEF ou de la fermeture.

Le même arbre Git a réussi deux fois en 3 à 10 secondes puis a échoué deux fois
à 60 secondes. Tu traites donc d'abord l'instabilité et le manque de preuve du
test ; tu ne modifies pas le comportement de Beaver sans avoir identifié une
étape fautive.

## Décision

Tu donnes au parcours une seule autorité temporelle. Elle crée une échéance
monotone au début du test. Chaque étape reçoit le minimum entre son plafond et
le temps global restant ; aucune étape ne crée son propre budget indépendant.

Tu traces chaque transition sous une forme structurée et bornée : identifiant
du parcours, nom d'étape, état `started`, `completed` ou `failed`, durée et code
d'erreur générique. Tu n'écris aucun chemin, URL ou contenu de page dans ces
traces.

Tu conserves un petit délai externe à Mocha uniquement pour laisser au `finally`
le temps de fermer le serveur local après l'expiration contrôlée du parcours.
Ce délai dérive de la même politique et ne constitue pas un deuxième budget de
travail.

## Structure

- `scripts/e2e/native-journey-deadline.mjs` possède l'échéance, exécute une
  étape bornée et produit les événements structurés.
- `scripts/e2e/native-journey-deadline.test.mjs` prouve les calculs de budget,
  l'échec borné et les traces de succès ou d'échec.
- `tests/e2e/native-cef-shutdown.spec.ts` orchestre le scénario existant en
  passant chaque frontière asynchrone par cette autorité.
- `wdio.conf.ts` dérive le délai Mocha de la politique exportée.

Les dépendances vont du scénario et de la configuration WebDriver vers la
politique de parcours. La politique ne dépend ni de WebDriver, ni de Tauri, ni
de CEF afin que ses garanties soient testées localement.

## Gestion des échecs

Une expiration est une panne récupérable du test : tu interromps l'attente,
tu émets une trace `failed` avec le code `stage-timeout`, puis le `finally`
exécute le nettoyage borné. Une exception de l'opération conserve son objet
technique pour le rapport de test mais reçoit le code de trace générique
`stage-error`.

Si l'échéance globale est déjà expirée, tu ne démarres pas l'étape suivante.
Tu échoues immédiatement avec son nom afin que la dernière frontière franchie
reste visible.

## Critères d'acceptation

- Une mutation qui redonne un budget neuf à chaque étape fait échouer un test.
- Une opération qui ne se termine pas échoue au plafond de son étape, avant le
  délai Mocha externe.
- Les traces nomment la dernière étape commencée et indiquent si elle a réussi
  ou échoué.
- Les événements et les noms d'étapes sont bornés et validés à leur frontière.
- Le parcours CEF conserve ses preuves : vraie page, vrai helper, chargement
  terminé et absence de processus possédé après la fermeture.
- Aucun fichier de code modifié ne dépasse 230 lignes.

## Hors périmètre

Tu ne corriges pas encore un blocage interne à Beaver : la prochaine exécution
macOS doit d'abord identifier précisément cette éventuelle cause. Tu ne relances
pas automatiquement le job et tu n'augmentes pas simplement son délai.
