//! Ce module contient toutes les fonctions de communications entre le client et le serveur.
use anyhow::anyhow;
use anyhow::Result;
use reqwest::blocking::Client;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::exit;
use utils::data::comm_struct::*;

use crate::account_manager::account_operation::*;
use crate::crypto::dhe::*;
use crate::crypto::keys_handler::*;
use crate::pwd_handler::write_esi_file;
use crate::pwd_handler::EsiFile;
use crate::session_crypt::crypt_rqst;
use crate::session_crypt::decrypt_rqst;

pub const SERV_URL: &str = "https://15.237.127.196:40443";
const PEM: &str = "cert.pem";

/*
#################################################################
                        FONCTION DE RECEPTION
#################################################################
*/

/// Fonction pour récupérer des messages auprès du serveur.
///
/// Envoi une requête au serveur pour demander si des messages sont à récupérer.
/// Attend une réponse du serveur indiquant s'il y a des messages ou non.
/// La réponse contient les messages s'il y en a présent sur le serveur.
/// En fonction de du type de réponse, différentes actions sont effectués.
///
///
pub fn pull_notif() {
    // Création du client pour communiquer avec le serveur
    let client = build_reqwest_client().expect("Erreur création client");

    // Récupération des infos du compte et du device
    let mut local_acc = get_account().expect("Erreur récupération du compte");

    // Création de la requête pour le serveur
    let rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::PullNotif,
        accnt: local_acc.account.clone(),
        signature: Some(0),
        exp: get_local_device(&local_acc),
        msg: None,
        dest: None,
    };

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur et récupération des potentiels réponses
    let resp = send_message(&client, j_rqst);

    // On boucle sur les réponses pour toutes les traiter
    for msg in resp {
        match msg.rq_type {
            // Si on a une requete de DHE
            EsiPassMsgType::RequestDHE => {
                println!("PULL NOTIF : RECEIVE REQUEST DHE");
                handle_request_dhe(msg, &mut local_acc, &client);
            }
            // Si on reçoit une réponse à une requete DHE
            EsiPassMsgType::RespDHE => {
                println!("PULL NOTIF : RECEIVE RESP DHE");
                handle_resp_dhe(msg);
            }
            // Si on a une mise à jour de la clé de session
            EsiPassMsgType::ChangeSessionKey => {
                println!("PULL NOTIF : RECEIVE CHANGE SESSION KEY");
                handle_chg_sess_key();
            }
            // Si on a une mise à jour des informations du compte
            EsiPassMsgType::LoadAccount => {
                println!("PULL NOTIF : RECEIVE LOAD ACCOUNT");
                handle_load_accnt(msg);
            }
            EsiPassMsgType::Challenge => {
                println!("PULL NOTIF : RECEIVE CHALLENGE");
            }
            EsiPassMsgType::UpdatePwdFile => {
                println!("PULL NOTIF : RECEIVE UPDATE PWD FILE");
                handle_update_pwd_file(msg);
            }
            EsiPassMsgType::RequestAllPwds => {
                println!("PULL NOTIF : RECEIVE REQUEST ALL PASSWORDS");
                handle_request_all_passwords(msg);
            }
            EsiPassMsgType::PullNotif => {
                println!("EsiPassMsgType PullNotif non attendu.");
            }
        }
    }
}

/// Fonction de gestion d'un EsiPassMsg de type RequestDHE
///
/// Récupère un EsiPassMsg de type RequestDHE reçu du serveur
/// La requête contient alors en message une structure `DeviceInfo`.
/// Cette structure contient les infos du Device expéditeur.
/// La fonction va alors chiffrer la clé de session reçu et l'envoyer.
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type RequestDHE
/// * `local_acc` : `Account` de l'utilisateur
/// * `client` : `Client` web pour l'envoi de la réponse
pub fn handle_request_dhe(rep: EsiPassMsg, local_acc: &mut LocalAccountData, client: &Client) {
    // On récupère l'ID du device qui nous envoi sa clé publique pour un DHE
    //let exp_dev = rep.exp.unwrap();
    // Récupération du Device local
    let dev_info = get_local_device(local_acc).unwrap();

    // Récupération du device en local et depuis la requete
    // C'est ce device qui envoie sa clé publique de DHE
    // let mut local_exp_comm = get_device(local_acc, exp_dev.device_id).unwrap();
    let dist_exp_comm: DeviceComm = serde_json::from_str(rep.msg.unwrap().as_str())
        .expect("RequestDHE: Erreur déserialisation du message");

    // // Mise à jour de ce Device dans LocalAccountData
    // local_exp_comm.pub_dhe_key = dist_exp_comm.pub_dhe_key;
    // local_acc.devices.insert(local_exp_comm);

    // On génère notre paire de clé pour ce DHE
    let pair = gen_dhe_keys();

    // On chiffre la clé de session avec le secret partagé issue du DHE avec la clé publique de l'expéditeur
    let enc_sess_key = encrypt_session_key(
        dist_exp_comm.pub_dhe_key.unwrap(),
        local_acc.account.user_id.unwrap(),
    );

    // Il faut créer la structure
    let dhe_resp = EsiPassDhe {
        dev_comm: DeviceComm {
            info: dev_info.clone(),
            pub_dhe_key: Some(pair.public_key),
        },
        session_key: hex::encode(enc_sess_key),
    };

    let msg = serde_json::to_string(&dhe_resp)
        .expect("RequestDHE: Erreur sérialisation message de réponse");

    // Création du message RespDHE
    let esi_msg = EsiPassMsg {
        rq_type: EsiPassMsgType::RespDHE,
        exp: Some(dev_info),
        dest: Some(dist_exp_comm.info.device_id),
        accnt: local_acc.account.clone(),
        signature: None,
        msg: Some(msg),
    };

    // Sérialisation et envoi du message de réponse
    let response =
        serde_json::to_string(&esi_msg).expect("RequestDHE: Erreur sérialization réponse DHE");
    send_message(client, response);
}

/// Fonction de gestion d'un EsiPassMsg de type RespDHE
///
/// Récupère une réponse à un RequestDHE sous la forme d'un EsiPassMsg de type RespDHE
/// La requête contient alors en message une structure EsiPassDhe.
/// Cette structure contient les infos du Device expéditeur ainsi que la clé de session chiffré par le secret partagé du DHE.
/// La fonction va alors déchiffrer la clé de session reçu et la sauvegarder.
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type RespDHE
/// * `accnt` : `Account` de l'utilisateur
pub fn handle_resp_dhe(resp: EsiPassMsg) {
    // Récupération de la clé chiffré et de la clé publique de l'expéditeur
    let dhe_resp: EsiPassDhe = serde_json::from_str(resp.msg.unwrap().as_str())
        .expect("RespDHE: Erreur désérialisation du message");

    let enc_session_key =
        hex::decode(dhe_resp.session_key).expect("RespDHE: erreur decodage session key chiffré");
    let exp_pub_key = dhe_resp.dev_comm.pub_dhe_key.unwrap();

    // Récupération des infos du compte et du device
    let local_acc = get_account().expect("Erreur récupération du compte");

    // Déchiffrement de la clé de session reçu
    let session_key = decrypt_session_key(
        exp_pub_key,
        enc_session_key,
        local_acc.account.user_id.unwrap(),
    );

    // Encodage et sauvegarde de la clé de session
    let encoded_key = hex::encode(session_key);
    write_session_key(encoded_key);
}

/// Fonction de gestion d'un EsiPassMsg de type ChangeSessionKey
///
/// Récupère un EsiPassMsg de type ChangeSessionKey
/// La fonction va alors envoyer un EsiPassMsg de type RequestDHE
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type RespDHE
/// * `accnt` : `Account` de l'utilisateur
pub fn handle_chg_sess_key() {
    // Création du client pour communiquer avec le serveur
    let client = build_reqwest_client().expect("Erreur création client");

    // Récupération des infos du compte et du device
    let local_acc = get_account().expect("Erreur récupération du compte");

    let pub_key = gen_dhe_keys().public_key;
    let dev_info = get_local_device(&local_acc);
    let dev_comm = DeviceComm {
        info: dev_info.clone().unwrap(),
        pub_dhe_key: Some(pub_key),
    };

    // Création de la requête pour le serveur
    let rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::RequestDHE,
        exp: dev_info,
        dest: None,
        accnt: local_acc.account,
        signature: Some(0),
        msg: Some(serde_json::to_string(&dev_comm).expect("Erreur serialisation DeviceComm")), // Doit être le DevComm avec la clé publique dedans
    };

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur et récupération des potentiels réponses
    send_message(&client, j_rqst);
}

/// Fonction de gestion d'un EsiPassMsg de type LoadAccount
///
/// Récupère un EsiPassMsg de type LoadAccount
/// La fonction va alors modifier les informations du compte local.
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type LoadAccount
pub fn handle_load_accnt(resp: EsiPassMsg) {
    // Récupération des infos du compte et du device
    let mut local_acc = get_account().expect("Erreur récupération du compte");

    // Récupération de la liste des DeviceInfo fournit par le serveur
    let mut devices: Vec<DeviceInfo> =
        serde_json::from_str(resp.msg.unwrap().as_str()).expect("Erreur désérialisation du msg ");
    let comms = update_local_devices_info(&mut devices, Some(local_acc.devices));
    local_acc.devices = comms;

    serialize_local_accnt(local_acc);
}

/// Fonction de gestion d'un EsiPassMsg de type Update Pwd File
///
/// Récupère un EsiPassMsg de type Update Pwd File
/// La fonction va alors déchiffrer avec la clé de transfert le mot de passe reçu
/// La fonction va ensuite écrire dans un fichier le mot de passe reçu
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type Update Pwd File
/// Fonction de gestion d'un EsiPassMsg de type Update Pwd File
///
/// Récupère un EsiPassMsg de type Update Pwd File
/// La fonction va alors déchiffrer avec la clé de transfert le mot de passe reçu
/// La fonction va ensuite écrire dans un fichier le mot de passe reçu
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type Update Pwd File
pub fn handle_update_pwd_file(resp: EsiPassMsg) {
    // Déchiffrement du fichier de mot de passe par une clé de transfert
    let string_esifile = decrypt_rqst(resp.msg.unwrap(), resp.accnt.user_id.unwrap()).unwrap();

    // Deserialisation du EsiPass du message
    let esifile: EsiFile = serde_json::from_str(string_esifile.as_str()).unwrap();

    // Ecriture du mot de passe
    write_esi_file(&esifile);

    // Affichage debug
    println!("PWD CHIFFRE RECUPERE : {:?}", esifile);
}

/// Fonction de gestion d'un EsiPassMsg de type RequestAllPwds
///
/// Récupère un EsiPassMsg de type RequestAllPwds
/// La fonction recoit une demande de tous les mots de passe du compte
/// Elle envoie chaque fichiers de mot de passe dans un PasswordUpdate
///
/// * `resp` : `EsiPassMsg` reçu du serveur et de type RequestAllPwds
pub fn handle_request_all_passwords(resp: EsiPassMsg) {
    // Création du client pour communiquer avec le serveur
    let client = build_reqwest_client().expect("Erreur création client");
    // Récupération des infos du compte et du device
    let local_acc = get_account().expect("Erreur récupération du compte");

    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("esipass/");

    let files = fs::read_dir(dir).unwrap();
    for file in files {
        let file_to_read = file.unwrap();

        if file_to_read.file_name() != ".gitignore" {
            // On ouvre le fichier
            let mut open_file =
                File::open(file_to_read.path()).expect("Erreur ouverture du fichier de MDP");
            // On lit le contenue du fichier de MDP
            let mut file_content = String::new();
            open_file
                .read_to_string(&mut file_content)
                .expect("Erreur de lecture dans le fichier EsiFile");
            //On crée le EsiFile a envoyer
            let esi_file: EsiFile = serde_json::from_str(file_content.as_str())
                .expect("Erreur deserialization du EsiFile");

            // Deuxième chiffrement du fichier de mot de passe par une clé de transfert
            let msg_content =
                serde_json::to_string(&esi_file).expect("Erreur serialisation du EsiFile");
            let msg = crypt_rqst(msg_content, local_acc.clone().account.user_id.unwrap()).unwrap();

            // Création de la requête pour le serveur
            let rqst = EsiPassMsg {
                rq_type: EsiPassMsgType::UpdatePwdFile,
                exp: get_local_device(&local_acc),
                dest: Some(resp.exp.clone().unwrap().device_id),
                accnt: local_acc.clone().account,
                signature: None,
                msg: Some(msg),
            };

            // Serialization de la requete sous la forme d'un String au format json
            let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

            // Envoi la requete au serveur
            send_message(&client, j_rqst);
        }
    }
}

/*
#################################################################
                        FONCTION D'ENVOI
#################################################################
*/

/// Fonction d'envoi d'un message via une requete HTTPS
///
/// Envoi le `String` `msg` dans une requete HTTPS par le `rqwest::blocking::Client` `client`.
/// Retourne un `Vec<EsiPassMsg>` comme réponse du serveur si tout se passe bien et une erreur sinon.
///
/// * `client` - Référence d'un `reqwest::blocking::Client` utilisé pour envoyer la requête HTTPS.
/// * `msg` - `String` du message à envoyer au serveur.
pub fn send_message(client: &reqwest::blocking::Client, msg: String) -> Vec<EsiPassMsg> {
    let resp = match client.post(format!("{}/send", SERV_URL)).json(&msg).send() {
        Ok(ok) => {
            let rep = ok.text().unwrap();
            println!("Réponse brute\n{:#?}\n", rep);

            let o: Vec<EsiPassMsg> = serde_json::from_str(rep.as_str()).expect("deser echec");

            println!("Objet désereliarisé\n{:#?}", o);
            o
        }

        Err(err) => {
            println!("ERR\n{:#?}", err);
            exit(0);
        }
    };

    resp
}

/// Fonction d'envoi d'un EsiPassMsg de type ChangeSessionKey
///
/// Retourne le Msg de la réponse qui est une `Option`.
pub fn send_chg_sess_key() -> Result<()> {
    // Création du client pour communiquer avec le serveur
    let client = build_reqwest_client().expect("Erreur création client");

    // Récupération des infos du compte et du device
    let local_acc = get_account().expect("Erreur récupération du compte");

    // Création de la requête pour le serveur
    let rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::ChangeSessionKey,
        exp: get_local_device(&local_acc),
        dest: None,
        accnt: local_acc.account.clone(),
        signature: Some(0),
        msg: None,
    };

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur et récupération des potentiels réponses
    let resp = send_message(&client, j_rqst);

    if resp.is_empty() {
        Ok(())
    } else {
        Err(anyhow!("Refus renouvellement de la session key"))
    }
}

pub fn send_challenge(
    client: &reqwest::blocking::Client,
    new_accnt: Account,
    dev_info: DeviceInfo,
) -> Result<()> {
    let mut rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::Challenge,
        accnt: new_accnt,
        signature: Some(0),
        exp: Some(dev_info),
        msg: None,
        dest: None,
    };

    // Serialization de la requete sous la forme d'un String au format json
    let mut j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    let _code = match client
        .post(format!("{}/send", SERV_URL))
        .json(&j_rqst)
        .send()
    {
        Ok(ok) => {
            let rep = ok.text().unwrap();

            println!("Réponse : {:#?}", rep);
            rep
        }

        Err(err) => {
            println!("ERR\n{:#?}", err);
            exit(0);
        }
    };

    rqst.msg = Some(String::from("5000"));

    j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    match client
        .post(format!("{}/send", SERV_URL))
        .json(&j_rqst)
        .send()
    {
        Ok(ok) => {
            let rep = ok.text().unwrap();

            println!("Réponse : {:#?}", rep);
            rep
        }

        Err(err) => {
            println!("ERR\n{:#?}", err);
            exit(0);
        }
    };

    Ok(())
}

/// Fonction d'envoi d'une requête d'initialisation de DHE.
///
/// Envoi le `DeviceComm` `comm` dans une requete 'EsiPassMsg', dans le champ msg.
/// Ce 'DeviceComm' correspond au 'DeviceInfo' de l'appareil actual au préalable complété par une clé publique DHE.
/// Aucune réponse particulière du serveur n'est attendue.
///
/// * `comm` - `DeviceComm` à envoyer au serveur.
pub fn send_dhe(comm: DeviceComm) {
    // Création du client pour communiquer avec le serveur
    let client = build_reqwest_client().expect("Erreur création client");

    // Récupération des infos du compte et du device
    let local_acc = get_account().expect("Erreur récupération du compte");

    // Création de la requête pour le serveur
    let rqst = EsiPassMsg {
        rq_type: EsiPassMsgType::RequestDHE,
        exp: get_local_device(&local_acc),
        dest: None,
        accnt: local_acc.account.clone(),
        signature: Some(0),
        msg: Some(serde_json::to_string(&comm).expect("Erreur serialisation DeviceComm")),
    };

    // Serialization de la requete sous la forme d'un String au format json
    let j_rqst = serde_json::to_string(&rqst).expect("Erreur serialisation de la requete");

    // Envoi la requete au serveur et récupération des potentiels réponses
    send_message(&client, j_rqst);
}

/*
#################################################################
                        FONCTION UTILITAIRES
#################################################################
*/

/// Initialisation d'un `reqwest::blocking::Client` faisant des requetes HTTPS.
///
/// Utilise le certificat auto-signé `cert.pem` pour faire les requetes en HTTPS.
pub fn build_reqwest_client() -> Result<Client> {
    // Récupération du certificat
    let mut buf = Vec::new();

    let mut pem_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    pem_path.push(PEM);

    File::open(pem_path)?.read_to_end(&mut buf)?;

    let _cert = reqwest::Certificate::from_pem(&buf)?;

    // Création du client
    Ok(reqwest::blocking::Client::builder()
        .danger_accept_invalid_certs(true)
        //.add_root_certificate(_cert)
        .build()?)
}
