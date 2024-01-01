# Solutions techniques

[toc]

## Communication sans serveur

Comme le chiffrement des mots de passe s'effectue toujours sur les appareils côté client, la problématique d'un gestionnaire de mots de passe décentralisé peut-être reformulé comme ceci : comment transfère-t-on des données chiffrées entre deux appareils ou plus sans passer par un serveur ? Pour cela, on s'intéresse ici à la communication entre les appareils directement. Cela implique pour toutes les solutions ci-dessous une certaine proximité au moment de l'échange de données, ce qui peut être un gros point négatif.

- **Bluetooth** : protocole de communication de proximité sans-fil
    - nécessite des appareils supportant le Bluetooth
- **QRCode** : format de code-barres bidimensionnel
    - nécessite une caméra sur l'appareil recevant les données
    - limité à 3 ko
- **Stockage externe**
    - nécessite des ports USB / SD ou adaptateurs
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
*   De la même manière, le **Peer-to-Peer** (***P2P***) comme le torrent nécessite de configurer le NAT. De plus, le protocole est utile pour partager des données statiques, mais assez peu efficace pour les modifier. Un serveur est également nécessaire pour partager les métadonnées afin d'assurer l'intégrité.
*   Les adresses **IPv6** permettent de surmonter le problème de translation d'adresse IP et de configuration NAT, en pratique ces adresses sont souvent bloquées par les firewalls pour des raisons de sécurité.
*   Les solutions dites **sans état** (*stateless*) qui consistent à calculer un mot de passe en combinant un mot de passe maître, un nom de domaine et un nom d'utilisateur via une fonction de hachage cryptographique sont en réalité moins pratiques car les mots de passe ne peuvent plus être modifiés en cas de fuite de données notamment, les sites peuvent parfois avoir plusieurs noms de domaine dont il faut se souvenir et le partage de secrets autres que des mots de passe n'est pas possible.

## Serveur sans état (*stateless*)

Protocole ICE

## Serveur avec état (*stateful*)

Solution retenue avec Signal Protocol
