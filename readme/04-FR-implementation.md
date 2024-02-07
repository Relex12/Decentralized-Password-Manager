# Sécurité

Détails sur la sécurité et l'implémentation algorithmique de la cryptographie

[Page précédente : Spécifications](https://relex12.github.io/fr/Decentralized-Password-Manager/Specifications)

[Page suivante : Fonctionnalités](https://relex12.github.io/fr/Decentralized-Password-Manager/Fonctionnalites)

## Sommaire

[toc]

## Prérequis

* une fonction de hachage (SHA256, SHA384, SHA512) : $hash$
* une courbe elliptique (Curve25519)
* un échange de Diffie-Hellman sur courbe elliptique (X25519) : $ecdh(PK,SK)$
* un système de signature sur courbe elliptique (Ed25519) : $ecdsa(SK)$
* un couplage
* un échange de Diffe-Hellman à base de couplage : $pbdh(PKa,PKb,SK)$
* un système de chiffrement symétrique (AES_256_GCM) : $cipher(SK)$
* une fonction de dérivation de clé (PBKDF2) : $kdf(SK)$
* un format de certificat (X.509)

## Séquence d'enregistrement

> TODO: faire des diagrammes, quitte à les inclure dans le readme 03 et supprimer celui-ci s'il est de trop
> peut-être pas en fait

* clés publique et privée de l'autorité de certificats : $PKca$ et $SKca$
* certificat de l'autorité de certificats : $Cert\_ca = PKca+validity$​
* clés publique et privée du serveur : $PKs$ et $SKs$
* certificat du serveur : $Cert\_s = PKca+validity+CA\ name + ecdsa(SKca,PKca+validity+CA\ name)$
* clés publique et privée d'un client : $PKc$ et $SKc$
* clé d'enregistrement : $RK=user\_id+PKc+vault\_id$
* hachage de clé d'enregistrement : $HRK=hash(RK)$
* clé éphémère symétrique : $EKsc=ecdh(PKs,SKc)=ecdh(PKc,SKs)$

```mermaid
sequenceDiagram
	participant TE as Téléphone / Email
    participant C as Client
    participant S as Serveur
    participant CA
    C ->> S: Demande certificat
    S ->> C: Cert_s
    Note over C: Récupère PKs et CA name
    C ->> CA: Demande certificat
    CA ->> C: Cert_ca
    Note over C: Vérifie la signature de Cert_s
    Note over C: Enregistre PKs
    C ->> S: PKc+cipher(EKsc,RK+HRK)
    S ->> TE: Challenge code
    C ->> S: Réponse challenge
    Note over S: Enregistre HRK dans le filtre de Bloom
    S ->> C: Info OK
```

## Découverte des clients et chiffrement avec le serveur

* clés publique et privée du serveur : $PKs$ et $SKs$
* clé publique et privée d'un client A, clé d'enregistrement et hachage : $PKa$ et $SKa$, $RKa=user\_id+PKa+vault\_id$, $HRKa=hash(RKa)$
* clé publique et privée d'un client B, clé d'enregistrement et hachage : $PKb$ et $SKb$, $RKb=user\_id+PKb+vault\_id$, $HRKb=hash(RKb)$
* clé publique et privée d'un client C, clé d'enregistrement et hachage : $PKc$ et $SKc$, $RKc=user\_id+PKc+vault\_id$, $HRKc=hash(RKc)$
* clés éphémères publique et privée de A, B et C : $EPKa$ et $ESKa$, $EPKb$ et $ESKb$, $EPKc$ et $ESKc$
* clés éphémères symétriques entre A, B, C respectivement et S : $EKsa=ecdh(PKs,ESKa)$, $EKsb=ecdh(PKs,ESKb)$, $EKsb=ecdh(PKs,ESKb)$
* clé partagée symétrique initiale : $IKabc=pbdh(PKa,PKb,SKc)$
* message chiffré de bout en bout : $message=cipher(IKabc, RKa+RKb+RKc)$
* numéro de sécurité : $SNabc=hash(HRKa+HRKb+HRKc)$

```mermaid
sequenceDiagram
    participant A as Client A
    participant S as Serveur
    participant B as Client B
    participant C as Client C
    par
    B ->> A: RKb
    and
    C ->> A: RKc
    end
    A ->> S: EPKa+cipher(EKsa, HRKa+HRKb+HRKc+message)
    Note over S: Vérifie HRKa HRKb et HRKc enregistrés
    Note over S: Ajoute message en file d'attente pour HRKb et HRKc
    par
    B ->> S: EPKb + cipher(EKsb, HRKb)
    S ->> B: cipher(EKsb, HRKa+message)
    Note over B: Déchiffre message avec IKabc
    and
    C ->> S: EPKc + cipher(EKsc, HRKc)
    S ->> C: cipher(EKsc, HRKa+message)
    Note over C: Déchiffre message avec IKabc
    end
    Note over A: Vérifie SNabc
    Note over S: Supprime message
    Note over B: Vérifie SNabc
    Note over C: Vérifie SNabc
```

## Envoi de messages entre clients

* clé publique et privée d'un client A, clé d'enregistrement et hachage : $PKa$ et $SKa$, $RKa=user\_id+PKa+vault\_id$, $HRKa=hash(RKa)$
* clé publique et privée d'un client B, clé d'enregistrement et hachage : $PKb$ et $SKb$, $RKb=user\_id+PKb+vault\_id$, $HRKb=hash(RKb)$
* clé publique et privée d'un client C, clé d'enregistrement et hachage : $PKc$ et $SKc$, $RKc=user\_id+PKc+vault\_id$, $HRKc=hash(RKc)$
* clés éphémères publique et privée de A, B et C : $E1PKa$ et $E1SKa$, $E1PKb$ et $E1SKb$, $E1PKc$ et $E1SKc$
* clé partagée symétrique initiale : $IKabc=pbdh(PKa,PKb,SKc)$​
* clés partagées éphémères potentielles :
    * $S1Kabc=pbdh(E1PKa,PKb,SKc)$
    * $S2Kabc=pbdh(E1PKa,PKb,E1SKc)$
    * $S3Kabc=pbdh(E2PKa,PKb,E1SKc)$

```mermaid
sequenceDiagram
    participant A as Client A
    participant B as Client B
    participant C as Client C
    par
    A ->> B: cipher(IKabc, E1PKa+message)
    and
    A ->> C: cipher(IKabc, E1PKa+message)
    end
    Note over A: Calcule S1Kabc avec E1SKa
    Note over B: Calcule S1Kabc avec E1PKa
    Note over C: Calcule S1Kabc avec E1PKa

    par
    C ->> A: cipher(S1Kabc, E1PKc+message)
    and
    C ->> B: cipher(S1Kabc, E1PKc+message)
    end
    Note over A: Calcule S2Kabc avec E1PKc
    Note over B: Calcule S2Kabc avec E1PKc
    Note over C: Calcule S2Kabc avec E1SKc

    par
    A ->> B: cipher(S2Kabc, E2PKa+message)
    and
    A ->> C: cipher(S2Kabc, E2PKa+message)
    end
    Note over A: Calcule S3Kabc avec E2SKa
    Note over B: Calcule S3Kabc avec E2PKa
    Note over C: Calcule S3Kabc avec E2PKa
```

## Format des trames entre clients

> TODO: liste exhaustive des types de messages

### Découverte des clients

* type de message (4 bits)
* clé publique (256 bits)
* bloc chiffré
  * nombre de clients dans le coffre (8 bits)
    * taille du nom de l'appareil (8 bits)
    * nom de l'appareil pour l'utilisateur (jusqu'à 255 octets)
    * clé publique du client (256 bits)
    * hachage de la clé d'enregistrement du client(256 bits)

### Mise à jour du coffre

* type de message (4 bits)
* bloc chiffré
  * hachage de la clé d'enregistrement de l'émetteur du message (256 bits)
  * nouvelle clé publique de l'émetteur (256 bits)
  * taille de la mise à jour (16 bits)
  * mise à jour (jusqu'à 65535 octets)

> La taille d'une mise à jour en situation réelle doit encore être déterminée.

## Format des requêtes entre client et serveur

> TODO: liste exhaustive des types de requêtes
>
> TODO: ajouter le protocole ICE

### Demande de certificat

* type de requête (8 bits)

### Demande d'enregistrement

* type de requête (8 bits)
* clé publique (256 bits)
* bloc chiffré
  * taille de l'identifiant utilisateur (8 bits)
  * identifiant utilisateur (jusqu'à 255 octets)
  * hachage de la clé d'enregistrement (256 bits)

> TODO: est-ce que le serveur doit connaître l'identifiant du coffre ?

### Envoi de la réponse au challenge

* type de requête (8 bits)
* clé publique éphémère (256 bits)
* bloc chiffré
  * hachage de la clé d'enregistrement (256 bits)
  * réponse (8 octets)

### Demande de récupération de message

* type de requête (8 bits)
* clé publique éphémère (256 bits)
* bloc chiffré
  * hachage de la clé d'enregistrement (256 bits)

