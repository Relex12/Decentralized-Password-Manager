use std::fs::File;

use serde::{Deserialize, Serialize};
use utils::data::comm_struct::{Account, EsiPassMsg};

use super::UserController;

#[derive(Serialize, Deserialize)]
#[allow(unused)]
#[derive(Clone, Debug)]

/// Structure permettant de gérer les informations personnelles liées à un compte.
pub struct AccountManager {
    /// Numéro de téléphone
    pub tel_num: Option<String>,
    /// Identifiant de compte
    pub user_id: Option<u128>,
}

impl AccountManager {
    /// Creation d'un compte à partir d'une requête.
    /// # Arguments
    /// * 'rq_data' - Un objet EsiPassRqst qui contient les données envoyées par le client.
    ///
    /// Retourne le controller de l'utilisateur.
    pub fn create_account(rq_data: &EsiPassMsg) -> UserController {
        log::info!("Création de compte détectée");

        let id = AccountManager::_create_acc_id();
        let acc = Account {
            user_id: Some(id),
            phone: rq_data.accnt.phone,
            name: rq_data.accnt.name.clone(),
            first_name: rq_data.accnt.first_name.clone(),
        };

        UserController::create_user_from_acc(acc)
    }

    /// Génération d'un ID unique de 16 caractères de long.
    /// Retourne l'ID du compte.
    fn _create_acc_id() -> u64 {
        let mut rng = rand::thread_rng();
        let mut new_id: u64 = rand::Rng::gen_range(&mut rng, 100_000_000_000..999_999_999_999);

        let mut generated = false;

        while !generated {
            // Génération d'un ID, recherche d'un ID unique.
            new_id = match File::open(AccountManager::generate_acc_path(new_id)) {
                Err(_) => {
                    generated = !generated;
                    new_id
                }

                // Si un fichier est trouver un nouvel id est généré.
                Ok(_) => rand::Rng::gen_range(&mut rng, 100_000_000_000..999_999_999_999),
            };
        }
        new_id
    }

    /// Retourne le chemin vers le fichier associé à la clé publique.
    /// # Arguments
    /// * 'id' - numéro d'ID du compte de l'utilisateur.
    ///
    pub fn generate_acc_path(id: u64) -> String {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("src/data/");
        format!("{}{}", path.to_str().unwrap(), id)
    }

    /// Récupération d'un compte existant depuis son ID
    /// # Arguments
    /// * 'rq_data' - Un objet EsiPassRqst qui contient les données envoyées par le client.
    ///
    /// Retourne le controller de l'utilisateur.
    #[allow(unused)]
    pub fn get_account(rq_data: &EsiPassMsg) -> anyhow::Result<UserController> {
        log::info!("Récupération de compte détectée");

        match rq_data.accnt.user_id {
            Some(id) => UserController::get_user_from_id(id),

            None => anyhow::bail!("User id doesn't exist while trying to create user handler"),
        }
    }
}
