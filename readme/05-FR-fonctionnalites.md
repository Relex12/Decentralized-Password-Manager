# Fonctionnalités

Description des fonctionnalités additionnelles du gestionnaire de mot de passe sans modifier la communication entre les appareils

[Menu principal](https://relex12.github.io/fr/Decentralized-Password-Manager)

[Page précédente : Sécurité](https://relex12.github.io/fr/Decentralized-Password-Manager/Securite)

## Sommaire

[toc]

## Développement

### Plateformes

Le gestionnaire de mots de passe doit être utilisable sur le plus d'appareils possible sans restriction. Pour les téléphones mobiles, une application devra être disponible sur Android dans un premier temps, dans le soucis de supporter iOS avec la même application. Pour les ordinateurs, une application de bureau pourrait exister pour Windows, Linux et macOS, ainsi qu'une version web pour une meilleure compatibilité via les navigateurs Google Chrome et Firefox au minimum, d'autres navigateurs sous Chromium si possible. Enfin, une application en interface de ligne de commande sera disponible pour les systèmes UNIX/Linux, à des fins de développement notamment.

### Langage

Le socle commun de l'implémentation des clients et du serveur sera développé en JavaScript (TypeScript etc.) ou autre langage qui offre le même niveau de compatibilité. Pour sa gestion fine de la mémoire, le serveur pourra être développé en Rust ou autre langage assez bas niveau.

### Licence et accès des sources

Le but du projet est de fournir un gestionnaire de mots de passe qui soit libre et dont le fonctionnement fasse office de standard. La licence sera donc libre et héréditaire. Le code source devra rester ouvert afin de permettre l'exploration de failles de sécurité par la communauté. L'implémentation du serveur et des clients est indissociable du gestionnaire de mots de passe, il y aura donc des implémentations officielles, mais la communauté pourra créer ses propres implémentations qui respectent les spécifications décrites. Le gestionnaire de mots de passe possèdera une ou plusieurs instances officielles, mais il sera donné la possibilité aux utilisateurs qui le souhaitent d'héberger leur propre serveur, pour les structures qui souhaitent conserver la gestion de leurs données notamment.

## Gestion des appareils

### Démarrage en tâche de fond et récupération passive

Dès que l'appareil de l'utilisateur est démarré, le client se lance en tâche de fond. Pour les appareils qui le peuvent, notamment sur téléphone mobile, les clients mettront en place une communication qui permet au serveur d'envoyer des messages au client sur base du hachage de son identifiant utilisateur, en conservant l'anonymat. Pour les appareils qui ne supportent pas le serveur push, c'est-à-dire la réception d'échanges à l'initiative du serveur, les clients devront envoyer au serveur des requêtes de récupération des mises à jours régulièrement, entre 15 et 60 min.

En plus de cela, les clients peuvent déclencher une nouvelle tentative de récupération de mise à jour lorsque l'utilisateur interagit avec, en conservant un délai minimum entre les requêtes. 

Lorsqu'un client récupère une mise à jour sans action de l'utilisateur, les messages en attente sont stockés chiffrés à côté de la mémoire du gestionnaire de mots de passe. La prochaine fois que l'utilisateur accède au coffre, les messages en attente sont déchiffrés et les modifications sont appliquées au coffre.

### État des autres clients

Le client d'un coffre de mots de passe peuvent faire une requête vers le serveur afin de savoir quels autres clients du même coffre ont des messages en liste d'attente qui doivent être récupérés. Cela permet à l'utilisateur de déterminer à partir de n'importe quel appareil le nombre d'appareils qui ne sont pas à jours et le temps qui lui reste pour les synchroniser. 

### Multi-coffre

Chaque client peut être membre de différents coffres de mots de passe, chaque coffre n'étant pas nécessairement partagé entre les mêmes appareils. Un utilisateur qui aurait différents cas d'usage peut ainsi créer différents coffres pour chacun d'entres eux, pour séparer les secrets personnels et professionnels par exemple.

> Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Appareil maître et gestion des appareils

En plus d'une utilité lors de la découverte des appareils suite à la création d'un coffre ou l'ajout d'un appareil, un appareil maître peut servir à l'utilisateur à gérer ses appareils. L'utilisateur choisi parmi ses appareils lequel est en mesure d'accorder le partage à de nouveaux appareils, par exemple un téléphone mobile. L'appareil maître est alors en mesure de supprimer des appareils du coffre, lorsque l'utilisateur possède un nouvel appareil par exemple.

Un appareil de secours au sein du coffre peut également être choisi par l'utilisateur, seul appareil capable de devenir maître à la place du maître, en cas de perte ou de vol par exemple.

>  Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Clients copie et proxy

Un appareil maître peut également accorder à des clients extérieurs au coffre la possibilité de devenir des clients copie. Un client copie ne fait pas partie de la liste des clients d'un coffre, il n'en reçoit pas directement les mises à jour, mais le client maître peut les lui transmettre lorsque lui-même les découvre. Le client maître devient un proxy pour le client copie : le client copie se situe derrière le proxy et ne peut accéder aux mises à jour que lorsque celui-ci accepte de les lui transmettre.

Comme les clients copie ne font pas partie du coffre, ils ne peuvent pas être mis à jour en utilisant la couche service du gestionnaire de mots de passe, ils sont obligatoirement synchronisés manuellement depuis l'appareil maître, via Bluetooth, QR Code ou protocole ICE.

>  Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Partage de mots de passe

Le partage d'un mot de passe avec d'autres personnes ne relève pas des bonnes pratiques vis-à-vis de la sécurité, mais c'est pourtant une pratique courante lorsque différentes personnes partagent un compte identique. Afin d'éviter le recours à un mot de passe plus faible, les clients du gestionnaire de mots de passe auront la possibilité de partager un ou plusieurs secrets du coffre avec des appareils n'appartenant pas au coffre. Le partage de mot de passe n'équivaut pas à un nouveau coffre : chaque utilisateur va pouvoir lier ce mot de passe avec les appareils d'autres utilisateurs. Dès que le mot de passe fait l'objet d'une mise à jour dans l'un des coffres, le client qui possède la liaison vers un client lié va lui transmettre la mise à jour, à réception le client lié va propager la mise à jour au sein de son propre coffre.

Le partage de mots de passe ressemble à l'utilisation d'un client copie, sauf qu'ici tous les clients liés sont des proxys : la mise à jour du mot de passe peut provenir de n'importe quel coffre auquel le mot de passe est lié, lorsque l'appareil lié du coffre en question reçoit la mise à jour, il la propage aux autres appareils liés qui font office de proxy pour leurs coffres respectifs.

La mise à jour du mot de passe partagé entre les appareils liés passe par la couche service. Si possible, la mise à jour est chiffrée de bout en bout avec une clé partagée entre tous les appareils liés, sinon elle est chiffrée de façon pair-à-pair et stockée plusieurs fois sur le serveur avec une date de péremption plus courte.

### Appareil de secours extérieur

Les clients d'un coffre peuvent ajouter un client extérieur comme client de secours. Le client de secours est ajouté au coffre comme un secret, afin d'être partagé et mis à jour entre les clients comme le reste du coffre. L'appareil de secours extérieur peut permettre de récupérer le mot de passe maître du coffre d'un utilisateur en cas d'oubli ou malheureusement de décès.

>  quid du cas de la perte de tous les appareils ?

Pour enregistrer un appareil de secours extérieur, ce client maître doit ajouter le hachage de l'identifiant d'enregistrement de l'appareil de secours et calculer une partie du mot de passe maître. La récupération du mot de passe maître se fait grâce au calcul multipartite sécurisé ou à un cryptosystème à seuil, c'est-à-dire au calcul d'une valeur secrète qui nécessite la collaboration de toutes les personnes qui possèdent une partie du secret. Ici, l'appareil maître calcule les deux parties nécessaires à la reconstitution du mot de passe, il partage une partie aux autres appareils en l'ajoutant au coffre, puis il transmet l'autre partie à l'appareil de secours extérieur qui devra obligatoirement la conserver au sein d'un coffre. Pour les appareils appartenant au coffre auquel est rattaché l'appareil de secours, la partie doit également être stockée sans chiffrement, afin d'être accessible lorsque le client est verrouillé.

Lorsque l'utilisateur d'un appareil de secours extérieur souhaite récupérer le mot de passe maître du coffre dont il est le secours, il doit manipuler l'un des appareils appartenant au coffre et initier une procédure de récupération de mot de passe maître. Le client va alors envoyer un message en utilisant la couche service du gestionnaire de mots de passe pour transmettre sa partie au client de secours extérieur. Le serveur ne doit rendre disponible le message pour le client de secours qu'après un délai, pour éviter que l'utilisateur de confiance ne fasse la procédure sans l'accord de l'utilisateur. Une fois que le message est disponible, le client de secours extérieur peut le récupérer et calculer le mot de passe maître.

Suite à cette procédure de récupération, le mot de passe maître doit être changé car il n'est plus connu du seul utilisateur.

Un appareil de secours n'est lié qu'à un seul coffre de mots de passe. Si l'utilisateur possède plusieurs coffres qui ont chacun un mot de passe maître différent, alors il faut réitérer l'opération autant de fois qu'il y a de coffre. L'appareil de secours extérieur peut être identique pour chaque coffre.

Si l'appareil de secours extérieur venait à ne plus être utilisable, pour cause de perte, de vol ou de renouvellement, il faut également recommencer la procédure. L'appareil maître doit alors mettre à jour le secret dans coffre pour les autres appareils.

> Cette fonctionnalité doit faire l'objet d'une étude plus aboutie sur sa faisabilité.

