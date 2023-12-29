# EsiPass - Password Manager

Le projet se découpe en trois module. La partie `client`, `serveur` et une partie `utils` qui sert au deux premiers modules.

## Installation du projet

Le projet a été développé en Rust et nécessite donc d'avoir l'environnement installé sur sa machine. 

Pour cela il suffit de lancer le script `INIT.sh` pour installer tout ce qui est nécessaire.

Après cela, il faut redémarrer le shell pour avoir accès au commandes cargo et à rustc. 

## Lancement du serveur

Pour lancer le serveur il suffit de lancer le script `START_SERVER.sh`. Par défaut, le serveur est en écoute sur le port 40443 en HTTP simple. 

On peut aussi le lancer avec la commande suivante : 

```bash
cargo run -p server -- --address 0.0.0.0 --port 40443 --cert ./server/cert.pem --key ./server/key.pem
```

Avec les différents paramètres : 

- `address` : adresse d'écoute du serveur
- `port` : port d'écoute du serveur 
- `cert` : certificat 
- `key` : clé privée du serveur pour les connexions HTTPS

## Lancement du client 

Pour lancer le client avec en paramètre le mot de passe maître de l'utilisateur : 

```bash
cargo r -p client -- --master-password "my master pwd"
```

## Documentation du code

Le projet est à peu près correctement commenté. Pour compiler et ouvrir la documentation du module souhaité, il faut être dans son dossier et lancer : 

```bash
cargo doc --open
```

## Remarques

### Communication HTTPS 

Les fichiers `cert` et `pem` devait servir pour la communication en HTTPS entre le client et le serveur. Actuellement, le certificat TLS du serveur n'est pas vérifié par le client car ce dernier est auto-signé. Le client est donc configuré pour accepter tous les certificats. À termes, le serveur pourrait ne plus se baser sur le protocole applicatif HTTP(s) et ainsi ne plus utiliser ces fichiers.

### Interface de l'application

A la base le projet devait présenter une interface entièrement en `CLI` mais les premices d'une interface graphique a finalement été développé facilitant grandement le debug de l'application.

### Module `utils`

- Debug : Un ensemble de structures et de méthodes sont définies afin de logger les activités du serveur et permettre au client de les visualiser. Ce module génère le fichier `/server/src/data/debug` que le client récupère avec le endpoint `/debug`.
- Macro : Dans le fichier `tools/our_macro.rs` se trouve des macro utile au développement du projet comme la macro `afaire` qui agit comme la balise `@TODO` dans Java et n'arrête pas le programme à l'instar de la macro `todo` de Rust.
