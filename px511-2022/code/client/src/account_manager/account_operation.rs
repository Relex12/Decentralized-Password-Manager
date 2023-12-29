//! Ce module contient toutes les fonctions concernant les opérations sur le compte et les Devices qui y sont associés.
use anyhow::Result;
use k256::PublicKey;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use crate::dhe::*;
use crate::keys_handler::gen_session_key;
use crate::rq_operations::{build_reqwest_client, send_dhe, send_message};
use utils::data::comm_struct::*;

#[derive(Serialize, Deserialize, Clone)]
/// Structure représentant les information du compte stockées en local sur l'appareil.
pub struct LocalAccountData {
    // Structure représentant le compte de l'utilisateur
    pub account: Account,
    // Numero identifiant le `Device` local actuel
    pub local_dev_id: u64,
    // Les informations de tous les `Device` associés au compte
    pub devices: HashSet<DeviceComm>,
}

#[derive(Serialize, Deserialize, Clone, Eq, Debug)]
/// Structure contenant les informations d'un `Device` ainsi que sa clé publique actuel permettant de faire un `DHE` avec ce `Device`.
pub struct DeviceComm {
    pub info: DeviceInfo,
    pub pub_dhe_key: Option<PublicKey>,
}

impl PartialEq for DeviceComm {
    fn eq(&self, other: &DeviceComm) -> bool {
        self.info.device_id == other.info.device_id
    }
}

impl std::hash::Hash for DeviceComm {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.info.device_id.hash(state);
    }
}

impl std::borrow::Borrow<u64> for DeviceComm {
    fn borrow(&self) -> &u64 {
        &self.info.device_id
    }
}

/// Méthode pour charger un compte
/// Génère la structure d'un compte et l'envoi au serveur
/// Simule un challenge avec le téléphone
pub fn load_account(num_phone: u64, name: String, first_name: String) -> Result<()> {
    // Création du compte
    let new_accnt = init_account(num_phone, name, first_name);

    // Attention en réalité cette clé est censé être unique à l'appareil et servir à la mise
    // en place d'une signature lors de l'envoie d'un EsiPassMsg
    // La clé est temporairement générée de la sorte pour éviter une erreur.
    let key = gen_dhe_keys().public_key;
    let dev_info = DeviceInfo {
        device_id: 0,
        dev_name: String::from("Unknown"),
        public_key: key,
    };

    // Création de la requête pour le serveur
    let mut rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::LoadAccount,
        exp: Some(dev_info),
        dest: None,
        accnt: new_accnt,
        signature: None,
        msg: None,
    };

    // Création du Client reqwest pour l'envoi de paquet
    let client = build_reqwest_client().expect("Erreur création client");

    // Simulation du challenge
    //send_challenge(&client, new_accnt, dev_info)?;

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur
    let rep = send_message(&client, j_rqst).last().unwrap().to_owned();

    // Sauvarge des informations du compte en local.
    save_account(rep, &mut rqst.accnt);

    Ok(())
}

/// Génère une structure `Account` représentant un compte et l'envoi au serveur
/// avec un `EsiPassRqst` dans une requête web via un `rqwest::blocking::Client`
///
/// * `device_name` - `String` représentant le nom donnés au device par l'utilisateur.
/// * `phone_num` - `u64` indiquant le numéro de téléphone renseigné par l'utilisateur.
pub fn create_account(
    device_name: String,
    num_phone: u64,
    name: String,
    first_name: String,
) -> Result<()> {
    // Création du compte
    let new_accnt = init_account(num_phone, name, first_name);

    // Génération des clés cryptographique du device servant à la communication avec le client
    let key = gen_dhe_keys().public_key;

    let dev_info = DeviceInfo {
        device_id: 0,
        dev_name: device_name,
        public_key: key,
    };

    // Création de la requête pour le serveur
    let mut rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::LoadAccount,
        accnt: new_accnt,
        signature: None,
        exp: Some(dev_info),
        msg: None,
        dest: None,
    };

    // Création du Client web reqwest pour l'envoi de la requete
    let client = build_reqwest_client().expect("Erreur création client");

    // Simulation du challenge
    //send_challenge(&client, new_accnt, dev_info)?;

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur
    let rep = send_message(&client, j_rqst).last().unwrap().to_owned();
    save_account(rep, &mut rqst.accnt);

    Ok(())
}

/// Initialisation d'un `Account`.
///
/// Cree un struct `Account` et rempli le champ `phone` à partir du numéro de téléphone donné.
/// Le `user_id` est initialisé à None car doit être donné par le serveur.
///
/// * `phone_num` - Numéro de téléphone associé au compte à initialiser.
pub fn init_account(num_phone: u64, name: String, first_name: String) -> Account {
    // Création du compte client
    Account {
        user_id: None,
        phone: num_phone,
        name,
        first_name,
    }
}

/// Sauvegarde les informations d'un `LocalAccountData` dans un fichier en local
///
/// Récupère le `user_id` de l'utilisateur dans une `EsiPassResp` renvoyé par le serveur.
/// Complète le `account` puis génère un `LocalAccountData` pour le sauvegarder dans un fichier local.
///
/// * `server_resp` - Réponse `EsiPassResp` renvoyée par le serveur.
/// * `account` - `Account` de l'utilisateur à compléter avec l'ID généré par le serveur.
pub fn save_account(server_resp: EsiPassMsg, account: &mut Account) {
    account.user_id = server_resp.accnt.user_id;

    // Récupération de la liste des DeviceInfo fournit par le serveur
    let mut devices: Vec<DeviceInfo> = serde_json::from_str(server_resp.msg.unwrap().as_str())
        .expect("Erreur désérialisation du msg ");
    let comms = update_local_devices_info(&mut devices, None);

    // Récupération du dernier Device enregistré par le serveur
    let dev_info = devices
        .last()
        .expect("SaveAccount: Le vecteur de Device est vide");

    // Création du LocalAccountData
    let account_data = LocalAccountData {
        account: account.to_owned(),
        local_dev_id: dev_info.device_id,
        devices: comms,
    };

    // Récupération du localaccount actuel
    let actual_accnt_res = get_account();

    // Sauvegarde des modifications dans un fichier
    serialize_local_accnt(account_data);

    // Si c'est la première fois que l'on charge le compte et qu'il contient plus d'un appareil
    // Envoi d'une requête DHE pour récupérer la clé de session.
    if actual_accnt_res.is_err() && devices.len() > 1 {
        // Création du dernier DeviceComm issu du server
        let last_comm = DeviceComm {
            info: dev_info.clone(),
            pub_dhe_key: Some(gen_dhe_keys().public_key),
        };
        send_dhe(last_comm);
    }
    // Si l'étape correspond à la création du compte, on génère une clé de session par défaut.
    else if actual_accnt_res.is_err() && devices.len() == 1 {
        gen_session_key();
    }
}

/// Récupère les informations du compte en local
pub fn get_account() -> Result<LocalAccountData> {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("account");

    let mut file = File::open(dir)?;

    // On lit le contenue du fichier du compte local
    let mut acc = String::new();
    file.read_to_string(&mut acc)
        .expect("Erreur lecture du fichier du compte local");

    // On deserialize
    let pass: LocalAccountData = serde_json::from_str(acc.as_str())
        .expect("Erreur deserialization de la struct LocalAccountData");

    Ok(pass)
}

/// Récupération des informations du compte chargé en local.
///
/// Récupère `DeviceInfo` à partir de la liste de device d'un `LocalACcountData`.
/// Le `DeviceInfo` correspond au `Device` sur lequel tourne le client.
///
/// * `acc` - Référence vers un `LocalAccountData` où chercher les info du `Device` local.
pub fn get_local_device(acc: &LocalAccountData) -> Option<DeviceInfo> {
    Some(get_device(acc, acc.local_dev_id).unwrap().info)
}

/// Récupération des informations d'un `Device`.
///
/// Récupère `DeviceComm` à partir de la liste de device d'un `LocalACcountData`.
/// Le `DeviceComm` correspond à l'id renseigné en paramètre.
///
/// * `acc` - Référence vers un `LocalAccountData` où chercher les info du `Device` recherché.
/// * `dev_id` - ID du `Device` recherché.
pub fn get_device(acc: &LocalAccountData, dev_id: u64) -> Option<DeviceComm> {
    acc.devices.get(&dev_id).cloned()
}

/// Met à jours un `HashSet<DeviceComm>` s'il existe ou le créer dans le cas échéant.
///
/// Ajoute successivement les `DeviceInfo` contenu dans `devices` dans `comms` s'ils n'y sont
/// pas déjà présent.
///
/// * `devices` - Référence vers un `Vec<DeviceInfo>` qui contient la lsite des appareils fournis par le server.
/// * `dcomms` - Peut contenir le HashSet du compte actuel ou le créer s'il n'existe pas.
pub fn update_local_devices_info(
    devices: &mut Vec<DeviceInfo>,
    comms: Option<HashSet<DeviceComm>>,
) -> HashSet<DeviceComm> {
    // Initialisation du HashSet
    let mut comms_set: HashSet<DeviceComm> = match comms {
        Some(set) => set,
        None => HashSet::new(),
    };

    // Complétion du HashSet de DeviceComm avec le vecteur de DeviceInfo
    for dev in devices {
        comms_set.insert(DeviceComm {
            info: dev.clone(),
            pub_dhe_key: None,
        });
    }

    comms_set
}

/// Sauvegarde en local d'un `LocalAccountData`.
///
/// Création du fichier account et sérialisation du 'LocalAccountData'.
///
/// * `account_data` - structure `LocalAccountData` à sauvegarder.
pub fn serialize_local_accnt(account_data: LocalAccountData) {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("account");

    // Création du fichier et sauvegarde des informations du compte dedans.
    let mut file = File::create(dir).expect("Erreur lors de la création du fichier de compte");
    let j_acc = serde_json::to_string(&account_data).expect("Erreur serialisation de la requete");
    file.write_all(j_acc.as_bytes())
        .expect("Erreur d'écriture dans le fichier du compte local");
}
#[cfg(test)]
mod tests {

    fn _test_save_account() -> bool {
        use k256::ecdh::EphemeralSecret;
        use rand_core::OsRng;

        use super::*;

        let mut acc: Account = Account {
            phone: 1_234_456_789,
            user_id: Some(100_000_000_000),
            name: String::from("Random"),
            first_name: String::from("Aléatoire"),
        };

        let priv_key = EphemeralSecret::random(&mut OsRng);
        let pub_key = priv_key.public_key();

        let local_dev = DeviceInfo {
            dev_name: String::from("app"),
            device_id: 0,
            public_key: pub_key,
        };

        let mut devices: Vec<DeviceInfo> = Vec::new();
        devices.push(local_dev);
        let msg_devices: String =
            serde_json::to_string(&devices).expect("Erreur sérialisation DeviceInfo");

        let resp = EsiPassMsg {
            rq_type: EsiPassMsgType::LoadAccount,
            exp: None,
            dest: Some(0),
            accnt: acc.clone(),
            signature: None,
            msg: Some(msg_devices),
        };

        save_account(resp, &mut acc);

        let local = get_account().expect("Erreur récupération du compte");

        local.account.phone == acc.phone
    }

    #[test]
    fn test_save_account() {
        assert!(_test_save_account());
    }
}
