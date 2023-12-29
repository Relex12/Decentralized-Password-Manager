//! Ce module gère toute les opérations sur les `EsiPwd`.
//! Création, modification et synchronisation avec les autres `Devices`.
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

use utils::data::comm_struct::*;

use crate::account_operation::get_account;
use crate::account_operation::get_local_device;
use crate::file_crypt::*;
use crate::rq_operations::build_reqwest_client;
use crate::rq_operations::pull_notif;
use crate::rq_operations::send_message;
use crate::session_crypt::crypt_rqst;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
/// Structure représentant un mot de passe.
/// Contient l'identifiant associé au mot de passe.
/// Le mot de passe lui même.
/// L'URL du site sur lequel utiliser ce couple.
pub struct EsiPwd {
    website: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash)]
/// Structure représentant un contenue chiffré et la valeur initiale utilisé pour chiffrer ce contenue.
pub struct EsiFile {
    pub esifile_id: u64,
    pub iv: [u8; 24],
    pub pwd: String,
}

/// Fonction de création d'un mot de passe dans l'application.
///
/// Créer un mot de passe, l'enregistre dans l'application en créant un fichier chiffré.
/// Envoi ensuite la mise à jour au serveur pour la transmettre aux autres `Devices`.
/// Retourne `Ok` si la création de mot de passe s'est bien passée.
///
/// * `master` - Référence d'un `String` représentant le mot de passe maitre utilisé pour le chiffrement.
/// * `url` - `String` représentant l'URL du site web sur lequel est à utiliser le couple identifiant mot de passe.
/// * `user` - `String` représentant l'identifiant associé au mot de passe à créer.
/// * `pwd` - `String` représentant le mot de passe à créer.
pub fn create_pwd(master: &String, url: String, user: String, pwd: String) -> Result<EsiFile> {
    let cyphered_file = create_pwd_wo_updt(master, url, user, pwd);
    pull_notif();
    // Envoi du MDP au serveur
    send_updt_pwd_file(&cyphered_file)?;

    Ok(cyphered_file)
}

pub fn create_pwd_wo_updt(master: &String, url: String, user: String, pwd: String) -> EsiFile {
    // Initialisation du MDP
    let pass = EsiPwd {
        website: url,
        username: user,
        password: pwd,
    };

    // Ecriture du contenue chiffré dans un fichier
    write_esipass(master, pass)
}

/// Fonction envoyant un fichier de mot de passe chiffré au serveur.
///
/// Créé un client web pour envoyer le message chiffré au serveur afin de le transférer aux autres `Devices`.
/// Le message chiffré correspond à un fichier de mot de passe doublement chiffré.
/// Retourne `Ok` si tout se passe bien. Une erreur sinon.
///
/// * `cyphered_file` - `String` du message chiffré à envoyer au serveur.
pub fn send_updt_pwd_file(cyphered_file: &EsiFile) -> Result<()> {
    // Chargement du compte et de l'appareil local.
    let acc = get_account()?;
    let local_device = get_local_device(&acc).expect("Local device not found");

    // Deuxième chiffrement du fichier de mot de passe par une clé de transfert
    let msg_content =
        serde_json::to_string(&cyphered_file).expect("Erreur serialisation du EsiFile");
    let msg = crypt_rqst(msg_content, acc.account.user_id.unwrap()).unwrap();

    // La signature n'est pas encore prise en compte.
    let rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::UpdatePwdFile,
        accnt: acc.account,
        signature: Some(0),
        exp: Some(local_device),
        msg: Some(msg),
        dest: None,
    };

    // Création du Client reqwest pour l'envoi de paquet
    let client = build_reqwest_client().expect("Erreur création client");

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur
    send_message(&client, j_rqst);
    Ok(())
}

/// Fonction de récupération d'un fichier de mot de passe.
///
/// Le nom d'un fichier de mot de passe correspond à l'URL du site sur lequel il est utilisé.
/// <span style="background-color: #FFFF00;color: black;">Vue ce qui se passe avec LastPass il ne faudrait plus utiliser les URL mais plutot un hash des URL.</span>
/// Retourne le fichier sous la forme d'un `File`.
///
/// * `file_id` - Entier correspondant à l'ID du fichier de mot de passe à récupérer.
pub fn get_file_pwd(file_id: &str) -> File {
    // Récupération du chemin absolu du projet
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // On concatène avec le nom du fichier qui est l'url du site associé au mot de passe
    dir.push("esipass/".to_string() + file_id + ".esipass");

    // Ouverture et envoie du fichier
    File::open(dir).expect("Erreur ouverture du fichier de MDP")
}

/// Chiffre à l'aide d'un mot de passe maitre et écrit un `EsiPwd` dans un fichier.
///
/// Serialize en `JSON` la struct `EsiPWd` avant de chiffrer le texte qui en résulte.
/// Ecrit ensuite le chiffré dans un fichier se nommant d'après l'URL du `EsiPwd`.
/// Retourne le contenu chiffré du fichier sous forme d'un `String`.
///
/// * `master` - Référence vers un `String` correspondant au mot de passe maitre de l'utilisateur.
/// * `pass` - Structure `EsiPwd` correspondant au mot de passe à chiffrer et enregistrer.
pub fn write_esipass(master: &String, pass: EsiPwd) -> EsiFile {
    // Serialisation en JSON de la structure
    let j_pass = serde_json::to_string(&pass).expect("Erreur serialisation du EsiPwd");

    // Récupération de l'ID de l'utilisateur
    let user_id = get_account()
        .expect("Local account not found")
        .account
        .user_id
        .unwrap();

    // Chiffrement du json représentant le EsiPwd
    let (cypher, iv) =
        crypt_file(master, user_id, j_pass).expect("Erreur de chiffrement du EsiPWd");

    // Génération d'un id pour le EsiFile
    let mut rng = rand::thread_rng();
    let mut new_id: u64 = rand::Rng::gen_range(&mut rng, 0..999_999_999);

    // Recherche d'un ID unique.
    let mut generated = false;
    while !generated {
        let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dir.push("esipass/".to_string() + new_id.to_string().as_str() + ".esipass");
        new_id = match File::open(dir) {
            Err(_) => {
                generated = !generated;
                new_id
            }
            // Si un fichier est trouvé un nouvel id est généré.
            Ok(_) => rand::Rng::gen_range(&mut rng, 0..999_999_999),
        };
    }

    // Création de la structure représentant le contenu du fichier avec le nonce et le chiffre
    let file_content = EsiFile {
        esifile_id: new_id,
        iv: iv.into(),
        pwd: cypher,
    };

    // Ecriture du EsiFile dans un fichier
    write_esi_file(&file_content);

    file_content
}

pub fn write_esi_file(esi_file: &EsiFile) -> String {
    // Sérialisation en JSON de la structure EsiFile
    let j_content = serde_json::to_string(&esi_file).expect("Erreur serialisation du EsiFile");

    // Récupération du chemin absolu du projet
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // On concatène avec le nom du fichier qui est l'url du site associé au mot de passe
    dir.push("esipass/".to_string() + esi_file.esifile_id.to_string().as_str() + ".esipass");

    // Création/ouverture et écriture du fichier de mot de passe
    let mut pass_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(dir)
        .expect("Erreur ouverture/création du fichier de MDP");

    pass_file
        .write_all(j_content.as_bytes())
        .expect("Erreur d'écriture dans le fichier de MDP");

    j_content
}

/// Fonction de récupération d'un mot de passe sous la forme d'un `EsiPwd`.
///
/// Utilise l'URL du site web sur lequel est utilisé le mot de passe pour récupérer un fichier en local.
/// Récupère le contenue du fichier et le déchiffre à l'aide du mot de passe maitre de l'utilisateur.
/// Retourne un `EsiPwd` si tout se passe bien. Une erreur sinon.
///
/// * `master` - Référence à `String` correspondant au mot de passe maitre de l'utilisateur.
/// * `esi_pass_id` - Entier correspondant à l'ID du fichier de mot de passe à récupérer.
pub fn get_pwd(master: &String, esi_pass_id: u64) -> Result<EsiPwd> {
    // Conversion de l'ID pour avoir le nom du fichier
    let file_id = esi_pass_id.to_string();

    // Récupération de l'ID de l'utilisateur
    let user_id = get_account()?.account.user_id.unwrap();

    // On recupere le fichier
    let mut pass_file = get_file_pwd(&file_id);

    // On lit le contenue du fichier de MDP
    let mut file_content = String::new();
    pass_file
        .read_to_string(&mut file_content)
        .expect("Erreur de lecture dans le fichier EsiFile");

    let esi_file: EsiFile =
        serde_json::from_str(file_content.as_str()).expect("Erreur deserialization du EsiFile");

    // On déchiffre
    let content = decrypt_file(master, user_id, esi_file.iv, esi_file.pwd)
        .expect("Erreur déchiffrement du contenue du fichier de MDP");

    // On deserialize
    let pass: EsiPwd =
        serde_json::from_str(content.as_str()).expect("Erreur deserialization du EsiPwd");

    Ok(pass)
}

/// Fonction permettant de modifier un mot de passe.
///
/// Récupère un fichier de mot de passe et déchiffre son contenue pour récupérer un `EsiPwd`.
/// Modifie le contenue du `EsiPwd` avec les paramètres passés.
/// Enregistre le `EsiPwd` mis à jour dans le même fichier en écrasant l'ancien.
/// Retourne `Ok` si tout se passe bien. Une erreur sinon.
///
/// * `master` - Référence à `String` correspondant au mot de passe maitre de l'utilisateur.
/// * `old_url` - `String` correspondant à l'ancien URL du site web sur lequel est utilisé le mot de passe à modifier.
/// * `new_url` - `String` correspondant au nouveau URL du site web sur lequel est utilisé le mot de passe à modifier.
/// * `new_user` - `String` correspondant au nouvel identifiant correspondant au mot de passe à modifier.
/// * `new_pwd` - `String` correspondant au nouveau mot de passe.
pub fn modify_pwd(
    master: &String,
    file_id: u64,
    new_url: String,
    new_user: String,
    new_pwd: String,
) -> Result<()> {
    // Récupération du MDP
    let mut pass = get_pwd(master, file_id)?;

    // Modification du MDP
    pass.website = new_url;
    pass.username = new_user;
    pass.password = new_pwd;

    // Puis on écrit dans le fichier de MDP
    write_esipass(master, pass);

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::pwd_handler::{create_pwd_wo_updt, EsiPwd};

    fn _test_get_pwd() -> bool {
        use super::get_pwd;

        let attendu = EsiPwd {
            website: "hello.there".to_string(),
            username: "Kenobi".to_string(),
            password: "Grievous".to_string(),
        };

        let esi_file = create_pwd_wo_updt(
            &"my master pwd".to_string(),
            "hello.there".to_string(),
            "Kenobi".to_string(),
            "Grievous".to_string(),
        );

        println!("EsiFile ID : {}", esi_file.esifile_id);
        let res = get_pwd(&"my master pwd".to_string(), esi_file.esifile_id);

        if res.is_ok() && (res.unwrap() == attendu) {
            true
        } else {
            false
        }
    }

    #[test]
    fn test_get_pwd() {
        assert!(_test_get_pwd());
    }
}
