use crate::handlers::{challenge_handler, request_dhe, update_pwd_file};
use actix_web::{HttpResponse, Responder};

use utils::data::comm_struct::*;

use super::{change_session_key, load_account, pull_notifs, response_dhe};

/// Lorsque le serveur reçoit une requête elle est parsée afin de connaître sa demande.
pub async fn parse_request(message: String) -> impl Responder {
    // Vérification que la requête est correcte.
    let mut reqwest_struct = match verify_request(message) {
        Ok(req) => req,

        Err(err) => {
            return HttpResponse::InternalServerError().json(err.to_string());
        }
    };

    // Redirection du traitement de la requête en fonction de sa spécialité.
    let resp = match reqwest_struct.rq_type {
        // Mise à jour de la clé de session de tous les devices
        EsiPassMsgType::ChangeSessionKey => change_session_key(reqwest_struct),

        // Ajout d'une modification de fichier de mots de passe
        EsiPassMsgType::UpdatePwdFile => update_pwd_file(reqwest_struct),

        // Vérifie si une opération est en attente pour un device
        EsiPassMsgType::PullNotif => pull_notifs(reqwest_struct),

        // Demande de DHE par un device
        EsiPassMsgType::RequestDHE => request_dhe(reqwest_struct),

        EsiPassMsgType::Challenge => challenge_handler(&reqwest_struct),

        // Opération sur le compte (chargement, création, ajout d'appareil...)
        EsiPassMsgType::LoadAccount => load_account(&mut reqwest_struct),

        // Reponse a une demande de DHE
        EsiPassMsgType::RespDHE => response_dhe(reqwest_struct),

        // Pas envoyé par les client
        EsiPassMsgType::RequestAllPwds => Ok(HttpResponse::NotAcceptable().finish()),
    };

    match resp {
        Ok(response) => response,
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

pub fn verify_request(req: String) -> anyhow::Result<EsiPassMsg> {
    // Transformation de la requête en une structure ordonnée pour la suite du programme.
    let unescaped_txt = snailquote::unescape(req.as_str())?;
    let reqwest_struct: EsiPassMsg = serde_json::from_str(&unescaped_txt)?;

    // La requête doit forcément posséder une clé publique (inclue dans exp).
    if reqwest_struct.exp.is_some() {
        Ok(reqwest_struct)
    } else {
        anyhow::bail!("Invalid public key")
    }
}
