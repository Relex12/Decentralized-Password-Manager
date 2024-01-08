# Solutions techniques

* [Solutions techniques](#solutions-techniques)
    * [Communication sans serveur](#communication-sans-serveur)
        * [ Comparaison](#-comparaison)
        * [ Solutions non retenues](#-solutions-non-retenues)
    * [Serveur sans état (*stateless*)](#serveur-sans-état-(*stateless*))
    * [Serveur avec état (*stateful*)](#serveur-avec-état-(*stateful*))
    * [Solution retenue](#solution-retenue)

<!-- table of contents created by Adrian Bonnet, see https://Relex12.github.io/Markdown-Table-of-Contents for more -->

## Communication sans serveur

Comme le chiffrement des secrets s'effectue toujours sur les appareils côté client, la problématique d'un gestionnaire de mots de passe décentralisé peut-être reformulé comme ceci : comment transfère-t-on des données chiffrées entre deux appareils ou plus sans passer par un serveur ? Pour cela, on s'intéresse ici à la communication entre les appareils directement. Cela implique pour toutes les solutions ci-dessous une certaine proximité au moment de l'échange de données, ce qui peut être un gros point négatif.

- **Bluetooth** : protocole de communication de proximité sans-fil
    - nécessite des appareils supportant le Bluetooth
- **QRCode** : format de code-barres bidimensionnel
    - nécessite une caméra sur l'appareil recevant les données
    - limité à 3 ko
- **Stockage externe** : format USB ou (micro)SD
    - nécessite des ports et/ou des adaptateurs
- **Réseau local**

###  Comparaison

|     Solution     |                    Facilité d'utilisation                    |
| :--------------: | :----------------------------------------------------------: |
|    Bluetooth     |                     depuis les appareils                     |
|   Réseau local   |                     depuis les appareils                     |
|      QRCode      | besoin de montrer l'écran d'un appareil à la caméra de l'autre |
| Stockage externe | besoin de connecter le support de stockage aux deux appareils l'un après l'autre |

###  Solutions non retenues

D'autres solutions peuvent être envisagées qui ne présentent pas le désavantage de la proximité, mais parmi d'autres défauts, beaucoup nécessitent de résoudre des problèmes de translation d'adresses IP via le NAT dont la mise en œuvre peut être difficilement abordable pour le grand public.

*   Les solutions basées sur la **Blockchain** sont rejetées car elles nécessitent une conservation des données chiffrées théoriquement pour toujours. De plus, les applications basées sur la Blockchain sont souvent des prétextes à l'essor d'une cryptomonnaie et ce n'est pas le but recherché.
*   L'utilisation du protocole **SFTP** nécessite soit la mise en place d'un serveur qui stocke les données chiffrées, ce que l'on cherche à éviter, soit la configuration réseau du NAT pour utiliser les appareils comme serveurs deux à deux.
*   De la même manière, le **Peer-to-Peer** (*P2P*) comme le torrent nécessite de configurer le NAT. De plus, le protocole est utile pour partager des données statiques, mais assez peu efficace pour les modifier. Un serveur est également nécessaire pour partager les métadonnées afin d'assurer l'intégrité.
*   Les adresses **IPv6** permettent de surmonter le problème de translation d'adresse IP et de configuration NAT, en pratique ces adresses sont souvent bloquées par les firewalls pour des raisons de sécurité.
*   Les solutions dites **sans état** (*stateless*) qui consistent à calculer un mot de passe en combinant un mot de passe maître, un nom de domaine et un nom d'utilisateur via une fonction de hachage cryptographique sont en réalité moins pratiques car les mots de passe ne peuvent plus être modifiés en cas de fuite de données notamment, les sites peuvent parfois avoir plusieurs noms de domaine dont il faut se souvenir et le partage de secrets autres que des mots de passe n'est pas possible.

## Serveur sans état (*stateless*)

Pour s'acquitter de la contrainte de proximité et éviter la configuration du NAT, il est possible d'utiliser un serveur qui ne s'occuperait que de transmettre les données entre les appareils. Aucune donnée persistante ne reste sur ce serveur, il ne sert que comme point de rendez-vous pour les clients. C'est ce que permet le protocole **ICE** (*Interactive Connectivity Establishment*) décrit dans la [RFC 8445](https://datatracker.ietf.org/doc/html/rfc8445), qui propose de lister et d'échanger les adresses que peuvent utiliser les deux appareils qui souhaitent communiquer.

Chaque appareil est un agent, chaque adresse (au niveau transport) est un candidat. Les agents possèdent chacun au minimum trois candidats : un ou plusieurs candidats hôte obtenus en associant un port à une adresse physique ou virtuelle, un candidat réfléchi vu par un serveur et issu de l'allocation d'un port et l'adresse publique du NAT et un candidat relayé c'est-à-dire l'adresse donnée par un serveur relai.

Le protocole ICE utilise un serveur STUN (*Session Traversal Utilities for NAT* [RFC 8489](https://datatracker.ietf.org/doc/html/rfc8489)) pour récupérer les candidats, un serveur de signalement de type SIP (*Session Initiation Protocol* [RFC 3261](https://datatracker.ietf.org/doc/html/rfc3261)) pour échanger les candidats entre agents et un serveur TURN (*Traversal Using Relays around NAT* [RFC 8656](https://datatracker.ietf.org/doc/html/rfc8656)) pour transmettre les données entre les agents lorsqu'ils sont derrière un firewall.

Lors de la découverte, le candidat réfléchi est vu par le serveur STUN contacté à travers le NAT et le candidat relayé est l'adresse du serveur TURN.

Le protocole ICE est essentiellement utilisé pour des applications d'appels vidéos ou de messagerie instantanée tel quel WebRTC, car il est utilisé pour acheminer des flux de données entre plusieurs utilisateurs. Cette solution semble surdimensionnée pour notre cas d'usage qui consiste à transmettre un paquet de temps en temps contenant la mise à jour des secrets chiffrée.

---

Pour alléger l'infrastructure liée au partage des données et puisque dans notre cas de figure le protocole ICE utiliserait majoritairement des candidats relayés, il est possible d'utiliser un serveur **SIP** également requis par ICE afin de transmettre les données. C'est l'extension pour messagerie instantanée de SIP ([RFC 3428](https://datatracker.ietf.org/doc/html/rfc3428)) qui permet d'utiliser le protocole pour envoyer des messages entre deux agents.

Avec ce protocole, les agents doivent s'enregistrer auprès d'un serveur SIP avant de pouvoir commencer l'échange de données, en fournissant les adresses sur lesquelles les contacter. Chaque adresse est sous la forme d'une URI, qui peut-être converti en un autre format pour ne comprendre que l'adresse de l'appareil et son port. Une session est initiée par l'un des agents, l'autre étant contacté par le serveur SIP via son adresse, puis les agents peuvent s'échanger des messages, qui dans notre cas contiendraient la mise à jour chiffrée des secrets. Les agents peuvent ensuite terminer la session et se désenregistrer du serveur SIP afin de limiter la charge de celui-ci.

Lors de ces échange, le chiffrement de bout en bout est possible en se basant sur les recommandations de la [RFC 8591](https://datatracker.ietf.org/doc/html/rfc8591). Bien que ce ne soit pas le plus adapté à notre cas d'usage, il est également possible d'utiliser MSRP (*Message Session Relay Protocol* [RFC 4975](https://datatracker.ietf.org/doc/html/rfc4975/)) afin d'établir des sessions persistantes entre les agents.

---

L'utilisation d'un serveur sans état, que ce soit avec le protocole ICE ou avec SIP, permet de livrer les flux à plusieurs agents simultanément, autrement dit de synchroniser les secrets depuis un appareil vers tous les autres appareils de l'utilisateur en une seule fois.

Pour autant, il est obligatoire de mettre à jour les appareils de façon synchrone : pour propager la mise à jour de ses secrets entre tous ses appareils, l'utilisateur est obligé d'intervenir manuellement sur chacun des appareils pour procéder à la mise à jour. Ceci oblige l'utilisateur à interagir avec chacun de ses appareils deux à deux en simultané, donc de nouveau de le désavantage de la proximité, mais il doit également propager la mise à jour vers chacun de ses appareils à chaque modification des secrets, au risque de se retrouver avec des différentes versions et d'aboutir à une divergence et des conflits sur les différents appareils.

## Serveur avec état (*stateful*)

La problématique de la synchronicité des mises à jour ne peut être résolue qu'avec un serveur qui stocke le message pendant un laps de temps : si l'utilisateur souhaite pouvoir envoyer une mise à jour de ses secrets depuis un appareil puis récupérer cette mise à jour dans un second temps sur un autre appareil sans avoir besoin de manipuler à nouveau le premier, il est nécessaire que cette mise à jour soit mémorisée le temps d'être récupérée.

Cette formulation est analogue au fonctionnement d'une messagerie instantannée chiffrée de bout en bout, telle que [Signal](https://www.signal.org/). Lorsqu'un service de messagerie instantannée met en place un chiffrement de bout en bout, il est impossible pour quiconque autre que l'émissaire et le destinataire de déchiffrer les messages, y compris les fournisseurs de ce service. En dehors de toute considération de revente de données ou de méta-données (qui peuvent toujours être stockées malgré le chiffrement), le fournisseur peut supprimer un message de ses serveurs une fois que celui-ci a été transmis à son ou ses destinataires.

Avec un gestionnaire de mots de passe basé sur ce principe, un appareil enverrait la mise à jour chiffrée de ses secrets comme s'il s'agissait d'un message. Le serveur recevrait ce message et le livrerait à chacun des autres appareils dès que ceux-ci se connectent, avant de supprimer le message de son espace de stockage. Ainsi, il n'est plus nécessaire d'avoir recourt à un tiers de confiance pour la sauvegarde des données chiffrées, puisque celles-ci ne sont stockées que sur les appareils de l'utilisateur.

Cependant le postulat de départ était de réaliser un gestionnaire de mots de passe qui soit décentralisé, c'est-à-dire sans serveur. Si ce changement de sujet est justifié par le critère de large acceptation par le grand public et par le besoin d'échange asynchrone, il faut encore motiver ce parti pris. Comment peut-on concevoir un gestionnaire de mots de passe centralisé dans le but de réduire au maximum les coûts de gestion de l'infrastructure et mutualiser la charge de travail ?

---

La première piste pour réduire la charge du serveur est la **suppression automatique** des messages non livrés après une **date de péremption** : une fois que le message de mise à jour des secrets est reçu par le serveur, les autres appareils de l'utilisateur ont un temps limité pour le récupérer. Passé ce délai, l'utilisateur est obligé de synchroniser manuellement les appareils non mis à jour via l'une des solutions présentées au dessus. L'intérêt de cette date de péremption est de servir les mises à jour aux appareils réguliers de l'utilisateur, mais de ne pas s'encombrer avec des messages destinés à des appareils qui ne se connectent que rarement.

Le délai avant suppression n'a pas de valeur théorique précise. Il doit être calculé d'après une étude statistique pour permettre au plus d'appareils possible de récupérer leurs mots de passe à temps, sans être pénalisé par d'éventuels appareils qui se connecteraient peut régulièrement au serveur. La fréquence de connection des appareils au serveur dépend des cas d'usage imaginés : plusieurs fois par jour pour un téléphone mobile, plusieurs fois par jour mais pas tous les jours de la semaine pour un ordinateur de bureau et quelques fois par mois voire par an pour un périphérique de sauvegarde de secours. La valeur devra donc être adaptée selon le profil de la distribution des fréquences de connection, pour minimiser le ratio entre le délai avant suppression et le nombre d'appareils capable de récupérer un message à temps. Il est question d'une date de péremption de l'ordre de dix jours.

Pour savoir si une synchronisation manuelle est requise, les appareils peuvent enregistrer la durée depuis leur dernier contact avec le serveur, si celle-ci est supérieure au délai de suppression des messages, alors l'appareil signale à l'utilisateur que sa version n'est peut-être pas la dernière. Pour vérifier cela, l'appareil peut utiliser le hachage cryptographique pour mettre à disposition de l'utilisateur une valeur spécifique à l'état des secrets stockés sur l'appareil. Cette valeur est comparée par l'utilisateur entre les appareils, s'il y a une différence, alors l'un des appareils n'est plus à jour et il faut procéder à une synchronisation manuelle depuis un appareil à jour grâce à une solution sans serveur ou avec serveur sans état.

---

Afin d'éviter les demandes abusives de stockage de message sur le serveur, les utilisateurs sont authentifiés après du serveur grâce à un **filtre de Bloom**. Le filtre de Bloom est une structure de données probabiliste qui permet de réduire grandement l'espace de stockage utilisé, car le filtre utilise autant d'espace indépendamment du nombre d'entrées. Il ne peut être utilisé que pour renvoyer un test d'appartenance : le filtre de Bloom sert à décrire si une entrée est dans la structure ou non, mais il ne permet pas de retrouver des informations associées à une clé.

L'approche probabiliste du filtre de Bloom implique que lorsque le test d'appartenance renvoie que l'élément est présent dans la structure, ce résultat n'est pas certain, il existe une faible probabilité que l'élément ne soit pas présent. En revanche, lorsque le test renvoie que l'élément est absent, ce résultat est absolument certain.

Pour obtenir une telle propriété, le filtre de Bloom ne liste pas les éléments qui sont entrés dans la structure. Le filtre se constitue d'un tableau T de booléen (ou bit) de taille M dont chaque case est initialisée à 0, ainsi que de k fonctions de hachage, notées h<sub>1</sub> à h<sub>k</sub>, telle que chacune de ces fonctions renvoie vers des cases aléatoires dans le tableau, mais différentes entre elles. 

Lors de l'ajout d'un élément e dans le tableau, on calcule les valeurs des fonctions de hachage v<sub>1</sub>=h<sub>1</sub>(e) jusqu'à v<sub>k</sub>=h<sub>k</sub>(e) et pour chacune de ses valeurs, on met un 1 dans la case du tableau correspondante T[v<sub>1</sub>]=1 jusqu'à T[v<sub>k</sub>]=1. Lors du test de présence d'un élément e dans le tableau, on vérifie les cases d'indice v<sub>1</sub>=h<sub>1</sub>(e) jusqu'à v<sub>k</sub>=h<sub>k</sub>(e), si toutes ces cases sont à 1, alors l'élément est surement présent, si au moins une case est à 0, alors il est certain que l'élément n'est pas présent.

Dans notre cas, l'implémentation d'un filtre de Bloom permet de refuser des envois de message de la part d'utilisateurs qui ne seraient pas enregistrés auparavant. Mais cela ne permet pas de contrer des attaques par déni de service (*deny of service* ou *DoS*), notamment en multipliant les envois de messages au serveur et en ne récupérant pas les messages en attente d'être délivré. Les messages non récupérés seront supprimés auprès quelques jours et il est possible de limiter le nombre de message en attente dans la file d'envoi de serveur pour un même utilisateur. Cependant, ces protections ne suffisent pas face à une attaque par déni de service distribué (*DDoS*), où un grand nombre d'utilisateurs se coordonnent pour attaquer en simultané.

## Solution retenue

Malgré un fonctionnement grâce à un serveur qui sert de mémoire tampon en attendant que tous les appareils aient récupéré la mise à jour des secrets, la solution du serveur avec état similaire à Signal est la plus intéressante car elle est la seule à proposer une utilisation à distance et asynchrone, sans configuration réseau de la part de l'utilisateur.

Ce choix semble raisonnable au vu des motivations de la réduction et de la mutualisation des coûts, grâce à la date de péremption et au filtre de Bloom expliqués ci-dessus. De plus, il est envisageable de proposer aux utilisateurs qui le souhaitent d'héberger un serveur afin de participer à la propagation du gestionnaire de mots de passe, tel que cela peut-être mis en place par le protocole [Matrix](https://matrix.org/), un standard de communication temps-réel qui implémente le chiffrement de bout en bout comme Signal et qui possède des ponts vers d'autres protocoles tels que SMTP (email), IRC et XMPP ainsi que de nombreuses applications de messagerie instantanée propriétaires.

Comme Matrix est un standard, il existe de nombreuses implémentations client et serveur de Matrix, les implémentations de référence étant [Element](https://element.io/) côté client et [Synapse](https://github.com/element-hq/synapse) côté serveur. De plus, la possibilité d'héberger son propre serveur et d'utiliser ses propres implémentations permet une plus grande confiance de la part d'acteurs sensibles tels que des organisations ou des gouvernements, contribuant à la propagation du gestionnaire de mots de passe.
