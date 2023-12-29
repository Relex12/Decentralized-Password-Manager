use actix_web::HttpResponse;
use anyhow::Ok;
use anyhow::Result;
use utils::data::comm_struct::EsiPassMsg;

use crate::handlers::create_server_resp;
use crate::models::UserController;

/// Demande de dhe pour pouvoir recevoir la cle de session
/// Contient la cle publique généré pour ce dhe
/// La demande est place dans l'inbox globale commune aux devices du compte
/// Le premier device ayant la cle de session à jour réalisant un pull notifs
/// recevra cette demande de dhe et y repondra
pub fn request_dhe(rq_data: EsiPassMsg) -> Result<HttpResponse> {
    // Vérification que la requête est correct.
    let mut user_controller: UserController = verify_request_dhe(&rq_data)?;

    log::info!(
        "Request DHE received from : acc_id={}, dev_id={}",
        rq_data.accnt.user_id.unwrap(),
        rq_data.exp.as_ref().unwrap().device_id
    );

    // Place les infos de la requete de DHE dans l'inbox globale
    user_controller.devices_manager.global_inbox.push(rq_data);

    user_controller.save();

    log::info!("Placed in global inbox");
    Ok(create_server_resp(vec![]))
}

fn verify_request_dhe(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
    // La requête doit posséder un ID de compte.
    if rq_data.accnt.user_id.is_none() {
        anyhow::bail!("User ID missing")
    }

    // La requête doit posséder un message contenant les informations pour le dhe.
    if rq_data.msg.is_none() {
        anyhow::bail!("No dhe info in request")
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
/// Reponse a une demande de dhe
/// Elle contient la cle publique genere pour ce dhe du device repondant
/// Elle est place dans l'inbox du device qui a réalisée la demande
pub fn response_dhe(rq_data: EsiPassMsg) -> Result<HttpResponse> {
    log::info!(
        "Response DHE received from : acc_id={}, dev_id={}",
        rq_data.accnt.user_id.unwrap(),
        rq_data.exp.as_ref().unwrap().device_id
    );

    // Vérification que la requête est correct.
    let mut user_controller: UserController = verify_response_dhe(&rq_data)?;
    let dest_id = rq_data.dest.unwrap();
    let mut dest = user_controller
        .devices_manager
        .find_device(dest_id)
        .unwrap();

    // Place les infos de la reponse de DHE dans l'inbox du destiantaire
    dest.inbox.push(rq_data);

    // La cle de session du device est maintenant a jour
    dest.session_key_up_to_date = true;

    user_controller.save();
    log::info!("Placed in dev_id={} inbox", dest_id);
    Ok(create_server_resp(vec![]))
}

fn verify_response_dhe(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
    // La requête doit posséder un ID de compte.
    if rq_data.accnt.user_id.is_none() {
        anyhow::bail!("User ID missing")
    }

    // La requête doit posséder un message contenant les informations pour le dhe.
    if rq_data.msg.is_none() {
        anyhow::bail!("No dhe info and SKey in response")
    }

    // La requête doit posséder desinataire.
    if rq_data.dest.is_none() {
        anyhow::bail!("destinataire missing")
    }

    // La requête doit faire référence à un compte existant.
    Ok(UserController::get_user_from_id(
        rq_data.accnt.user_id.unwrap(),
    )?)
}
