use actix_web::HttpResponse;
use anyhow::Error;
use anyhow::Ok;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use utils::data::debug_struct::DebugBuffer;

use utils::data::comm_struct::*;

use super::create_server_resp;
use crate::models::AccountManager;
use crate::models::Device;
use crate::models::UserController;

#[derive(Serialize, Deserialize)]
pub struct CreateAccountInfo {
    public_key: Option<String>,
    code: Option<u64>,
}

/// Création d'un appareil
/// Si la requête ne contient pas d'ID de compte alors créer aussi un compte.
pub fn load_account(rq_data: &mut EsiPassMsg) -> anyhow::Result<HttpResponse> {
    // Vérification de la conformité de la requête
    let exp = rq_data
        .exp
        .clone()
        .ok_or_else(|| Error::msg("Device id missing"))?;

    log::info!(
        "Load account received for user: Nom={}, Prenom={}, Tel={}",
        rq_data.accnt.name,
        rq_data.accnt.first_name,
        rq_data.accnt.phone
    );

    let mut _user: Option<UserController> = None;
    // Création de message de debug.
    let mut debug_msg_in = String::new();
    DebugBuffer::add_msg(&mut debug_msg_in, &rq_data.rq_type.to_string());
    let mut debug_msg_out = String::new();

    // Le serveur essai de load le compte
    let user = match search_account(rq_data, &mut debug_msg_out) {
        // S'il exsite alors le controller est récupéré avec la création d'un appareil.
        anyhow::Result::Ok(mut user) => {
            log::info!("Account found");

            // Le compte est complété avec la création de l'appareil.
            let new_dev_id = user.devices_manager.create_device(rq_data)?;
            let send_user = user.devices_manager.get_devs_info();
            // Pour chaque device du compte
            for device in &mut user.devices_manager.devices {
                // Sauf celui ayant été créé
                if device.dev_info.device_id != new_dev_id {
                    // On place dans l'inbox le message leur indiquant qu'un nouveau compte à été créé

                    device.inbox.push(EsiPassMsg {
                        rq_type: EsiPassMsgType::LoadAccount,
                        msg: Some(serde_json::to_string(&send_user)?),
                        exp: None,
                        dest: None,
                        accnt: user.accnt.clone(),
                        signature: None,
                    });
                };
            }

            user
        }

        // DAns le cas échéant, le comtpe est créé
        Err(_) => {
            log::info!("Account created");
            let mut user = AccountManager::create_account(rq_data);
            user.devices_manager.devices.push(Device::create_device(
                exp.clone().dev_name,
                1,
                exp.public_key,
                true,
            ));
            DebugBuffer::add_msg(
                &mut debug_msg_out,
                format!(
                    "Création de compte réalisée : nom={};prénom={};tel={}",
                    user.accnt.name, user.accnt.first_name, user.accnt.phone
                )
                .as_str(),
            );
            user
        }
    };

    // Sauvegarde du compte à jour.
    user.save();
    DebugBuffer::add_msg(
        &mut debug_msg_out,
        &format!(
            "Création d'un nouvel appareil réalisée d'ID : {}",
            user.devices_manager
                .devices
                .last()
                .unwrap()
                .dev_info
                .device_id
        ),
    );

    // Préparation et envoi de la réponse.
    let resp = EsiPassMsg {
        rq_type: EsiPassMsgType::LoadAccount,
        msg: Some(serde_json::to_string(
            &user.devices_manager.get_devs_info(),
        )?),
        exp: Some(exp),
        dest: None,
        accnt: user.accnt,
        signature: None,
    };

    DebugBuffer::to_debug(
        rq_data.to_owned(),
        resp.clone(),
        debug_msg_in,
        debug_msg_out,
    );
    log::info!("New device created");
    Ok(create_server_resp(vec![resp]))
}

pub fn search_account(
    rq_data: &mut EsiPassMsg,
    debug_msg_out: &mut String,
) -> anyhow::Result<UserController> {
    // Chemin absolu vers les comptes utilisateurs.
    let mut accnts_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    accnts_path.push("src/data/");

    // Parcour de chaque utilisateur
    let paths = fs::read_dir(accnts_path).unwrap();
    for path in paths {
        // Récupération de l'ID d'un utilisateur.
        let file_name = path?.file_name().into_string().unwrap();
        log::info!("{}", file_name);
        if file_name != "debug" && file_name != "challenge" && file_name != ".gitignore" {
            let id = file_name.parse::<u64>()?;
            let us = UserController::get_user_from_id(id)?;

            // Si l'utilisateur est le bon, ses informations sont retournées.
            if us.accnt.first_name == rq_data.accnt.first_name
                && us.accnt.name == rq_data.accnt.name
                && us.accnt.phone == rq_data.accnt.phone
            {
                DebugBuffer::add_msg(
                    debug_msg_out,
                    &format!("Compte trouvé:id={}", us.accnt.user_id.unwrap()),
                );

                return Ok(us);
            }
        }
    }

    anyhow::bail!("No account found")
}

/// Gestion du challenge dupuis son début jusqu'à la fin
pub fn challenge_handler(rq_data: &EsiPassMsg) -> Result<HttpResponse> {
    // Création de message de debug.
    let mut debug_msg_in = String::new();
    DebugBuffer::add_msg(&mut debug_msg_in, &rq_data.rq_type.to_string());
    let mut debug_msg_out = String::new();

    // Traitement du challenge avant la création du compte.
    match &rq_data.msg {
        // Si la requête ne contient pas de code alors nous somme au début du challenge.
        None => {
            // Création et sauvegarde du code.
            let code = _begin_challenge(&rq_data.accnt)?;
            let resp = EsiPassMsg {
                rq_type: EsiPassMsgType::Challenge,
                msg: Some(code.to_string()),
                exp: rq_data.exp.to_owned(),
                dest: None,
                accnt: rq_data.accnt.to_owned(),
                signature: None,
            };

            DebugBuffer::to_debug(
                rq_data.to_owned(),
                resp.clone(),
                debug_msg_in,
                debug_msg_out,
            );
            Ok(create_server_resp(vec![resp]))
        }

        // Sinon nous vérifions le code.
        Some(code) => {
            let res = _verify_challenge(
                &rq_data.accnt,
                code.parse().expect("Erreur lecture code challenge"),
            )?;
            if !res {
                DebugBuffer::add_msg(&mut debug_msg_out, "Vérification du code erroné");
                Ok(HttpResponse::Unauthorized().finish())
            } else {
                DebugBuffer::add_msg(&mut debug_msg_out, "Vérification ud code correct");

                let resp = EsiPassMsg {
                    rq_type: EsiPassMsgType::Challenge,
                    msg: Some("true".to_string()),
                    exp: rq_data.exp.to_owned(),
                    dest: None,
                    accnt: rq_data.accnt.to_owned(),
                    signature: None,
                };

                DebugBuffer::to_debug(
                    rq_data.to_owned(),
                    resp.clone(),
                    debug_msg_in,
                    debug_msg_out,
                );
                Ok(create_server_resp(vec![resp]))
            }
        }
    }
}

/// Génération et sauvegarde local du code du challenge.
fn _begin_challenge(data_struct: &Account) -> anyhow::Result<u64> {
    //Génération du code, ici choisi statiquement pour simuler le fonctionnement du challenge.
    let chall_code = 5000;

    //Création d'un espace de stockage pour mémoriser le numéro du challenge.
    let client_chall_path = _generate_path(&format!(
        "{}{}{}",
        data_struct.first_name, data_struct.name, data_struct.phone
    ));
    let mut file = File::create(client_chall_path)?;

    //On sauvegarde la structure du challenge du client en question.
    let json = serde_json::to_string(&chall_code)?;
    file.write_all(json.as_bytes())?;
    Ok(chall_code)
}

/// Vérification du code du challenge
fn _verify_challenge(data_struct: &Account, code: u64) -> anyhow::Result<bool> {
    //Lecture du fichier du client
    let client_chall_path = _generate_path(&format!(
        "{}{}{}",
        data_struct.first_name, data_struct.name, data_struct.phone
    ));
    let text = fs::read_to_string(client_chall_path.clone())?;
    let chall_info = serde_json::from_str::<u64>(&text)?;
    std::fs::remove_file(client_chall_path)?;
    Ok(chall_info == code)
}

///Retourne le chemin vers le fichier associé à la clé publique.
fn _generate_path(pk: &String) -> String {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/data/challenge/");
    format!("{}{}", path.to_str().unwrap(), pk)
}

/*

#[cfg(test)]
mod tests {
    use client::account_operation::init_account;
    use client::dhe::gen_dhe_keys;
    use client::rq_operations::build_reqwest_client;
    use std::process::exit;
    use std::str::FromStr;
    use utils::data::comm_struct::{Account, DeviceInfo, EsiPassResp, EsiPassRqst, RqstType};

    #[test]
    /// sumalation challenge client-serveur
    fn create_account() {
        let new_accnt = init_account(0789799307);

        // Génération des clés cryptographique du compte
        let key = gen_dhe_keys().public_key;

        // Création de la requête pour le serveur
        let rqst = EsiPassRqst {
            rq_type: RqstType::CreateDevice,
            accnt: new_accnt,
            signature: 0,
            device_info: DeviceInfo {
                device_id: 0,
                dev_name: String::from_str("testhhdevice").unwrap(),
                public_key: key,
            },
            msg: None,
            to: None,
        };

        // Création du Client reqwest pour l'envoi de paquet
        let client = match build_reqwest_client() {
            Ok(client) => client,
            Err(_) => {
                println!("Erreur création client");
                exit(0);
            }
        };

        // Serialization de la requete sous la forme d'un String au format json
        let j_rqst = match serde_json::to_string(&rqst) {
            Ok(j) => j,
            Err(_) => {
                println!("Erreur serialisation de la requete");
                exit(0);
            }
        };

        // Envoi la requete au serveur
        let response = client
            .post(format!("{}/send", "https://127.0.0.1:40443"))
            .json(&j_rqst)
            .send()
            .unwrap();
        assert!(response.status().is_success());
        println!("{:#?}", response.json::<EsiPassResp>().expect("fail"));
    }
    #[test]
    fn create_device() {
        let new_accnt = init_account(0789799307);

        // Génération des clés cryptographique du compte
        let key = gen_dhe_keys().public_key;

        // Création de la requête pour le serveur
        let rqst = EsiPassRqst {
            rq_type: RqstType::CreateDevice,
            accnt: new_accnt,
            signature: 0,
            device_info: DeviceInfo {
                device_id: 0,
                dev_name: String::from_str("testhhdevice").unwrap(),
                public_key: key,
            },
            msg: None,
            to: None,
        };

        // Création du Client reqwest pour l'envoi de paquet
        let client = match build_reqwest_client() {
            Ok(client) => client,
            Err(_) => {
                println!("Erreur création client");
                exit(0);
            }
        };

        // Serialization de la requete sous la forme d'un String au format json
        let j_rqst = match serde_json::to_string(&rqst) {
            Ok(j) => j,
            Err(_) => {
                println!("Erreur serialisation de la requete");
                exit(0);
            }
        };

        // Envoi la requete au serveur
        let response: EsiPassResp = client
            .post(format!("{}/send", "https://127.0.0.1:40443"))
            .json(&j_rqst)
            .send()
            .unwrap()
            .json()
            .unwrap();

        let new_rqst = EsiPassRqst {
            rq_type: RqstType::CreateDevice,
            accnt: Account {
                user_id: Some(response.accnt_id),
                phone: 0783799307,
                name: r,
                first_name: todo!(),
            },
            signature: 0,
            device_info: DeviceInfo {
                device_id: 0,
                dev_name: String::from_str("testhhdevice").unwrap(),
                public_key: key,
            },
            msg: None,
            to: None,
        };
        let new_j_rqst = match serde_json::to_string(&new_rqst) {
            Ok(j) => j,
            Err(_) => {
                println!("Erreur serialisation de la requete");
                exit(0);
            }
        };
        let response = client
            .post(format!("{}/send", "https://127.0.0.1:40443"))
            .json(&new_j_rqst)
            .send()
            .unwrap();
        assert!(response.status().is_success());
    }
}
*/
