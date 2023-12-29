use client::{display::menu::*, rq_operations::build_reqwest_client};
use std::{sync::mpsc::channel, thread, time::Duration};
use utils::data::debug_struct::DebugBuffer;

/// Point d'entrée de l'application `EsiPass`.
/// Utilise une structure `Args` de la crate `clap::Parser` pour récupérer des arguments au lancement de l'application.
fn main() {
    let (tx, rx) = channel::<Vec<DebugBuffer>>();

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(1263.0, 900.0)),
        ..Default::default()
    };
    let app = MyApp::get(rx);

    thread::spawn(move || {
        let client = build_reqwest_client().unwrap();

        loop {
            let serv_resp = send_debug_request(&client);
            if let Ok(serv_resp) = serv_resp {
                tx.send(serv_resp)
                    .expect("Sending into debug channel error");
            } else {
                println!("Error while puling debug information");
            }
            std::thread::sleep(Duration::from_secs(3));
        }
    });

    println!("Avant");
    eframe::run_native(
        "Démonstration EsiPass",
        options,
        Box::new(move |_cc| Box::new(app)),
    );
    println!("Après");
}

/// Fonction d'envoi du String msg dans une requete HTTP par le client reqwest
pub fn send_debug_request(client: &reqwest::blocking::Client) -> anyhow::Result<Vec<DebugBuffer>> {
    let resp = match client
        .get(format!("{}/debug", client::rq_operations::SERV_URL))
        .send()
    {
        Ok(ok) => {
            let rep = ok.text().unwrap();
            let o: Vec<DebugBuffer> = serde_json::from_str(rep.as_str())?;

            //println!("Objet désereliarisé\n{:#?}", o);
            o
        }

        Err(err) => {
            println!("ERR\n{:#?}", err);
            std::process::exit(0);
        }
    };

    Ok(resp)
}
