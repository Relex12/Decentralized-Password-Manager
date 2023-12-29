use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

use super::comm_struct::{DeviceInfo, EsiPassMsg};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DebugBuffer {
    pub origine: Instance,
    pub destination: Instance,
    pub msg_in: String,
    pub msg_out: String,
    pub index: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instance {
    pub accnt_id: Option<u64>,
    pub dev: DeviceInfo,
}

impl DebugBuffer {
    pub fn print(&self) -> String {
        let og_accnt = match self.origine.accnt_id {
            Some(id) => id.to_string(),
            None => String::from("Unknown"),
        };

        let dest_accnt = match self.destination.accnt_id {
            Some(id) => id.to_string(),
            None => String::from("Unknown"),
        };

        format!(
            "------------------------------------------------------------------------------------------\nRequete {}\n\n{}@{}  ------------------>>>   Serveur:\n\n{}\n\n{}@{}  <<<------------------   Serveur\n\n{}------------------------------------------------------------------------------------------\n",
            self.index,
            og_accnt,
            self.origine.dev.dev_name,
            self.msg_in,
            dest_accnt,
            self.destination.dev.dev_name,
            self.msg_out
        )
    }

    // Modifie l'index de la ligne de debogage
    pub fn set_index(&mut self, index: u64) {
        self.index = index;
    }

    // Ajoute une ligne de description pour le debug.
    pub fn add_msg(msg: &mut String, str: &str) {
        msg.push_str(format!("\t-{}\n", str).as_str());
    }

    /// Méthode appelée par le serveur pour sauvegarder des logs pour le débogage.
    pub fn to_debug(request: EsiPassMsg, response: EsiPassMsg, msg_in: String, msg_out: String) {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../server/src/data/debug");

        // L'index est recalculé directement dans le programme de debbuging.
        let mut debug_obj = DebugBuffer {
            origine: Instance {
                accnt_id: request.accnt.user_id,
                dev: request.exp.clone().unwrap(),
            },
            msg_in,
            msg_out,
            index: 0,
            destination: Instance {
                accnt_id: response.accnt.user_id,
                dev: request.exp.unwrap(),
            },
        };

        // Récupération du vecteur de debug existant
        let mut debug_vec = match std::fs::read_to_string(path.clone()) {
            Ok(str) => {
                let debug_vec = serde_json::from_str::<Vec<DebugBuffer>>(str.as_str())
                    .expect("Deserializing debug file error");
                debug_vec
            }
            Err(_) => Vec::<DebugBuffer>::new(),
        };

        if !debug_vec.is_empty() {
            debug_obj.index = debug_vec.last().unwrap().index + 1;
        }

        debug_vec.push(debug_obj);
        //On sauvegarde la structure du challenge du client en question.
        let json = serde_json::to_string(&debug_vec).expect("Serializing debug vec error");
        let mut file = File::create(&path).expect("Opening debug file error");
        file.write_all(json.as_bytes())
            .expect("Serializing debug file error");
    }
}
