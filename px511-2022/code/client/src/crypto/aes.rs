//! Ce module contient des fonctions crytographiques basées sur AES pour chiffrer ou déchiffrer des `String`
use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead, KeyInit},
    Aes256GcmSiv, Nonce,
};
use rand::Rng;

/// Fonction utilisant AES pour chiffrer une chaine de caractère.
///
/// La chaine de caractère est un `String` qui est chiffré à l'aide d'une clé passé en paramètre.
/// La fonction génère un nonce de manière aléatoire utilisé pour le chiffrement puis retourné.
/// Retourne aussi un vecteur de bytes `Vec<u8>` représentant le chiffré.
///
/// * `plaintext` - `String` représentant le texte à chiffrer.
/// * `key` - Tableau d'octet devant être de longueur 32. Sert de clé de chiffrement dans AES.
pub fn aes_cypher(plaintext: String, key: Vec<u8>) -> (Vec<u8>, Nonce) {
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key));
    // Génération aléatoire du nonce à réutiliser pour le déchiffrement
    // Fait 96-bits (12 octets) et doit être unique pour chaque message
    let nonce: Nonce = Nonce::from(rand::thread_rng().gen::<[u8; 12]>());

    (cipher.encrypt(&nonce, plaintext.as_ref()).expect(""), nonce)
}

/// Fonction utilisant AES pour chiffrer un vecteur d'octet.
///
/// Le `Vec<u8>` est chiffré à l'aide d'une clé passé en paramètre.
/// La fonction utilise un nonce pour le chiffrement puis est retourné.
/// Retourne aussi un vecteur de bytes `Vec<u8>` représentant le chiffré.
///
/// * `to_cypher` - `Vec<u8>` représentant les octets à chiffrer.
/// * `key` - Tableau d'octet devant être de longueur 32. Sert de clé de chiffrement dans AES.
/// * `nonce` - Valeur initial utilisé pour le chiffrement
pub fn aes_cypher_bytes_with_nonce(
    to_cypher: Vec<u8>,
    key: Vec<u8>, //GenericArray<u8, U32>,
    nonce: u64,
) -> (Vec<u8>, Nonce) {
    // On génère le chiffreur AES à partir de la clé de chiffrement
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key));

    // On convertit l'ID en Nonce utilisé par la fonction de chiffrement
    let lestring = nonce.to_string();
    let nbytes = lestring.as_bytes();

    let xnonce = Nonce::from_slice(nbytes);

    (
        cipher.encrypt(xnonce, to_cypher.as_slice()).expect(""),
        xnonce.to_owned(),
    )
}

/// Fonction utilisant AES pour chiffrer une chaine de caractère avec une valeur initial donnée.
///
/// La chaine de caractère est un `String` qui est chiffré à l'aide d'une clé passé en paramètre.
/// La fonction récupère un nonce passé en paramètre utilisé pour le chiffrement puis retourné.
/// Retourne aussi un vecteur de bytes `Vec<u8>` représentant le chiffré.
///
/// * `plaintext` - `String` représentant le texte à chiffrer.
/// * `key` - Tableau d'octet devant être de longueur 32. Sert de clé de chiffrement dans AES.
/// * `nonce` - Valeur initial utilisé pour le chiffrement
pub fn aes_cypher_with_nonce(plaintext: String, key: Vec<u8>, nonce: &Nonce) -> (Vec<u8>, Nonce) {
    // On génère le chiffreur AES à partir de la clé de chiffrement
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key));

    (
        cipher.encrypt(nonce, plaintext.as_ref()).expect(""),
        nonce.to_owned(),
    )
}

/// Fonction utilisant AES pour déchiffrer un vecteur d'octets en une chaine de caractère.  
///
/// Le vecteur d'octet est déchiffré à l'aide d'une clé passé en paramètre.
/// La fonction récupère le nonce qui doit être le même utilisé au chiffrement et l'utilise pour le déchiffrement.
/// Retourne un `String` représentant le texte déchiffré.
///
/// * `cyphertext` - Vecteur d'octet représentant le texte à déchiffrer.
/// * `key` - Tableau d'octet devant être de longueur 32. Sert de clé de déchiffrement dans AES.
/// * `nonce` - `Nonce` représentant la valeur initial utilisé pour chiffrer le message en clair. Doit être le même au chiffrement et déchiffrement.
pub fn aes_decypher(cyphertext: Vec<u8>, key: Vec<u8>, nonce: &Nonce) -> String {
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key));
    let plaintext = cipher.decrypt(nonce, cyphertext.as_ref()).expect("");

    String::from_utf8(plaintext).expect("marche pas")
}

/// Fonction utilisant AES pour déchiffrer un vecteur d'octets en un vecteur d'octets.  
///
/// La chaine de caractère est un `String` qui est chiffré à l'aide d'une clé passé en paramètre.
/// La fonction récupère le nonce qui doit être le même utilisé au chiffrement et l'utilise pour le déchiffrement.
/// Retourne un `String` représentant le texte déchiffré.
///
/// * `cyphertext` - Vecteur d'octet représentant le texte à déchiffrer.
/// * `key` - Tableau d'octet devant être de longueur 32. Sert de clé de déchiffrement dans AES.
/// * `nonce` - `Nonce` représentant la valeur initial utilisé pour chiffrer le message en clair. Doit être le même au chiffrement et déchiffrement.
pub fn aes_decypher_bytes(cyphertext: Vec<u8>, key: Vec<u8>, nonce: &Nonce) -> Vec<u8> {
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key));
    cipher
        .decrypt(nonce, cyphertext.as_ref())
        .expect("Erreur déchiffrement AES")
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_aes() {
        assert!(_test_aes());
    }

    fn _test_aes() -> bool {
        use crate::aes::{aes_cypher, aes_decypher};
        use crate::keys_handler::derive_key;

        let text = String::from("Hello there !");
        let my_key = String::from("arandomkey");
        let user_id: u64 = 1_000_000_000_000_000;
        let derived = derive_key(&my_key, user_id);

        let (cypher, nonce) = aes_cypher(text.clone(), derived.as_bytes().to_vec());

        let decypher = aes_decypher(cypher, derived.as_bytes().to_vec(), &nonce);

        text == decypher
    }

    #[test]
    fn test_aes_with_given_nonce() {
        assert!(_test_aes());
    }

    fn _test_aes_with_given_nonce() -> bool {
        use crate::aes::{aes_cypher_with_nonce, aes_decypher, Nonce};
        use crate::keys_handler::derive_key;

        let text = String::from("Hello there !");
        let my_key = String::from("arandomkey");
        let user_id: u64 = 1_000_000_000_000_000;
        let derived = derive_key(&my_key, user_id);

        // On convertit l'ID en Nonce utilisé par la fonction de chiffrement
        let nbytes = hex::decode(user_id.to_string()).unwrap();
        let nonce = Nonce::from_slice(&nbytes);

        let (cypher, nonce) =
            aes_cypher_with_nonce(text.clone(), derived.as_bytes().to_vec(), nonce);

        let decypher = aes_decypher(cypher, derived.as_bytes().to_vec(), &nonce);

        text == decypher
    }
}
