use k256::PublicKey;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

impl Display for EsiPassMsgType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EsiPassMsgType::UpdatePwdFile => write!(f, "Modification de mots de passe"),
            EsiPassMsgType::ChangeSessionKey => write!(f, "Changement de la clé de session"),
            EsiPassMsgType::PullNotif => write!(f, "Récupération de notification"),
            EsiPassMsgType::RequestDHE => write!(f, "Initialisation d'un échange DHE"),
            EsiPassMsgType::Challenge => write!(f, "Challenge en cours"),
            EsiPassMsgType::LoadAccount => write!(f, "Opération sur le compte"),
            EsiPassMsgType::RespDHE => write!(f, "Réponse à une requête DHE"),
            EsiPassMsgType::RequestAllPwds => {
                write!(f, "Demande d'envoi de tous les mots de passe")
            }
        }
    }
}

/// Enumerateur sur les différents types de message EsiPassMsg
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EsiPassMsgType {
    Challenge,
    LoadAccount,
    UpdatePwdFile,
    PullNotif,
    ChangeSessionKey,
    RequestDHE,
    RespDHE,
    RequestAllPwds,
}

/// Structure de donnée que l'on transfert entre nos clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EsiPassMsg {
    pub rq_type: EsiPassMsgType,
    pub exp: Option<DeviceInfo>,
    pub dest: Option<u64>,
    pub accnt: Account,
    pub signature: Option<u64>,
    pub msg: Option<String>,
}

/// Structure de donnée représentant les comptes clients
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Account {
    pub user_id: Option<u64>,
    pub phone: u64,
    pub name: String,
    pub first_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeviceInfo {
    pub device_id: u64,
    pub dev_name: String,
    pub public_key: PublicKey,
}

impl Hash for DeviceInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.device_id.hash(state);
    }
}

impl PartialEq for DeviceInfo {
    fn eq(&self, other: &Self) -> bool {
        self.device_id == other.device_id
    }
}

impl Eq for DeviceInfo {}
