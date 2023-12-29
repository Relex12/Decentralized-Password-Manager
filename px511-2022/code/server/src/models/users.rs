use std::fs::File;
use std::io::Write;

use crate::models::AccountManager;
use crate::models::DevicesManager;
use crate::models::PwdFileManager;
use actix_web::HttpResponse;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use utils::data::comm_struct::Account;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserController {
    pub accnt: Account,
    pub devices_manager: DevicesManager,
}

impl UserController {
    #[allow(unused)]
    pub fn create_user_from_acc(accnt: Account) -> UserController {
        UserController {
            accnt,
            devices_manager: DevicesManager {
                pwd_manager: PwdFileManager {
                    last_file_version: 0,
                    pwd_file_updates: Vec::new(),
                },
                devices: Vec::new(),
                global_inbox: Vec::new(),
            },
        }
    }

    /// Charge les informations d'utilisateur en fonction de son ID
    pub fn get_user_from_id(id: u64) -> Result<UserController> {
        let path = AccountManager::generate_acc_path(id);

        let us_str = std::fs::read_to_string(path)?;
        let us = serde_json::from_str::<UserController>(&us_str)?;

        Ok(us)
    }

    /// Sauvegarde les données d'un utilisateur dans le dossier src/data
    pub fn save(&self) -> Option<HttpResponse> {
        match self._serialize_user() {
            Ok(_) => None,
            Err(err) => Some(HttpResponse::InternalServerError().json(err.to_string())),
        }
    }

    /// Réalise la fonction de sérialisation et retourne un result
    fn _serialize_user(&self) -> anyhow::Result<()> {
        if let Some(id) = self.accnt.user_id {
            let path = AccountManager::generate_acc_path(id);

            let mut file = File::create(path)?;

            //On sauvegarde la structure du challenge du client en question.
            let json = serde_json::to_string(&self)?;
            file.write_all(json.as_bytes())?;
        }

        Ok(())
    }
}
