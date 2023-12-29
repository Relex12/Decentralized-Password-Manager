//! Ce module `chacha` contient des fonctions crytographiques basées sur `XChacha20` pour chiffrer ou déchiffrer des `String`. Les `Nonce` doivent donc faire 24 octets de long.
use chacha20::cipher::generic_array::GenericArray;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::{XChaCha20, XNonce};
use rand::Rng;
use std::str;

use crate::keys_handler::derive_key;

/// Fonction factorisé appliquant simplement un flux de `XChacha20` à un `Vec<u8>` plain.
///
/// Le flux `XChacha20` est généré à partir d'un `String` `master_pwd` qui sera dérivé en clé de chiffrement.
/// Le `u64` `user_id` sert de `salt` à la fonction de dérivation de la clé de chiffrement.
/// La fonction retourne un `Vec<u8> issus du flux de chiffrement.
///
/// * `master_pwd` - Référence d'un `String` utilisé pour générer la clé de chiffrement.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer la clé de chiffrement.
/// * `iv` - `XNonce` Valeur initial utilisé par le flux de chiffrement. Doit être le même au chiffrement et au déchiffrement d'un même message.
/// * `plain` - Vecteur d'octet `Vec<u8>` sur lequel sera appliqué le flux de chiffrement.
fn apply_chacha(master_pwd: &String, user_id: u64, iv: XNonce, plain: Vec<u8>) -> Vec<u8> {
    // Dérivation du MDP maitre
    let derived = derive_key(master_pwd, user_id);

    // Conversion en bytes
    let key_bytes = GenericArray::clone_from_slice(derived.as_bytes());

    // Chiffrement
    let mut res = plain;
    let mut cipher = XChaCha20::new(&key_bytes, &iv);
    cipher.apply_keystream(res.as_mut_slice());

    res
}

/// Fonction de chiffrement utilisant `XChacha20` sur une chaine de caractère.
///
/// Applique un flux de chiffrement `XChacha20` à un `String`. Voir la fonction `apply_chacha` pour plus de détail ici.
/// Génère aléatoirement la valeur initial `XNonce` utilisé par le flux de chiffrement.
/// Le résultat du flux de chiffrement est une suite d'octet qui est encodé sous forme d'un `String` ainsi que le `XNonce` généré.
///
/// * `plaintext` - `String` du texte à chiffrer.
/// * `master_pwd` - Référence d'un `String` utilisé pour générer la clé de chiffrement.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer la clé de chiffrement.
pub fn chapher(plaintext: String, master_pwd: &String, user_id: u64) -> (String, XNonce) {
    // Génération aléatoire du nonce à réutiliser pour le déchiffrement
    let iv: XNonce = XNonce::from(rand::thread_rng().gen::<[u8; 24]>());

    // Conversion en bytes
    let plain = plaintext.into_bytes();

    // Application de chacha
    let chares = apply_chacha(master_pwd, user_id, iv, plain);

    // Encodage des bytes en string
    let encoded = hex::encode(chares);

    (encoded, iv)
}

/// Fonction de déchiffrement utilisant `XChacha20` à une chaine de caractère.
///
/// Récupère un `String` représentant une suite d'octet encodé en chaine de caractères.
/// Décode la suite d'octet en un `Vec<u8> puis applique un flux de chiffrement `XChacha20` à ce `Vec`. Voir la fonction `apply_chacha` pour plus de détail ici.
/// Récupère la valeur initial `XNonce` utilisé par le flux de chiffrement qui doit être généré au chiffrement du texte à déchiffrer.
/// Le résultat du flux de déchiffrement est un `String` représentant le Plaintext.
///
/// * `chiffre` - `String` encodant les octets à déchiffrer.
/// * `master_pwd` - Référence d'un `String` utilisé pour générer la clé de chiffrement.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer la clé de chiffrement.
/// * `nonce` - `XNonce` Valeur initial utilisé par le flux de chiffrement. Doit être le même qu'au chiffrement.
pub fn dechapher(chiffre: String, master_pwd: &String, user_id: u64, nonce: XNonce) -> String {
    // Récupération des bytes chiffré
    let decode = hex::decode(chiffre).expect("Erreur decodage du chiffre");

    // Application de chacha
    let chares = apply_chacha(master_pwd, user_id, nonce, decode);

    // Conversion en String
    let dechiffre = str::from_utf8(&chares).expect("Erreur conversion UTF8 du dechiffre");
    String::from(dechiffre)
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_chapher() {
        assert!(_test_chapher());
    }

    fn _test_chapher() -> bool {
        use crate::chacha::{chapher, dechapher};

        let text = String::from("Hello there kenobi");
        let my_key = String::from("arandomkey");
        let user_id: u64 = 1_000_000_000_000_000;

        let (cypher, nonce) = chapher(text.clone(), &my_key, user_id);

        let decypher = dechapher(cypher, &my_key, user_id, nonce);

        text == decypher
    }
}
