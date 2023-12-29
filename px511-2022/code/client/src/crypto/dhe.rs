//! Ce module `dhe` contient des fonctions crytographiques autour du `Diffie-Hellman Exchange`.
//! On y trouve les fonctions de générations de pair de clés asymétriques et leur gestion sur le client en local.
//! Mais aussi les fonctions de générations de secret partagés dans le sens d'un `DHE`.
use k256::{ecdh::EphemeralSecret, ecdh::SharedSecret, PublicKey, Secp256k1};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::mem::{size_of, transmute};
use std::path::Path;
use std::path::PathBuf;

use crate::account_operation::DeviceComm;

/// Structure contenant une pair de clé asymétrique cryptographique k256.
pub struct PairKey {
    pub ephemeral_secret: EphemeralSecret,
    pub public_key: k256::elliptic_curve::PublicKey<Secp256k1>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EsiPassDhe {
    pub dev_comm: DeviceComm,
    pub session_key: String,
}

/// Génération d'un secret commun
///
/// Utilise DHE et la clé publique d'un autre `Device` pour générer un secret partagé.
/// Retourne le secret partagé sous la forme d'un `SharedSecret`.
///
/// * `other_pub_key` - Clé publique du `Device` avec lequel partager un secret.
pub fn dhe_compute_shared_secret(other_pub_key: PublicKey) -> SharedSecret {
    // Récupération de la pair de clé local
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("private_key.txt");

    let local_pair_key = get_local_priv_key(dir);

    // Génération du secret partagé
    local_pair_key.diffie_hellman(&other_pub_key)
}

/// Génération d'une paire de clé asymétrique cryptographique k256
///
/// Génére de manière aléatoire une paire de clé et les sauvegardes dans deux fichiers en local.
/// Un fichier `private_key.txt` pour la clé privée et un fichier `pub_key.txt` pour la clé publique.
/// Retourne la paire de clé sous la forme d'une `PairKey`.
pub fn gen_dhe_keys() -> PairKey {
    // On génère une courbe élliptique à partir d'un générateur de nombre aléatoire
    let private_key = EphemeralSecret::random(&mut OsRng);
    let pub_key = private_key.public_key();

    // Initialisation de la struct contenant la pair de clé
    let pair_key = PairKey {
        ephemeral_secret: private_key,
        public_key: pub_key,
    };

    // Sérialization de la clé privée
    let private_bytes: &[u8] = unsafe { any_as_u8_slice(&pair_key.ephemeral_secret) };
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("private_key.txt");
    let mut priv_file = File::create(dir).expect("");
    priv_file
        .write_all(private_bytes)
        .expect("Erreur d'écriture de la clé privée");

    // Sérialization de la clé publique
    let str_pub = pair_key.public_key.to_string();
    let pub_bytes = str_pub.as_bytes();
    let mut pub_file = File::create("pub_key.txt").expect("");
    pub_file
        .write_all(pub_bytes)
        .expect("Erreur d'écriture de la clé publique");

    pair_key
}

// Récupéré sur internet, quand est-il du gib endian/little endian?
unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
}

// Récupéré sur internet, quand est-il du gib endian/little endian?
/// Récupère une pair de clé à partir d'un fichier local.
///
/// Retourne la paire de clé sous la forme d'une `PairKey`.
///
/// * `path` - Chemin vers le fichier contenant la pair de clé locale.
fn get_local_priv_key<P: AsRef<Path>>(path: P) -> EphemeralSecret {
    let mut file = File::open(path).expect("");

    let pair_key: EphemeralSecret = {
        let mut h = [0u8; size_of::<EphemeralSecret>()];

        file.read_exact(&mut h[..]).expect("");

        unsafe { transmute(h) }
    };

    pair_key
}
