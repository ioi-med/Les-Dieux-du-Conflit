# Les Dieux du Conflit — Le Prix du Silence

Version 3.5.0 — jeu d'action-aventure en Rust, pour Windows 64 bits.

## Une histoire plus longue

Le dieu du Capitalisme vous engage pour vaincre le dieu du Communisme et promet des remèdes aux habitants. Au fil de votre voyage, Sélène cherche son frère disparu tandis que vous découvrez des dettes fictives, des remèdes confisqués et des contrats falsifiés. Les deux dieux ont chacun causé des souffrances. À la fin de votre mission, choisissez d'accepter la récompense ou de trahir votre employeur et de l'affronter.

L'aventure compte **24 chapitres principaux et un chapitre supplémentaire si vous trahissez le dieu du Capitalisme**. Chaque chapitre comprend quatre quartiers. Ils réutilisent des types d'arènes et d'ennemis avec différentes compositions, des documents à collecter, des passages à sauter, des épreuves de résistance, un duel et deux boss. Les décors varient selon les quartiers : rues, jardins, égouts, usines, archives et palais.

Cette version a été conçue comme une aventure nettement plus longue visant une à deux heures. **La durée n'a pas été mesurée sur une partie complète et ne peut pas être garantie** : elle dépend du joueur, des défaites et de l'exploration.

## Installation

- Il faut Windows 10 ou 11, 64 bits et une carte graphique compatible OpenGL.
- Si une version précédente est installée, fermez-la puis désinstallez-la dans **Applications installées** ou **Programmes et fonctionnalités** avant d'installer cette version. Le MSI ne met pas automatiquement l'ancienne version à jour.
- Lancez `Les-Dieux-du-Conflit-Setup.msi` pour installer le jeu et créer son raccourci dans le menu Démarrer. Vous pourrez le désinstaller depuis les applications installées.
- Vous pouvez aussi lancer directement `Les-Dieux-du-Conflit.exe` sans installation ; les ressources, les sons et l'icône sont intégrés.
- Les fichiers ne sont pas signés : Windows peut afficher un avertissement. Fermez le jeu avant de le désinstaller.
- La fenêtre s'adapte au redimensionnement en conservant les proportions ; des bandes latérales ou horizontales peuvent apparaître.

## Commandes

| Action | Touches |
|---|---|
| Nouvelle partie / continuer le dialogue / recommencer | Entrée |
| Reprendre une sauvegarde depuis le menu | C |
| Déplacement | ZQSD ou WASD |
| Attaque normale | R, à presser ou à maintenir |
| Saut / esquive | Espace |
| Flamme verte | E, apprise au chapitre 4 |
| Vague de sang | T, apprise au chapitre 13 ; elle traverse plusieurs ennemis |
| Interaction (escaliers, serment du Panthéon) | E |
| Jugement des âmes (dieu de la justice, une fois par quartier) | Y |
| Pause / reprendre | Échap |
| Depuis la pause : retourner au menu | Q |
| Depuis la pause : couper ou rétablir le son | M |
| Depuis la pause : baisser ou monter le volume | Moins (-) ou Égal (+) |
| Depuis la pause : saisir un code | F2, puis lettres et chiffres, Entrée pour valider |
| Duel : choisir ordinateur ou 2 joueurs | 2 sur l'écran du chapitre 2 |
| Duel local, joueur 2 | Flèches et Maj droite ou K pour frapper |
| Choix final | A pour accepter ou T pour trahir |

## Second cycle : devenir dieu de la justice

À la fin de l'aventure, l'écran de fin propose le trône de **dieu de la justice**. En appuyant sur **J**, tout recommence au chapitre 1, mais le monde a changé :

- **Les ennemis ordinaires ne peuvent plus vous blesser.** Leurs coups, leurs os, leurs pièces et leurs immeubles traversent le dieu que vous êtes.
- **Certains ennemis sont promus demi-dieux**, un sur trois environ : ils portent un liseré doré et la mention DEMI-DIEU. Eux seuls, avec les dieux, peuvent encore vous atteindre. Ils ont aussi 60 % de vie en plus et frappent plus vite.
- **Les deux dieux reviennent en zombies** : le dieu du Communisme comme celui du Capitalisme ont **deux fois plus de vie, se déplacent et tirent deux fois plus vite, et frappent deux fois plus fort**. Ils sont signalés par la mention ZOMBIE et un halo vert.
- La progression du personnage repart de zéro et se reconstruit comme au premier cycle. Le cycle choisi est conservé par la sauvegarde.

### Le Jugement des âmes

Le dieu de la justice dispose d'un pouvoir unique, **une seule fois par quartier** : appuyez sur **Y** pour juger l'âme vivante la plus proche.

- **Âme morte-vivante** (squelette, garde, ou dieu revenu en zombie) : elle est reconnue coupable et **tombe aussitôt à la moitié de sa vie**. Sur un dieu-zombie, cela retire donc la moitié de ses points de vie.
- **Âme vivante** (slime, spectre, ou tout autre ennemi encore vivant) : le jugement la **double** au lieu de la punir, c'est-à-dire que sa vitesse, sa cadence de tir, ses dégâts et sa frappe au contact sont multipliés par deux. Elle porte alors la mention JUGÉ ×2. À vous de choisir qui vous jugez.
- Le compteur **JUGEMENT** en haut à droite indique s'il vous reste le pouvoir dans ce quartier. Il revient à chaque nouveau quartier, jamais au milieu d'un quartier.
- Le pouvoir n'existe que dans le cycle de la justice ; les autres personnages n'y ont pas accès.

## Progression du personnage

Chaque ennemi vaincu rend le personnage plus fort, définitivement et sans menu :

- **Tous les dix ennemis vaincus**, la vie maximale augmente de 5 points, et ces points sont rendus immédiatement. Le premier palier donne donc 105 points de vie, le deuxième 110, et ainsi de suite.
- **À chaque victoire**, les coups portent un peu plus fort : environ +4 % par ennemi, pour l'épée comme pour la flamme et la vague.
- Les paliers s'arrêtent à 100 victoires (vie maximale 600, coups environ quatre fois plus forts). Le code **INV** va au-delà.
- La progression est conservée par la sauvegarde et affichée en haut de l'écran : ennemis vaincus, dégâts des coups, prochain palier.

## Jouer

Éliminez les ennemis et ramassez les documents demandés, puis rejoignez la porte à droite lorsqu'elle s'ouvre. Certains quartiers demandent aussi de résister aux renforts pendant un temps donné. Sautez les failles dans les tunnels ; les marqueurs verts de soins restaurent 25 points de vie, dans la limite de 100. Des soins apparaissent dans certains quartiers et après certains ennemis vaincus.

### Ce que le jeu ne dit pas

Certaines choses ne sont expliquées nulle part, ni au menu ni dans l'interface. Une zone hors de l'aire de jeu vous emmène ailleurs : c'est à vous de la trouver, comme la manière d'en sortir et ce que l'on y gagne. Pour cette raison, la suite de cette section reste volontairement muette sur le sujet. **Ne lisez pas la section suivante si vous préférez chercher par vous-même.**

<details>
<summary>Solution complète, à ouvrir seulement si vous êtes bloqué</summary>

1. Franchissez n'importe quelle bordure de l'aire de combat : vous tombez dans les Backrooms.
2. Les Backrooms comportent dix-huit niveaux. Chaque niveau est un dédale ; un escalier discret (marches et flèche) permet de monter, un second escalier, sur certains niveaux, permet de descendre. **E** agit sur un escalier.
3. Le niveau 18 s'ouvre sur le Panthéon. Là, **E** vous fait prêter serment et devenir demi-dieu ; **R** vous renvoie dans les Backrooms sans transformation.
4. Revenir à la surface se fait avec **Ctrl + G**. Au premier retour, vous rapportez la **Glace éternelle**. **X** emprisonne dans la glace et vainc en un coup l'adversaire vivant le plus proche.
4 bis. Une fois demi-dieu, les bordures ne vous emmènent plus nulle part : elles redeviennent de simples limites, et les Backrooms vous sont fermées. C'est le prix du serment.
4 ter. Dans le menu pause, **F2** ouvre un champ de code.
   - **ML** mène directement au chapitre 23, le dieu du Communisme, flamme et vague déjà débloquées.
   - **INV** donne **999 points de vie maximaux** et des **coups à 999**, à l'épée comme avec les sorts. On reste dans sa partie en cours.
   - **GLACE** donne **999 utilisations** de la Glace éternelle. Le compteur passe dans l'interface et la réserve reste exacte après sauvegarde et reprise : elle ne redescend qu'en jouant. Les dieux restent protégés tant que vous n'êtes pas demi-dieu.
5. Un mortel ne peut **pas** geler un dieu : le pouvoir se brise sur les deux boss. Seul un demi-dieu venu du Panthéon peut geler un dieu, et ce droit reste acquis même après avoir quitté le Panthéon.
6. La Glace éternelle n'a qu'une utilisation par partie (sauf code **GLACE**). Si aucun adversaire n'est présent, elle n'est pas consommée. Une nouvelle partie la remet à zéro ; l'aventure en cours mémorise l'étage des Backrooms déjà atteint et l'état de la Glace.
7. L'air des Backrooms fatigue lentement. Chaque escalier rend un peu de souffle.

</details>

## Sons

**Correction de la version 3.1.0 :** la version 3.0.0 ne jouait aucun son. La bibliothèque audio de Macroquad n'était pas activée à la compilation : les fichiers étaient chargés mais aucune sortie n'était branchée. L'option `audio` est maintenant activée dans `Cargo.toml`, le volume des ambiances et des effets a été relevé, et le jeu signale dans la console combien d'effets et de musiques ont réellement été chargés.

Le jeu contient des ambiances et des effets sonores libres, intégrés à l'exécutable : musique d'ambiance au menu, à la surface, dans les Backrooms et au Panthéon selon la situation, et effets pour les coups, les sorts, les soins, les portes, la glace, les passages et l'évolution en demi-dieu. Sans périphérique audio, le jeu reste silencieux sans erreur. Les sources, auteurs et licences sont dans `assets/audio/CREDITS-AUDIO.md` du dossier des sources : musiques du **Dark Sci-Fi Audio Pack** de SRG774, **Dark Ambience** de SpringySpringo, sons du **RPG Sound Pack** d'artisticdude, de **Juhani Junkala** et de **Kenney**, tous sous licence CC0.

## Sauvegarde

Le jeu sauvegarde automatiquement le chapitre, le quartier, le choix du duel, le niveau des Backrooms déjà atteint et l'état de vos pouvoirs. La sauvegarde a lieu au début de chaque quartier, à chaque escalier franchi, et lors de l'obtention, de l'utilisation ou de la perte d'un pouvoir. Au menu, **C** reprend la sauvegarde. Le quartier repris recommence avec toute votre vie : les ennemis et les documents de ce quartier réapparaissent. Une nouvelle partie remplace la sauvegarde.

Sous Windows, la sauvegarde se trouve dans `%APPDATA%\LesDieuxDuConflit\aventure-v3.txt`. Le code saisi dans le menu pause est sauvegardé comme un changement de chapitre : il reste donc possible de reprendre plus tard à cet endroit. Désinstaller le jeu ne supprime pas volontairement cette sauvegarde. Si le message « SAUVEGARDE IMPOSSIBLE » apparaît dans le jeu, vérifiez les droits d'écriture de ce dossier. Les sauvegardes des versions 2.x ne sont pas importées.

## Vérifications et limites

- Trente-neuf tests automatiques Rust couvrent les 25 chapitres et les 100 quartiers, les deux boss, les soins, la mêlée, le redimensionnement, la sauvegarde, les dix-huit étages des Backrooms, les escaliers, le Panthéon et la règle du gel des dieux.
- Une simulation automatisée lance la vraie boucle du jeu, force les objectifs des 100 quartiers jusqu'à la fin, parcourt les 18 étages jusqu'au Panthéon, puis refait les 100 quartiers du cycle de la justice. Ce n'est pas une partie jouée normalement.
- Des essais graphiques sous Linux ont vérifié le début de l'histoire, les Backrooms à plusieurs niveaux, le retour Ctrl + G, la Glace, le refus de geler un dieu, le Panthéon et le serment de demi-dieu.
- Les sons : la compilation inclut désormais le moteur audio. Lancée sous Wine, la version Windows sort **du son réel et continu** (capture de 25 secondes, 140 Mo d'échantillons, signal de crête présent). 17 effets sur 17 et 8 musiques sur 8 sont chargés au démarrage, et le jeu continue sans erreur si aucun périphérique audio n'est présent. L'environnement de développement n'a pas de carte son pour Linux (aucun module noyau disponible), donc le rendu n'a pas pu être écouté à l'oreille ici : le contrôle a été fait par analyse du signal. Un réglage de volume général est disponible dans la pause (- et +), en plus de la coupure du son avec M.
- Le Jugement des âmes est couvert par quatre tests (moitié de vie des morts-vivants, doublement des vivants, dieu-zombie traité comme mort-vivant, un seul usage par quartier).
- Le second cycle est couvert par quatre tests (dégâts bloqués puis autorisés, promotion en demi-dieu, dieux-zombies à deux fois plus de vie et de force, sauvegarde) et par une simulation qui parcourt les 100 quartiers du cycle de la justice jusqu'à la fin.
- La progression a été vérifiée à l'image et par tests : 27 dégâts au départ, palier de vie tous les dix ennemis, sauvegarde de la puissance, code INV à 999/999 et 999 dégâts. Le menu pause a été vérifié à l'image : saisie masquée, code **ML** accepté (chapitre 23 avec les deux sorts), code **GLACE** accepté avec le compteur affiché à 999. Des tests vérifient la réserve de 999 charges, sa consommation une par une, son écriture exacte en sauvegarde et le maintien du droit de geler les dieux après le serment.
- Un demi-dieu ne franchit plus les bordures vers les Backrooms ; un test automatique le vérifie, et un autre vérifie qu'un mortel y tombe toujours.
- Installation, démarrage et désinstallation du MSI testés sous Wine. La version n'a pas été testée sur un PC Windows réel ni chronométrée de bout en bout.

## Code source et ressources

`Sources-Les-Dieux-du-Conflit.zip` contient le projet Rust, les sprites fournis, la police, les sons et leurs licences, ainsi que le fichier de construction de l'installateur. Lancez `cargo test` pour les tests ou `cargo build --release` pour compiler. Sous Linux, l'exécutable Windows GNU est construit avec `cargo build --release --target x86_64-pc-windows-gnu` et un compilateur MinGW ; `wixl` utilise ensuite `windows/installateur.wxs` pour produire le MSI. Des essais sont disponibles en lançant le jeu avec `--self-test` (progression) ou `--audio-check` (chargement des sons) dans un environnement graphique.

La police DejaVu Sans est accompagnée de sa licence dans `assets/DejaVu-LICENSE.txt`. Le jeu utilise la bibliothèque libre Macroquad et ses dépendances.

Codé par M.Mehdi TANNOUBI et arena.ai
