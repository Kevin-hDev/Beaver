# E10 — Compatibilité des formats historiques

Date : 17 septembre 2026
Commit de départ : `a943ff75`

## Décision

Conserver la migration des profils de compression v1, la migration des sessions v3 et la lecture des journaux Ollama sans `rejected_target`. Ces formats n'ont pas été inclus dans une version publique connue, mais ils ont pu être écrits par des builds de développement. Leur suppression ferait perdre des réglages, classerait des conversations comme corrompues ou bloquerait une reprise sûre pour un gain limité.

Les sessions v7 et les automatisations v2 sont livrées et restent hors des candidats au retrait.

## Profils de compression v1

Le format v1 a existé entre les commits du 30 et du 31 août 2026. Le tag `v1.1.9` ne contient pas ce magasin, tandis que `v1.2.0` utilise déjà le schéma 2. L'absence de release intermédiaire prouve qu'il n'a pas été publié, pas qu'il n'a jamais été écrit.

Le chemin actuel sauvegarde les octets exacts dans `compression-profiles.v1.bak`, migre identité, nom, prompts, politique et sélection, puis ne supprime la sauvegarde qu'après un lancement ultérieur réussi. Remplacer ce chemin par les valeurs par défaut ferait perdre silencieusement des choix d'un build de développement. Il reste donc conservé.

## Sessions v3

La v3 corrige le garde de compression avant d'appliquer les migrations v5, v6 et v7. Une session n'est migrée durablement qu'à sa prochaine écriture ; un document créé par un build de développement peut donc rester en v3 pendant longtemps.

Le poste examiné ne contient actuellement aucune session v3 : 15 documents déclarent v5 et 15 v6. Il contient toutefois quatre sauvegardes v2 et neuf sauvegardes v4, preuve locale que les formats de développement et leurs migrations ont réellement circulé. Ce relevé ne permet pas de conclure pour les autres machines. Le parseur, la transformation et la sauvegarde v3 restent donc nécessaires.

## Journaux Ollama avec `rejected_target: null`

La production actuelle écrit toujours une empreinte `Some`. Le parseur accepte néanmoins `null` pour les deux phases de retour arrière prévues par le schéma initial.

La décision de reprise sans empreinte est volontairement étroite :

- si seul le bundle précédent exact est actif, le journal terminé peut être retiré ;
- si seul le précédent exact est en sauvegarde, le retour arrière peut reprendre ;
- toute présence d'un bundle rejeté ou toute disposition ambiguë exige une intervention, car Beaver ne possède pas son identité.

Rendre le champ obligatoire transformerait un document ancien lisible en journal invalide sans réduire la logique de disposition qui doit déjà rester prudente. La lecture tolérante et les branches `None` sont conservées.

## Politique retenue

- lire et migrer tous les formats déjà acceptés ;
- écrire uniquement les formats courants ;
- garder une sauvegarde exacte avant transformation ;
- ouvrir les versions futures en lecture seule quand le contrat le permet ;
- ne jamais revenir aux valeurs par défaut pour simplifier une migration valide ;
- ne retirer un format qu'avec une décision explicite de rupture et une procédure de récupération documentée.

## Validation

Les suites ciblées réussissent :

- profils de compression : 13 tests ;
- migrations de sessions v1 à v7 et versions futures : 34 tests ;
- documents de journal Ollama : 10 tests ;
- décisions de reprise Ollama : 42 tests.

Total : 99 tests réussis, aucun échec. Les tests couvrent les sauvegardes exactes, les échecs avant publication, les formats futurs, v3 vers v7, le champ absent et les dispositions ambiguës.
