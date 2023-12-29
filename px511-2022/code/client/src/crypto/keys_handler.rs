//! Ce module permet de gérer les différentes clés cryptographiques utilisés dans le programme :
//! * Le mot de passe maitre (MP) servant à chiffrer les mot de passes en local.
//! * La clé de transfert servant à chiffrer les messages envoyés entre les `Device`.
//! * La clé de session servant à générer la clé de transfert grâce à un `Double Ratchet`.
//!
//! Le MP est en réalité hashé par une fonction de dérivation afin de générer une clé de chiffrement.
//! TODO vérifier le MP d'un client.
use aes_gcm_siv::{
    aead::{KeyInit, OsRng},
    Aes256GcmSiv, Nonce,
};

use anyhow::{anyhow, Result};
use k256::PublicKey;
use pbkdf2::{
    password_hash::{Output, PasswordHasher},
    Pbkdf2,
};
use std::fs::File;
use std::io::Write;

use std::io::Read;
use std::path::PathBuf;

use crate::{
    aes::{aes_cypher_bytes_with_nonce, aes_decypher_bytes},
    dhe::dhe_compute_shared_secret,
    rq_operations::send_chg_sess_key,
};

/// Fonction dérivant une chaine de caractère en hash de 32 octets.
///
/// Prend un `String` et le hash par une fonction de dérivation de clé `PBKDF2`.
/// Le salt doit être sur un nombre de caractère multiple de 8.
/// Retourne un `Output` qui est par défaut une suite de 32 octets.
///
/// * `master_key` - Référence d'un `String` à dériver en hash de 32 octets.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer le hash.
pub fn derive_key(master_key: &String, user_id: u64) -> Output {
    utils::afaire!("keys_handler.rs : Fonction de dérivation du MP - vérifier le MP");
    let salt = user_id.to_string();
    let pwd_hash = Pbkdf2
        .hash_password(master_key.as_bytes(), salt.as_str())
        .expect("Erreur hashage du MDP maitre");

    pwd_hash.hash.unwrap()
}

/// Fonction générant une clé de session pour l'application.
///
/// Génére une clé de session qui est un secret partagé entre tous les `Device`.
/// La fonction va ensuite sauvegarder cette clé dans un fichier avant de la retourner dans un `Vec<u8>`.
pub fn gen_session_key() -> Vec<u8> {
    // Génération aléatoire de la clé de session
    let session_key = Aes256GcmSiv::generate_key(&mut OsRng);
    let encoded_key = hex::encode(session_key);

    // Sauvegarde de la clé de session
    write_session_key(encoded_key);

    session_key.to_vec()
}

/// Fonction sauvegardant une clé de session dans un fichier
///
/// Récupère une chaine de caractère sous la forme d'un `String` représentant la clé de session.
/// Doit être les octets de la clé de session encodé sous la forme d'un `String`.
/// Le `String` est ensuite écrit dans un fichier `session_key.txt`.
///
/// * `session_key` : `String` représentant la clé de session.
pub fn write_session_key(session_key: String) {
    // Récupération du chemin absolu du projet
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // On concatène avec le nom du fichier
    path.push("session_key.txt");

    // Création et écriture du fichier contenant la clé de session
    let mut file = File::create(path).expect("Erreur création du fichier de la clé de session.");
    file.write_all(session_key.as_bytes())
        .expect("Erreur écriture dans le fichier de la clé de session.");
}

/// Fonction renouvelant la clé de session en local si besoin
///
/// Vérifie si la clé de session a besoin d'être renouveler. CONDITIONS A DETERMINER
/// Si c'est le cas, regènere une nouvelle clé de session et envoi une notif ChangeSessionKey.
pub fn renew_session_key() -> Result<()> {
    // On demande au serveur si on peut changer la clé de session
    match send_chg_sess_key() {
        Ok(()) => {
            // C'est bon alors on regénère une nouvelle clé de session
            gen_session_key();
            Ok(())
        }

        Err(_) => Err(anyhow!("Refus renouvellement de la session key")),
    }
}

/// Fonction récupérant la clé de session de l'application
///
/// Lit le fichier `session_key.txt` contenant la clé de session qui est un secret partagé entre tous les `Device`.
/// La fonction retourne la clé de session dans un `Vec<u8>`.
pub fn get_session_key() -> Vec<u8> {
    // Récupération du chemin absolu du projet
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // On concatène avec le nom du fichier
    path.push("session_key.txt");

    let mut file_content = String::new();
    // Création et écriture du fichier contenant la clé de session
    let mut file = File::open(path).expect("Erreur ouverture du fichier de la clé de session.");
    file.read_to_string(&mut file_content)
        .expect("Erreur lecture du fichier de la clé de session.");

    hex::decode(file_content).unwrap()
}

/// Fonction d'encryption de la clé de session.
///
/// Permet de chiffrer la clé de session afin de l'échanger entre les `Device`.
/// Récupère une paire de clé DHE et la clé publique du `Device` destinataire puis calcul un secret partagé.
/// Puis utilise le secret partagé et `AES` pour chiffrer la clé de session.
/// Retourne le chiffré de la clé de session.
///
/// * `session_key` - Vecteur d'octet représentant la clé de session.
pub fn encrypt_session_key(pair_pub_key: PublicKey, user_id: u64) -> Vec<u8> {
    // Récupération du secret partagé
    let shared_secret = dhe_compute_shared_secret(pair_pub_key);

    // Récupération de la clé de session à chiffrer
    let session_key = get_session_key();

    // On récupère une clé de chiffrement à partir du secret partagé
    let raw = shared_secret.raw_secret_bytes().to_vec();

    // Chiffrement de la clé de session
    aes_cypher_bytes_with_nonce(session_key, raw, user_id).0
}

pub fn decrypt_session_key(pair_pub_key: PublicKey, to_decrypt: Vec<u8>, user_id: u64) -> Vec<u8> {
    // Récupération du secret partagé
    let shared_secret = dhe_compute_shared_secret(pair_pub_key);

    // On récupère une clé de chiffrement à partir du secret partagé
    let key = shared_secret.raw_secret_bytes().to_vec();

    // On convertit l'ID en Nonce utilisé par la fonction de chiffrement
    let id_str = user_id.to_string();
    let nonce = Nonce::from_slice(id_str.as_bytes());

    // Chiffrement de la clé de session
    aes_decypher_bytes(to_decrypt, key, nonce)
}

#[cfg(test)]
mod tests {
    use crate::{
        dhe::gen_dhe_keys,
        keys_handler::{decrypt_session_key, encrypt_session_key},
    };

    #[test]
    fn test_gen_session_key() {
        assert!(_test_gen_session_key());
    }

    fn _test_gen_session_key() -> bool {
        use crate::keys_handler::{gen_session_key, get_session_key};

        // génération aléatoire de la clé de session et écriture dans un fichier
        let attendu = gen_session_key();
        // récupération de la clé de session depuis le fichier
        let get_key = get_session_key();

        attendu == get_key
    }

    #[test]
    fn test_crypt_session_key() {
        assert!(_test_gen_session_key());
    }

    fn _test_crypt_session_key() -> bool {
        use crate::keys_handler::gen_session_key;
        use k256::ecdh::EphemeralSecret;
        use rand_core::OsRng;

        let user_id: u64 = 1_000_000_000_000_000;

        // génération de la paire de clé pour le DHE
        gen_dhe_keys();

        // génération de la clé de session à chiffrer
        let attendu = gen_session_key();

        // création d'un clé publique simulant un autre device
        let pair_private_key = EphemeralSecret::random(&mut OsRng);
        let pair_pub_key = pair_private_key.public_key();

        // chiffrement et déchiffrement de la clé de session
        let encrypted = encrypt_session_key(pair_pub_key, user_id);
        let get_key = decrypt_session_key(pair_pub_key, encrypted, user_id);

        // Comparaison
        attendu == get_key
    }
}
