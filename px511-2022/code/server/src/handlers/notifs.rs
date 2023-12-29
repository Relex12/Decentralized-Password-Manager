use actix_web::HttpResponse;
use anyhow::Ok;
use anyhow::Result;
use utils::data::comm_struct::*;
use utils::data::debug_struct::DebugBuffer;

use crate::handlers::create_server_resp;
use crate::models::UserController;

/// Demande de récupérations des notifs sur le serveur :
/// L'inbox du device pouvant contenir des reponses de DHE
/// L'inbox globale pouvant contenir des demandes de DHE mais uniquement pull
/// si le device réalisant le pull a la cle de session a jour
/// Les mises a jour de mot de passes manquant au device
pub fn pull_notifs(rq_data: EsiPassMsg) -> Result<HttpResponse> {
    // Vérification que la requête est correct.
    let mut user_controller: UserController = verify_pull_notifs(&rq_data)?;

    log::info!(
        "Pul notifs received from : acc_id={}, dev_id={}",
        rq_data.accnt.user_id.unwrap(),
        rq_data.exp.as_ref().unwrap().device_id
    );

    // Création de message de debug.
    let mut debug_msg_in = String::new();
    DebugBuffer::add_msg(&mut debug_msg_in, &rq_data.rq_type.to_string());
    let mut debug_msg_out = String::new();

    DebugBuffer::add_msg(
        &mut debug_msg_out,
        format!(
            "Pull notifs recu depuis : acc_id={}, dev_id={}",
            rq_data.accnt.user_id.unwrap(),
            rq_data.exp.clone().unwrap().device_id
        )
        .as_str(),
    );

    let exp_id = rq_data.exp.as_ref().unwrap().device_id;

    let mut uclone = user_controller.clone();
    let exp = uclone.devices_manager.find_device(exp_id).unwrap();

    let mut vec_msg = Vec::<EsiPassMsg>::new();
    let devmanager = &mut user_controller.devices_manager;
    let pwd_manager = &mut devmanager.pwd_manager.clone();

    //PLace les messages de l'inbox perso dans le vec de reponse
    log::info!("send personnal inbox");
    vec_msg.append(&mut devmanager.find_device(exp_id).unwrap().inbox);

    // Place les messages de l'inbox globale dans le vec reponse si cle de sesion du device a jour
    if exp.session_key_up_to_date {
        log::info!("send global inbox");
        vec_msg.append(&mut devmanager.global_inbox);
    }

    // Prend differnce entre la derniere mise a jour sur l'appareil et celle disponible
    let mut pwd_diff = pwd_manager.last_file_version as usize - exp.file_version as usize;
    // Recupere le nombre de pwdUpdate dans le serveur
    let len = pwd_manager.pwd_file_updates.len();
    match len as i64 - pwd_diff as i64 {
        x if x >= 0 => {
            // Si le device a pas de pwd irrécupérable (flush par server ou nouveau device)

            // Si il n'a pas sa clé de session à jour
            if !exp.session_key_up_to_date {
                log::info!("send only missing pwds with old SessionKey");
                let mut sended = 0;
                // on envoies ceux qui manque au device mais uniquement ceux avec lancienne cle de session
                while !pwd_manager.pwd_file_updates[len - pwd_diff].actual_session_key {
                    vec_msg.push(EsiPassMsg {
                        rq_type: utils::data::comm_struct::EsiPassMsgType::UpdatePwdFile,
                        exp: None,
                        dest: None,
                        accnt: rq_data.accnt.clone(),
                        signature: None,
                        msg: Some(
                            pwd_manager.pwd_file_updates[len - pwd_diff]
                                .pwd_file
                                .clone(),
                        ),
                    });
                    pwd_diff -= 1;
                    sended += 1;
                }
                // on update le numero de version du device
                devmanager.find_device(exp_id).unwrap().file_version += sended;
            } else {
                log::info!("send all missing pwds");
                //sinon on envoi tout ceux qui manque au device
                while pwd_diff > 0 {
                    vec_msg.push(EsiPassMsg {
                        rq_type: utils::data::comm_struct::EsiPassMsgType::UpdatePwdFile,
                        exp: None,
                        dest: None,
                        accnt: rq_data.accnt.clone(),
                        signature: None,
                        msg: Some(
                            pwd_manager.pwd_file_updates[len - pwd_diff]
                                .pwd_file
                                .clone(),
                        ),
                    });
                    pwd_diff -= 1;
                }
                // on update le numero de version du device
                devmanager.find_device(exp_id).unwrap().file_version =
                    pwd_manager.last_file_version;
            }
        }
        // Si le device a des pwds irrécupérable (flush par server ou nouveau device)
        x if x < 0 => {
            // S'il y a des pwd irrécupérable et qu'il n'a la cle de session a jour on lui fait demander tous les pwd
            if exp.session_key_up_to_date {
                log::info!("RequestAllPwds add in global inbox");
                devmanager.global_inbox.push(EsiPassMsg {
                    rq_type: EsiPassMsgType::RequestAllPwds,
                    exp: Some(exp.clone().dev_info),
                    dest: None,
                    accnt: rq_data.accnt.clone(),
                    signature: None,
                    msg: None,
                });
                // on update le numero de version du device
                devmanager.find_device(exp_id).unwrap().file_version =
                    pwd_manager.last_file_version;
            }
        }
        _ => {}
    }

    // On récupre le numero de mise a jour du device le moins a jour
    let mut smaller_device_update_number = pwd_manager.last_file_version;
    for device in &devmanager.devices {
        if device.file_version < smaller_device_update_number {
            smaller_device_update_number = device.file_version;
        }
    }
    if devmanager.find_device(exp_id).unwrap().file_version == smaller_device_update_number {
        // On retire de la liste de mise a jour des pwd ceux qui ont été recu par tout les devices
        let not_all_distributed =
            (pwd_manager.last_file_version - smaller_device_update_number) as usize;
        // de 0 à la différence entre le nombre de maj sur le serveur et le nombre quil faut en garder
        devmanager
            .pwd_manager
            .pwd_file_updates
            .drain(..(pwd_manager.pwd_file_updates.len() - not_all_distributed));
    }
    user_controller.save();

    DebugBuffer::to_debug(rq_data.clone(), rq_data, debug_msg_in, debug_msg_out);

    Ok(create_server_resp(vec_msg))
}

// Vérification que la requête contient un ID de compte, un message, fait référence à un ID d'appareil existant.
fn verify_pull_notifs(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
    // La requête doit posséder un ID de compte.
    if rq_data.accnt.user_id.is_none() {
        anyhow::bail!("User ID missing")
    }

    // La requête doit posséder un expéditeur.
    if rq_data.exp.is_none() {
        anyhow::bail!("expediteur missing")
    }

    // La requête doit faire référence à un compte existant.
    Ok(UserController::get_user_from_id(
        rq_data.accnt.user_id.unwrap(),
    )?)
}
