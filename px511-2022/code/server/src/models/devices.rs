use std::collections::HashSet;

use anyhow::{Ok, Result};
use k256::PublicKey;
use serde::{Deserialize, Serialize};

use utils::data::comm_struct::{DeviceInfo, EsiPassMsg};

#[derive(Serialize, Deserialize, Clone, Debug)]
/// Structure de gestion des appreils d'un utilisateur.
pub struct DevicesManager {
    /// Objet de gestion de fichier de mots de passe.
    pub pwd_manager: PwdFileManager,
    /// Liste d'appareils.
    pub devices: Vec<Device>,
    /// Buffer de message commun à tous les appareils.
    pub global_inbox: Vec<EsiPassMsg>,
}

impl DevicesManager {
    #[allow(unused)]
    /// Ajoute un device au compte depuis une requête.
    /// Pré-condition : la requête est de type create_device
    /// # Arguments
    /// * 'rq_data' - Un objet EsiPassRqst qui contient les données envoyées par le client.
    pub fn create_device(&mut self, rq_data: &EsiPassMsg) -> anyhow::Result<(u64)> {
        let exp = rq_data
            .clone()
            .exp
            .ok_or_else(|| anyhow::Error::msg("Device id missing"))?;

        // Recherche d'un device ID unique.
        let mut ids = HashSet::<u64>::new();
        for dev in self.devices.iter() {
            ids.insert(dev.dev_info.device_id);
        }

        let mut rng = rand::thread_rng();
        let mut new_id: u64 = 1;

        while ids.contains(&new_id) {
            new_id = rand::Rng::gen_range(&mut rng, 1..999_999_999_999);
        }

        // Création d'un device.
        let dev = Device::create_device(exp.dev_name, new_id, exp.public_key, false);

        self.devices.push(dev);
        Ok(new_id)
    }

    #[allow(unused)]
    pub fn check_updates(dev_id: u32) {
        todo!();
    }

    #[allow(unused)]
    /// Ajoute une modification de fichier de mots de passe.
    /// # Arguments
    /// * 'update' - String représentant un fichier de mot de passe chiffré par le client.
    pub fn add_file_update(&mut self, update: String) {
        // Redirection vers le bon appel de méthode.
        // Réduit la taille du code, le programmeur descend moins dans les appels d'attributs.
        self.pwd_manager.add_file_update(update);
    }

    #[allow(unused)]
    pub fn get_inbox(dev_id: u32) -> Result<()> {
        todo!();
    }

    /// Retourne la liste devs_info qui contient le nom, l'ID et la clé publique de tous les appareils du compte.
    pub fn get_devs_info(&self) -> Vec<DeviceInfo> {
        let mut devs_info = Vec::<DeviceInfo>::new();

        for dev in &self.devices {
            devs_info.push(dev.dev_info.clone());
        }

        devs_info
    }

    /// Incrémente le numéro de version du fichier de mots de passe de l'appareil correspondant à dev_id.
    /// # Arguments
    /// * 'dev_id' - ID de l'appareil à modifier.
    /// Retourne la nouvelle version.
    pub fn update_device_version(&mut self, dev_id: u64) -> anyhow::Result<u64> {
        let mut dev = self.find_device(dev_id)?;
        dev.file_version += 1;
        Ok(dev.file_version)
    }

    // Récupération d'un Device depuis son ID.
    /// # Arguments
    /// * 'dev_id' - ID de l'appareil à modifier.
    /// Retourne l'objet représentant l'appareil.
    pub fn find_device(&mut self, dev_id: u64) -> anyhow::Result<&mut Device> {
        let mut devices_iter = self.devices.iter_mut();

        loop {
            // Ittération sur tous les membres du vecteur.
            let dev = devices_iter.next();

            // Si l'ittération est terminée sans trouver l'ID de l'appareil une erreur est retournée.
            if dev.is_none() {
                anyhow::bail!("Device ID {dev_id} has not been found in the server")
            }

            // Autrement l'ittération retourne un élément existant
            // COmparaison de l'ID du device actuel avec celui recherché.
            let existing_dev = dev.unwrap();
            if existing_dev.dev_info.device_id == dev_id {
                return Ok(existing_dev);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Device {
    pub dev_info: DeviceInfo,
    pub session_key_up_to_date: bool,
    pub file_version: u64,
    pub inbox: Vec<EsiPassMsg>,
}

impl Device {
    pub fn create_device(
        device_name: String,
        dev_id: u64,
        pub_key: PublicKey,
        session_key_up_to_date: bool,
    ) -> Device {
        Device {
            dev_info: DeviceInfo {
                device_id: dev_id,
                dev_name: device_name,
                public_key: pub_key,
            },
            session_key_up_to_date,
            file_version: 0,
            inbox: Vec::<EsiPassMsg>::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PwdFileBuffer {
    pub pwd_file: String,
    pub actual_session_key: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PwdFileManager {
    pub last_file_version: u64,
    pub pwd_file_updates: Vec<PwdFileBuffer>,
}

impl PwdFileManager {
    #[allow(unused)]
    pub fn reset_files() {
        todo!();
    }

    #[allow(unused)]
    pub fn add_file_update(&mut self, update: String) {
        self.pwd_file_updates.push(PwdFileBuffer {
            pwd_file: update,
            actual_session_key: true,
        });
    }

    /// Met à jour le numéro de version du fichier de mots de passe le plus récent de tous les appareils
    /// si la version passée en argument est plus grande que l'actuelle.
    pub fn check_ver_update(&mut self, new_ver: u64) {
        if self.last_file_version < new_ver {
            self.last_file_version = new_ver;
        }
    }
}
