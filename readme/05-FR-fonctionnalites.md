# Fonctionnalités

Description des fonctionnalités additionnelles du gestionnaire de mots de passe sans modifier la communication entre les appareils

[Menu principal](https://relex12.github.io/fr/Decentralized-Password-Manager)

[Page précédente : Implémentation](https://relex12.github.io/fr/Decentralized-Password-Manager/Implementation)

## Sommaire

* [Fonctionnalités](#fonctionnalités)
  * [Sommaire](#sommaire)
  * [Développement](#développement)
    * [Plateformes](#plateformes)
    * [Langage](#langage)
    * [Licence et accès des sources](#licence-et-accès-des-sources)
  * [Gestion des appareils](#gestion-des-appareils)
    * [Démarrage en tâche de fond et récupération passive](#démarrage-en-tâche-de-fond-et-récupération-passive)
    * [État des autres clients](#état-des-autres-clients)
    * [Informations d'utilisation](#informations-dutilisation)
    * [Multi-coffre](#multi-coffre)
    * [Appareil maître et gestion des appareils](#appareil-maître-et-gestion-des-appareils)
    * [Clients copie et proxy](#clients-copie-et-proxy)
    * [Partage de mot de passe](#partage-de-mot-de-passe)
    * [Appareil de secours extérieur](#appareil-de-secours-extérieur)
    * [Client de sauvegarde](#client-de-sauvegarde)
  * [Sécurité](#sécurité)
    * [Saisie de mot de passe maître](#saisie-de-mot-de-passe-maître)
    * [Changement du mot de passe maître](#changement-du-mot-de-passe-maître)
    * [Double authentification](#double-authentification)
    * [Générateur de mot de passe](#générateur-de-mot-de-passe)
    * [Outils de détection de mot de passe faible](#outils-de-détection-de-mot-de-passe-faible)
    * [Numéro de sécurité](#numéro-de-sécurité)
    * [Chiffrement du stockage multi-niveau](#chiffrement-du-stockage-multi-niveau)
  * [Communication entre appareils](#communication-entre-appareils)
    * [Découverte Bluetooth, caméra et ports](#découverte-bluetooth-caméra-et-ports)
    * [Découverte réseau](#découverte-réseau)
    * [Support du protocole ICE](#support-du-protocole-ice)

<!-- table of contents created by Adrian Bonnet, see https://Relex12.github.io/Markdown-Table-of-Contents for more -->

## Développement

### Plateformes

Le gestionnaire de mots de passe doit être utilisable sur le plus d'appareils possible sans trop de restrictions. Pour les téléphones mobiles, une application devra être disponible sur Android dans un premier temps, avec le soucis de supporter iOS depuis la même application. Pour les ordinateurs, une application de bureau pourrait exister pour Windows, Linux et macOS, ainsi qu'une version web pour une meilleure compatibilité via les navigateurs Google Chrome et Firefox au minimum, d'autres navigateurs sous Chromium si possible. Enfin, une application en interface de ligne de commande sera disponible pour les systèmes UNIX/Linux, à des fins de développement notamment.

### Langage

Le socle commun de l'implémentation des clients et du serveur sera développé en JavaScript (TypeScript etc.) ou autre langage qui offre le même niveau de compatibilité. Pour sa gestion fine de la mémoire, le serveur pourra être développé en Rust ou autre langage assez bas niveau.

### Licence et accès des sources

Le but du projet est de fournir un gestionnaire de mots de passe qui soit libre et dont le fonctionnement fasse office de standard. La licence sera donc libre et héréditaire. Le code source devra rester ouvert afin de permettre l'exploration de failles de sécurité par la communauté. L'implémentation du serveur et des clients est indissociable du gestionnaire de mots de passe, il y aura donc des implémentations officielles, mais la communauté aura la possibilité de créer ses propres implémentations qui respectent les spécifications décrites. Le gestionnaire de mots de passe possèdera une ou plusieurs instances officielles, mais il sera donné la possibilité aux utilisateurs qui le souhaitent d'héberger leur propre serveur, pour les structures qui souhaitent conserver la gestion de leurs données notamment, comme des entreprises ou des institutions.

## Gestion des appareils

### Démarrage en tâche de fond et récupération passive

Dès que l'appareil de l'utilisateur est démarré, le client se lance en tâche de fond. Pour les appareils qui le peuvent, notamment sur téléphone mobile, les clients mettront en place une communication avec le serveur qui permet à celui-ci d'envoyer des messages au client sur base du hachage de son identifiant utilisateur, en conservant l'anonymat. Pour les appareils qui ne supportent pas le *serveur push*, c'est-à-dire la réception d'échanges à l'initiative du serveur, les clients devront envoyer au serveur des requêtes de récupération des mises à jours régulièrement mais suffisamment espacées pour ne pas surcharger le serveur. Le délai entre deux requêtes de récupération pourrait être de l'ordre de 30 min. En plus de cela, les clients peuvent déclencher une nouvelle tentative de récupération de mise à jour lorsque l'utilisateur interagit avec, en conservant un délai minimum entre les requêtes.

Lorsqu'un client récupère une mise à jour sans action de l'utilisateur, les messages en attente sont stockés chiffrés à côté de la mémoire du gestionnaire de mots de passe. La prochaine fois que l'utilisateur accède au coffre, les messages en attente sont déchiffrés et les modifications sont appliquées au coffre.

### État des autres clients

Le client d'un coffre de mots de passe peuvent faire une requête vers le serveur afin de savoir quels autres clients du même coffre ont des messages en liste d'attente qui doivent être récupérés. Cela permet à l'utilisateur de déterminer à partir de n'importe quel appareil le nombre d'appareils qui ne sont pas à jours et le temps qui lui reste pour les synchroniser.

### Informations d'utilisation

Les clients d'un coffre peuvent conserver des informations relatives à l'utilisation des secrets contenus dans le coffre, comme la date de création, la date de la dernière modification ou la date de la dernière utilisation. Ces informations sont stockées individuellement sur chaque client à titre indicatif pour l'utilisateur, elles ne sont pas transmises aux autres clients.

### Multi-coffre

Chaque client peut être membre de différents coffres de mots de passe, chaque coffre n'étant pas nécessairement partagé entre les mêmes appareils. Un utilisateur qui aurait différents cas d'usage peut ainsi créer différents coffres pour chacun d'entres eux, pour séparer les secrets personnels et professionnels par exemple.

> Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Appareil maître et gestion des appareils

En plus de son utilité lors de la découverte des clients ou pour l'ajout d'un nouvel appareil, un appareil maître peut permettre à l'utilisateur à gérer ses appareils. L'utilisateur choisi parmi ses appareils lequel est en mesure d'accorder le partage à de nouveaux appareils, par exemple un téléphone mobile. L'appareil maître est alors en mesure de supprimer des appareils du coffre, lorsque l'utilisateur possède un nouvel appareil par exemple.

Un appareil de secours au sein du coffre peut également être choisi par l'utilisateur, seul appareil capable de devenir maître à la place du maître, en cas de perte ou de vol par exemple.

> Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Clients copie et proxy

Un appareil maître peut également accorder à des clients extérieurs au coffre la possibilité de devenir des clients copie. Un client copie ne fait pas partie de la liste des clients d'un coffre, il n'en reçoit pas directement les mises à jour, mais le client maître peut les lui transmettre lorsque lui-même les reçoit. Le client maître devient un proxy pour le client copie : le client copie se situe derrière le proxy et ne peut accéder aux mises à jour que lorsque celui-ci accepte de les lui transmettre.

Comme les clients copie ne font pas partie du coffre, ils ne peuvent pas être mis à jour en utilisant la couche service du gestionnaire de mots de passe, ils sont obligatoirement synchronisés manuellement depuis l'appareil maître, via le Bluetooth, le QR Code ou le protocole ICE.

> Cette fonctionnalité est obligatoire dans le cas où le gestionnaire de mots de passe est limité à trois clients par coffre.

### Partage de mot de passe

Le partage d'un mot de passe avec d'autres personnes ne relève pas des bonnes pratiques vis-à-vis de la sécurité, mais c'est pourtant une pratique courante lorsque différentes personnes partagent un compte identique. Afin d'éviter le recours à un mot de passe plus faible, les clients du gestionnaire de mots de passe auront la possibilité de partager un ou plusieurs secrets du coffre avec des appareils n'appartenant pas au coffre. Le partage de mot de passe n'équivaut pas à un nouveau coffre : chaque utilisateur possède le mot de passe partagé dans son coffre, le mot de passe étant relié à d'autres clients dans les coffres d'autres utilisateurs. Dès que le mot de passe fait l'objet d'une mise à jour dans l'un des coffres, le client qui possède la liaison vers un client lié va lui transmettre la mise à jour, à réception le client lié va propager la mise à jour au sein de son propre coffre.

Le partage de mots de passe ressemble à l'utilisation d'un client copie, sauf qu'ici tous les clients liés sont des proxys : la mise à jour du mot de passe peut provenir de n'importe quel coffre auquel le mot de passe est lié, lorsque l'appareil lié du coffre en question reçoit la mise à jour, il la propage aux autres appareils liés qui font office de proxy pour leurs coffres respectifs.

La mise à jour du mot de passe partagé entre les appareils liés passe par la couche service. Si possible, la mise à jour est chiffrée de bout en bout avec une clé partagée entre tous les appareils liés, sinon elle est chiffrée de façon pair-à-pair et stockée plusieurs fois sur le serveur avec une date de péremption plus courte.

### Appareil de secours extérieur

Les clients d'un coffre peuvent ajouter un client extérieur comme client de secours. Le client de secours est ajouté au coffre comme un secret, afin d'être partagé et mis à jour entre les clients comme le reste du coffre. L'appareil de secours extérieur peut permettre de récupérer le mot de passe maître du coffre d'un utilisateur en cas d'oubli ou malheureusement de décès.

Pour enregistrer un appareil de secours extérieur, le client maître doit ajouter le hachage de l'identifiant d'enregistrement de l'appareil de secours et calculer une partie du mot de passe maître. La récupération du mot de passe maître se fait grâce au calcul multipartite sécurisé ou à un cryptosystème à seuil, c'est-à-dire grâce au calcul d'une valeur secrète qui nécessite la collaboration de toutes les personnes qui possèdent une partie du secret. Ici, l'appareil maître calcule les deux parties nécessaires à la reconstitution du mot de passe, il partage une partie aux autres appareils en l'ajoutant au coffre, puis il transmet l'autre partie à l'appareil de secours extérieur qui devra obligatoirement la conserver au sein d'un coffre. Pour les appareils appartenant au coffre auquel est rattaché l'appareil de secours, la partie du secret doit être stockée sans chiffrement, afin d'être accessible même lorsque le client est verrouillé.

---

Lorsque l'utilisateur d'un appareil de secours extérieur souhaite récupérer le mot de passe maître du coffre dont il est le secours, il doit manipuler l'un des appareils appartenant au coffre et initier une procédure de récupération de mot de passe maître. Le client va alors envoyer un message en utilisant la couche service du gestionnaire de mots de passe pour transmettre sa partie au client de secours extérieur. Le serveur ne doit rendre disponible le message pour le client de secours qu'après un délai, pour éviter que l'utilisateur de confiance n'exécute la procédure sans l'accord du propriétaire du coffre. Une fois que le message est disponible, le client de secours extérieur peut le récupérer et calculer le mot de passe maître.

Suite à cette procédure de récupération, le mot de passe maître doit être changé car il n'est plus connu du seul utilisateur.

Un appareil de secours n'est lié qu'à un seul coffre de mots de passe. Si l'utilisateur possède plusieurs coffres qui ont chacun un mot de passe maître différent, alors il faut réitérer l'opération autant de fois qu'il y a de coffres. L'appareil de secours extérieur peut être identique pour chaque coffre.

Si l'appareil de secours extérieur venait à ne plus être utilisable, pour cause de perte, de vol ou de renouvellement, il faut également recommencer la procédure. L'appareil maître doit alors mettre à jour la partie de secret et le hachage de l'identifiant d'enregistrement de l'appareil de secours dans coffre pour les autres appareils.

> Cette fonctionnalité doit faire l'objet d'une étude plus aboutie sur sa faisabilité.

### Client de sauvegarde

L'utilisateur peut également utiliser un périphérique de stockage externe pour sauvegarder ses coffres. Le périphérique de stockage externe, clé USB ou carte SD par exemple, est alors utilisée pour conserver les données chiffrées du coffre. Le périphérique est formatté avant d'être écrit, l'ensemble de l'espace de stockage est chiffré de la même manière que s'il s'agissait de l'espace mémoire d'un appareil ayant accès au coffre. Le périphérique de stockage est alors un client de sauvegarde, il interagit avec le coffre de la même manière qu'un client copie, il est mis à jour depuis n'importe quel client du coffre qui devient alors un client proxy.

L'utilisation du périphérique de stockage pour un client de sauvegarde n'est pas la même que dans le cas d'une communication entre des clients, mais le même périphérique peut être utilisé, à condition d'espace mémoire suffisant. À ce moment-là, le périphérique de stockage contient à la fois le coffre chiffré en temps que client de sauvegarde, et la mise à jour du coffre chiffrée de bout en bout à transmettre à un autre client, notamment suite à la péremption de messages en attente sur le serveur. Les informations d'un coffre vont systématiquement du client enregistré vers le client de sauvegarde, le client de sauvegarde ne peut jamais mettre à jour un autre client.

---

Du point de vue du client de sauvegarde, le client enregistré n'est qu'un lecteur pour le périphérique de stockage. Chaque client aura deux modes de fonctionnement : le stockage local qui est le fonctionnement par défaut et le stockage externe pour les clients de sauvegarde. Comme le contenu stocké est chiffré, d'un point de vue extérieur, le périphérique est inexploitable sans un client du gestionnaire de mots de passe.

Le client de sauvegarde permet même à l'utilisateur de récupérer ses mots de passe depuis un client qui n'appartient pas au coffre. En connectant le périphérique à un appareil et en renseignant le mot de passe maître dans le client, l'utilisateur peut accéder à ses mots de passe tels qu'ils ont été sauvegardés.

> Il est possible de concevoir et de vendre des appareils électroniques sécurisés compatibles et prêts à l'emploi pour ce cas d'usage, d'une manière similaire à la YubiKey.

Cette fonctionnalité permet la récupération du coffre en cas de perte de tous les autres appareils.

## Sécurité

### Saisie de mot de passe maître

Dans la plupart des applications ou sites web avec lesquels sont amenés à interagir les utilisateurs, l'entrée des mots de passe se fait en masquant chaque caractère lors de la saisie, en utilisant des astérisques `*` ou des gros points `•`. Il est même possible que le caractère tapé reste affiché une fraction de seconde avant d'être remplacé afin de vérifier les fautes de frappes.

Ce mode de saisie permet à un observateur malveillant de connaître la taille du mot de passe, ce qui facilite grandement les attaques par force brute. Pour remédier à cela, il est possible de ne pas ajouter un caractère masqué lors de la saisie d'un nouveau caractère, mais de signaler l'entrée d'un nouveau caractère via un clignotement du dernier caractère masqué ou en faisant apparaître le dernier caractère tapé une fraction de seconde.

Les outils en ligne de commande sur UNIX/Linux ont une solution plus efficace encore : ils n'écrivent aucun caractère lors de la saisie d'un mot de passe, le champ reste vide jusqu'à ce que l'utilisateur appuie sur Entrée.

Le gestionnaire de mots de passe permettra de choisir entre les modes de saisie du mot de passe maître selon les préférences de l'utilisateur, pour offrir une meilleure sécurité.

### Changement du mot de passe maître

L'utilisateur pourra modifier son mot de passe maître depuis l'un de ses appareils. Le mot de passe maître est utilisé pour déchiffrer la mémoire du client, ce changement mot de passe a pour conséquence de changer la clé de ce chiffrement. Le mot de passe maître sera ajouté au coffre comme un secret particulier. Une fois que le mot de passe est modifié sur l'un des clients, la modification peut être récupérée par les autres clients et appliquée à réception. À ce moment, l'utilisateur peut être informé que son mot de passe maître est modifié par la dernière mise-à-jour du coffre, afin de valider ce changement.

### Double authentification

Pour authentifier l'utilisateur sur un client, le gestionnaire de mots de passe pourra demander une double authentification, c'est-à-dire le renseignement du mot de passe maître et d'une autre méthode d'authentification. Pour les appareils qui possèdent de la biométrie, les clients devront supporter ces technologies (reconnaissance faciale, empreinte digitale, etc.).

Les clients, notamment en ligne de commande, pourront accepter l'authentification par clé publique. Le gestionnaire de mots de passe sera également compatible avec les dispositifs d'authentification électronique du style YubiKey, qui se compose d'une clé physique qui stocke des clés cryptographiques à l'intérieur.

Le gestionnaire de mots de passe supportera également la double authentification à base de mot de passe à usage unique (*One Time Password* ou *OTP*). Cette méthode permet de générer des codes à usage unique qui sont produits depuis un appareil, électronique ou via une application comme FreeOTP ou les Authenticator de Microsoft ou Google par exemple. Le client peut alors s'assurer de l'authentification car l'utilisateur a accès au secret commun qui lui permet de calculer le bon code.

### Générateur de mot de passe

Le gestionnaire de mots de passe proposera la possibilité de générer des mots de passe forts pour l'utilisateur. Comme l'utilisateur n'a plus besoin de retenir ses mots de passe, ceux-ci peuvent extrêmement robustes en étant longs et ayant de nombreux caractères spéciaux. Cette fonctionnalité reposera sur la génération de nombre aléatoire (*Random Number Generator* ou *RNG*) des appareils, lesquels font l'objet d'une vérification minutieuse.

### Outils de détection de mot de passe faible

Le gestionnaire de mots de passe proposera également la possibilité d'analyser les mots de passes, afin de rechercher des motifs comme des noms ou des formats de date, vérifier la longueur et l'utilisation de majuscules, minuscules, chiffres et caractères spéciaux, vérifier la réutilisation de mots de passe ou de parties de mots de passe.

La recherche de vulnérabilité dans les mots de passe peut également être réalisée en profondeur, grâce à des outils de vérification de robustesse comme John the Ripper ou la recherche des identifiants et mots de passe dans des fuites de données comme ce que permet Have i been pwned.

Ces analyses ne peuvent être réalisées que sur les mots de passe déchiffrés. Comme la manipulation de données déchiffrées est sensible, le gestionnaire de mots de passe ne fera jamais ces analyses automatiquement, sauf demandé explicitement par l'utilisateur.

### Numéro de sécurité

Afin de vérifier que son coffre de mots de passe est sécurisé, l'utilisateur sera amené à vérifier son numéro de sécurité. Pour cela, l'utilisateur doit accéder à plusieurs appareils simultanément et vérifier que ce numéro, c'est-à-dire une suite de quelques dizaines de chiffres, est identique sur chacun. Le numéro de sécurité peut également être vérifié via le QR Code, mais pas via le Bluetooth ou autre afin de limiter les risques d'attaque. Le numéro de sécurité est calculé grâce à une fonction de hachage sur le hachage des identifiants d'enregistrement de tous les client du coffre. Si le numéro est identique sur chaque appareil, c'est que tous les clients se connaissent effectivement correctement et que le coffre est sécurisé. L'utilisateur peut ensuite marquer ses appareils comme étant vérifiés.

Un second numéro de sécurité peut être calculé pour vérifier que les clients sont synchronisés. L'utilisateur peut ainsi vérifier si tous ses appareils sont à jour ou si certains doivent récupérer des messages en attente. Cette vérification doit également être faite manuellement par l'utilisateur en comparant les valeurs deux à deux. Ce second numéro de sécurité est calculé grâce à une fonction de hachage sur le coffre chiffré. Si ce numéros de sécurité diffèrent, cela signifie qu'un client au moins n'est pas à jour, il n'y a pas de compromission des données tant que le premier numéro de sécurité est identique.

### Chiffrement du stockage multi-niveau

Dans la mémoire de l'appareil, les coffres d'un client sont stockés sous forme chiffrée. Ce chiffrement est effectué à deux niveaux, d'abord chaque fichier de secret est chiffré individuellement, puis l'ensemble des données du coffre sont chiffrées une seconde fois, c'est-à-dire les fichiers de secret chiffrés mais aussi les identifiants d'enregistrement des autres clients et la clé partagée. Ceci permet d'éviter la manipulation de ressources déchiffrées au niveau de l'appareil de l'utilisateur. Dans le système d'exploitation, une donnée est déchiffrée uniquement lorsque l'utilisateur souhaite y accéder, en lecteur ou en écriture. Le reste du temps, même lorsque le client est ouvert, les données sensibles restent chiffrées.

Selon les recommandations de sécurité, il est possible que la clé de chiffrement individuel des fichiers de secret ne soit pas dérivée du mot de passe maître mais stockée de manière chiffrée dans le coffre et protégée par le droit d'accès aux ressources du système d'exploitation. Si une telle clé est mise en place, elle sera créée par chaque client indépendamment et ne sera jamais partagée entre eux. Ce n'est pas un problème puisque cette clé ne sert qu'à chiffrer les données stockées localement, le résultat de ce chiffrement ne transite jamais entre les clients. Cette clé supplémentaire sert à empêcher la possibilité d'utiliser une copie de la mémoire du coffre, même si l'attaquant connait le mot de passe maître.

## Communication entre appareils

### Découverte Bluetooth, caméra et ports

Sur chaque appareil, le client va scanner le support du protocole Bluetooth, la présence d'une caméra, l'accès à l'interface graphique pour le client en ligne de commande et la présence de ports USB, USB-C, SD ou micro SD, afin de proposer à l'utilisateur les solutions de communication pair-à-pair adaptée entre chaque appareil.

Une fois que la découverte des clients est réalisée et que les clients peuvent communiquer de manière chiffrée de bout en bout, ils peuvent s'échanger ces informations dans des trames prévues à cet effet. Ils peuvent en profiter pour s'échanger le nom des appareils pour l'utilisateur.

### Découverte réseau

Lorsqu'ils essaient de communiquer de façon pair-à-pair, les clients peuvent initier une découverte réseau afin de découvrir les potentiels clients qui seraient sur le même réseau local. Si c'est le cas, les clients peuvent échanger en direct sans passer par d'autres moyens de communication.

Cette découverte ne peut pas être faite une seule fois après la découverte des autres clients comme pour les autres moyens de communication pair-à-pair car les appareils peuvent être déplacés et changer de réseau.

Les trames correspondant à la découverte réseau ne sont pas encore spécifiées. Elles devront apporter un soin particulier à la sécurité, les deux attaques principales seraient la possibilité d'interagir avec un client qui n'est pas dans le coffre en usurpant une identité et la possibilité de récupérer un identifiant d'enregistrement pour attaquer plus tard le serveur. Cette découverte nécessite aussi que les utilisateurs ouvrent l'accès au réseau pour les clients.

### Support du protocole ICE

Les clients pourront utiliser en dernier recours le protocole ICE pour communiquer de façon pair-à-pair. Pour cela, les clients et le serveur doivent supporter le protocole. Comme décrit précédemment, l'implémentation du protocole sera faite selon les RFC décrivant les protocoles sous-jacents.
