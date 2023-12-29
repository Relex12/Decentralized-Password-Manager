use actix_web::HttpResponse;
use anyhow::Error;
use anyhow::Ok;
use anyhow::Result;
use utils::data::{
    comm_struct::{EsiPassMsg, EsiPassMsgType},
    debug_struct::DebugBuffer,
};

use crate::{handlers::create_server_resp, models::UserController};

/// Push la modification de mots de passe et met à jours le numéro de version le plus récent du compte.
pub fn update_pwd_file(rq_data: EsiPassMsg) -> Result<HttpResponse> {
    // Vérification que la requête est correct.
    let mut user_controller = verify_pwd_update(&rq_data)?;

    // S'il n'y a aps de destinataire
    if rq_data.dest.is_none() {
        log::info!(
            "Update pwdFile received from : acc_id={}, dev_id={}",
            rq_data.accnt.user_id.unwrap(),
            rq_data.exp.as_ref().unwrap().device_id
        );

        let exp = rq_data
            .exp
            .clone()
            .ok_or_else(|| Error::msg("Device id missing"))?;
        let device_id = exp.device_id;

        // Création des messages de debug.
        let mut debug_msg_in = String::new();
        DebugBuffer::add_msg(
            &mut debug_msg_in,
            "Ajout d'une modification de mot de passe",
        );
        let mut debug_msg_out = String::new();

        // Ajout de la modification de mots de passe.
        // Le unwrap n'est pas risqué ici car la méthode verify_pwd_update assure que l'option ne contient pas None.
        user_controller
            .devices_manager
            .pwd_manager
            .add_file_update(rq_data.msg.clone().unwrap());

        // Incrémentation de la version du fichier de mots de passe de l'appareil.
        let new_ver = user_controller
            .devices_manager
            .update_device_version(device_id)?;

        // Mis à jour de la version du fichier de mots de passe la plus récente si nécessaire.
        user_controller
            .devices_manager
            .pwd_manager
            .check_ver_update(new_ver);

        // Préparation et envoi de la réponse.
        let resp = EsiPassMsg {
            rq_type: EsiPassMsgType::UpdatePwdFile,
            msg: None,
            exp: Some(exp),
            dest: None,
            accnt: user_controller.clone().accnt,
            signature: None,
        };

        DebugBuffer::add_msg(
            &mut debug_msg_out,
            format!("Nouvelle version du fichier de mot de passe : {}", new_ver).as_str(),
        );
        DebugBuffer::to_debug(rq_data, resp.clone(), debug_msg_in, debug_msg_out);

        // Sauvagarde des changements
        user_controller.save();

        log::info!("Placed in password manager");

        Ok(create_server_resp(vec![resp]))
    }
    //Sinon met dans l'inbox du destiantaire (pour la recuperation des mots de passe lors déune desynchro)
    else {
        user_controller
            .devices_manager
            .find_device(rq_data.dest.unwrap())
            .unwrap()
            .inbox
            .push(rq_data.clone());
        // Sauvagarde des changements
        user_controller.save();

        log::info!(
            "Placed in inbox dev_id = {:?}",
            user_controller
                .devices_manager
                .find_device(rq_data.dest.unwrap())
        );
        Ok(create_server_resp(vec![]))
    }
}

// Vérification que la requête contient un ID de compte, un message, fait référence à un ID d'appareil existant.
fn verify_pwd_update(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
    // La requête doit posséder un ID de compte.
    if rq_data.accnt.user_id.is_none() {
        anyhow::bail!("User ID missing")
    }

    // La requête doit posséder une mise à jour de fichier de mots de passe.
    if rq_data.msg.is_none() {
        anyhow::bail!("No password update in request")
    }

    // La requête doit faire référence à un compte existant.
    Ok(UserController::get_user_from_id(
        rq_data.accnt.user_id.unwrap(),
    )?)
}
