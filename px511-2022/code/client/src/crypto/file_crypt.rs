//! Ce module est une API de chiffrement pour les fichiers de mot de passe.
//! C'est ici qu'il faut modifier les fonctions si l'on veut changer d'algo de chiffrement.
use anyhow::Result;
use chacha20::XNonce;

use crate::chacha::{chapher, dechapher};

/// Fonction chiffrant le contenu d'un fichier EsiPass à l'aide d'un mot de passe maitre.
///
/// Utilise un `String` pour générer une clé de chiffrement symétrique et le `user_id` comme salt pour la génération de la clé.
/// Retourne le contenue chiffré et la valeur initial utilisé pour chiffrer le contenue.
///
/// * `master_pass` - Référence d'un `String` utilisé pour générer la clé de chiffrement.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer la clé de chiffrement.
/// * `file_content` - `String` à chiffrer.
pub fn crypt_file(
    master_pass: &String,
    user_id: u64,
    file_content: String,
) -> Result<(String, XNonce)> {
    let (cipher, iv) = chapher(file_content, master_pass, user_id);
    Ok((cipher, iv))
}

/// Fonction déchiffrant le contenu d'un fichier EsiPass à l'aide d'un mot de passe maitre et d'une valeur initiale donnée.
///
/// Utilise un `String` pour générer une clé de chiffrement symétrique et le `user_id` comme salt pour la génération de la clé.
/// Retourne le contenue chiffré et la valeur initial utilisé pour chiffrer le contenue.
///
/// * `master_pass` - Référence d'un `String` utilisé pour générer la clé de chiffrement.
/// * `user_id` - `u64` représentant l'ID de l'utilisateur et utilisé comme salt dans la fonction de dérivation pour générer la clé de chiffrement.
/// * `iv` - Valeur initial utilisé pour le déchiffrement. Doit être le même qu'au chiffrement du même message.
/// * `to_decrypt` - `String` encodant les octets à déchiffrer.
pub fn decrypt_file(
    master_pass: &String,
    user_id: u64,
    iv: [u8; 24],
    to_decrypt: String,
) -> Result<String> {
    let nonce = XNonce::from(iv);
    Ok(dechapher(to_decrypt, master_pass, user_id, nonce))
}
