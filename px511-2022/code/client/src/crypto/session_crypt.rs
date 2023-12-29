//! Ce module est une API de chiffrement du contenue des requêtes EsiPass avec une clé de transfert.
//! C'est ici qu'il faut faire des modifications si l'on veut changer d'algo de chiffrement.
use aes_gcm_siv::Nonce;
use anyhow::Result;

use crate::aes::{aes_cypher_with_nonce, aes_decypher};

/// Fonction de chiffrement du contenue d'une requête vers le serveur grace à une clé de transfert.
///
/// Cette fonction récupère la clé de transfert géré par un Double Ratchet afin de chiffrer une chaine de caractère.
/// On ne récupère pas la valeur initial utilisé pour le chiffrement car elle ne change pas entre les messages. On utilise l'ID de l'utilisateur.
/// On peut se permettre de garder le même Nonce entre les différents messages car la clé de chiffrement change à chaque fois.
/// Retourne un `Result` contenant le `String` représentant la chaine de caractère chiffré ou une erreur.
///
/// * `plaintext` - `String` à chiffrer.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme valeur initiale dans le chiffrement.
pub fn crypt_rqst(plaintext: String, user_id: u64) -> Result<String> {
    // Récupération de la clé de transfert
    utils::afaire!("Fonction crypt_rqst : récupération de la clé de transfert.");
    let transfert_key =
        hex::decode("ea589449840c9bb4f2a863563a9554b105249816b71a235f09765ce03737668e").unwrap();

    // On convertit l'ID en Nonce utilisé par la fonction de chiffrement
    let nbytes = user_id.to_string();
    let nonce = Nonce::from_slice(nbytes.as_bytes());

    // Chiffrement du plaintext
    let cypher = aes_cypher_with_nonce(plaintext, transfert_key, nonce).0;

    // En encode les octets chiffré en chaine de caractère.
    let res = hex::encode(cypher);

    Ok(res)
}

/// Fonction de déchiffrement du contenue d'une réponse du serveur grace à une clé de transfert.
///
/// Cette fonction récupère la clé de transfert géré par un Double Ratchet afin de chiffrer une chaine de caractère.
/// On utilise l'ID de l'utilisateur comme valeur initial utilisé pour le chiffrement car elle ne change pas entre les messages.
/// On peut se permettre de garder le même Nonce entre les différents messages car la clé de chiffrement change à chaque fois.
/// Retourne un `Result` contenant le `String` représentant la chaine de caractère chiffré ou une erreur.
///
/// * `plaintext` - `String` à chiffrer.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme valeur initiale dans le chiffrement.
pub fn decrypt_rqst(to_decrypt: String, user_id: u64) -> Result<String> {
    // Récupération de la clé de transfert
    utils::afaire!("Fonction decrypt_rqst : récupération de la clé de transfert.");
    let transfert_key =
        hex::decode("ea589449840c9bb4f2a863563a9554b105249816b71a235f09765ce03737668e").unwrap();

    // On décode la chaine de caractère en octet
    let ciphered = hex::decode(to_decrypt).unwrap();

    // On convertit l'ID en Nonce utilisé par la fonction de chiffrement
    let nbytes = user_id.to_string();
    let nonce = Nonce::from_slice(nbytes.as_bytes());

    // On déchiffre les octets
    let clair = aes_decypher(ciphered, transfert_key, nonce);
    Ok(clair)
}
