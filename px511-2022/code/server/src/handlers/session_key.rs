use actix_web::HttpResponse;
use anyhow::Ok;
use anyhow::Result;
use utils::data::comm_struct::EsiPassMsg;

use crate::handlers::create_server_resp;
use crate::models::UserController;
/// Indique que le device a changer la clé de session
/// Si le device faisant le changement n'a pas actuellment la cle de session a
/// jour, lui notifie que le changement a deja recemment ete fait
/// Sinon, place dans les inbox de tous les autres devices du compte une
/// notification leur indiquant le changement
pub fn change_session_key(rq_data: EsiPassMsg) -> Result<HttpResponse> {
    // Vérification que la requête est correct.
    let mut user_controller: UserController = verify_change_session_key(&rq_data)?;

    log::info!(
        "Chnage session key received from : acc_id={}, dev_id={}",
        rq_data.accnt.user_id.unwrap(),
        rq_data.exp.as_ref().unwrap().device_id
    );

    let exp_id = rq_data.exp.clone().unwrap().device_id;

    // Si le demandeur du changement na pas sa cle de session a jour (un autre device a fait la demande avant) onlui repond immediatement
    // le client regarde si la reponse contient quelque chose et si cest le cas comprend que sa semande a été refusé
    if !user_controller
        .devices_manager
        .clone()
        .find_device(exp_id)
        .unwrap()
        .session_key_up_to_date
    {
        log::info!("Rejet changement de cle de session : device pas à jour");
        return Ok(create_server_resp(vec![rq_data]));
    }

    let devices = &mut user_controller.devices_manager.devices;
    // On indique que les pwd en attente dans le serveur sont avec lancienne cle de session
    for mut pwd_update in &mut user_controller.devices_manager.pwd_manager.pwd_file_updates {
        pwd_update.actual_session_key = false;
    }

    // Pour chaque device du compte
    for mut device in devices {
        // Sauf celui ayant fait le changement
        if device.dev_info.device_id != exp_id {
            // On place dans l'inbox le message leur indiquant qu'il ya eu un changement
            device.inbox.push(rq_data.clone());
            // On indique que leur clé de session n'est plus à jour
            device.session_key_up_to_date = false;
        };
    }
    user_controller.save();
    log::info!("Placed in all inboxs");
    Ok(create_server_resp(vec![]))
}

// Vérification que la requête contient un ID de compte, un message, fait référence à un ID d'appareil existant.
fn verify_change_session_key(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
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
