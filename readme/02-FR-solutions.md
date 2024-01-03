# Solutions techniques

[toc]

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

Solution retenue avec Signal Protocol
