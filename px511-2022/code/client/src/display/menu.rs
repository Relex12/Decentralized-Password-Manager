//! Ce module gère toute l'interface utilisateur.
//! Affichage et gestion des entrées utilisateur pour naviguer et faire des actions dans l'application de gestion de mot de passe.
use egui::{Color32, Key, RichText, SidePanel};
use std::sync::mpsc::Receiver;
use utils::afaire;
use utils::data::debug_struct::DebugBuffer;

use crate::account_operation::{get_account, load_account};
use crate::keys_handler::renew_session_key;
use crate::pwd_handler::create_pwd;
use crate::rq_operations::pull_notif;

pub fn left_panel(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Gestion de compte");
    ui.set_width(400.0);

    // Bloc création de compte
    ui.label("Création/chargement de compte");

    ui.horizontal(|ui| {
        let lab = ui.label("entrez votre nom: ");
        ui.text_edit_singleline(&mut app.name).labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        let lab = ui.label("entrez votre Prénom: ");
        ui.text_edit_singleline(&mut app.firstname)
            .labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        let lab = ui.label("entrez votre numéro de téléphone: ");
        ui.text_edit_singleline(&mut app.tel).labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Valider")).clicked() {
            app.accnt_loaded = false;
            load_account(
                app.tel.parse::<u64>().unwrap(),
                app.name.clone(),
                app.firstname.clone(),
            )
            .expect("Load account creation impossible");
        }
    });

    //Bloc ajout/modification mots de passe
    ui.horizontal(|ui| {
        ui.radio_value(
            &mut app.pwd_choice,
            PwdChoice::PwdCrea,
            "Création d'un mot de passe",
        );
        ui.radio_value(
            &mut app.pwd_choice,
            PwdChoice::PwdChange,
            "Modification d'un mot de passe (TODO)",
        );
    });

    ui.horizontal(|ui| {
        let lab = ui.label("Site web: ");
        ui.text_edit_singleline(&mut app.pwd_website)
            .labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        let lab = ui.label("Identifiant: ");
        ui.text_edit_singleline(&mut app.pwd_login)
            .labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        let lab = ui.label("Mot de passe: ");
        ui.text_edit_singleline(&mut app.pwd_pass)
            .labelled_by(lab.id);
    });

    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Valider (TODO:DEMANDER LE MASTER PWD"))
            .clicked()
        {
            match app.pwd_choice {
                PwdChoice::PwdCrea => {
                    create_pwd(
                        &String::from("my master pwd"),
                        app.pwd_website.clone(),
                        app.pwd_login.clone(),
                        app.pwd_pass.clone(),
                    )
                    .expect("Impossible to create password");
                }
                PwdChoice::PwdChange => {
                    afaire!("Modification de mot de passe")
                }
            };
        }
    });

    // Bloc suppression de mot de passe
    ui.label("Suppression de mots de passe");
    ui.horizontal(|ui| {
        let lab = ui.label("Site web: ");
        ui.text_edit_singleline(&mut app.pwd_website)
            .labelled_by(lab.id);
    });

    ui.horizontal(
        |ui| {
            if ui.add(egui::Button::new("Valider TODO")).clicked() {}
        },
    );

    ui.horizontal(|ui| {
        if ui.add(egui::Button::new("Force pull notif")).clicked() {
            pull_notif();
        }
    });

    ui.horizontal(|ui| {
        if ui
            .add(egui::Button::new("Force change session key"))
            .clicked()
        {
            renew_session_key().expect("Erreur lors du reset de la clé de session");
        }
    });
}

pub fn central_panel(ui: &mut egui::Ui, app: &mut MyApp) {
    ui.heading("Vue du compte");
    ui.set_width(400.0);

    // Bloc informations sur le compte
    ui.label("Information sur le compte :");
    if !app.accnt_loaded {
        ui.label("Pas de compte");

        if let Ok(local_accnt) = get_account() {
            app.firstname = local_accnt.account.first_name;
            app.name = local_accnt.account.name;
            app.tel = local_accnt.account.phone.to_string();
            app.accnt_loaded = true;
        }
    } else {
        ui.label(format!("Bienvenue {} {}", app.firstname, app.name));
        ui.label(format!("Numéro de téléphone: {}", app.tel));
    }

    // Bloc affichage des mots de passe
    ui.label("Liste des mots de passe:");
}

#[derive(PartialEq)]
pub enum AccntChoice {
    AcntCrea,
    AcntLoad,
}

#[derive(PartialEq)]
pub enum PwdChoice {
    PwdCrea,
    PwdChange,
}

pub struct MyApp {
    pub rx: Receiver<Vec<DebugBuffer>>,
    pub lines: Vec<DebugBuffer>,
    pub index_sel: u64,
    pub index_cpt: u64,

    pub name: String,
    pub firstname: String,
    pub tel: String,

    pub pwd_website: String,
    pub pwd_login: String,
    pub pwd_pass: String,
    pub pwd_choice: PwdChoice,

    pub accnt_loaded: bool,

    pub force_pull: bool,
}

impl MyApp {
    pub fn get(rx: Receiver<Vec<DebugBuffer>>) -> Self {
        Self {
            rx,
            lines: Vec::<DebugBuffer>::new(),
            index_sel: 0,
            index_cpt: 0,
            name: String::new(),
            firstname: String::new(),
            tel: String::new(),
            pwd_website: String::new(),
            pwd_login: String::new(),
            pwd_pass: String::new(),
            pwd_choice: PwdChoice::PwdCrea,
            accnt_loaded: false,
            force_pull: false,
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        SidePanel::right("Flux serveur").show(ctx, |ui| {
            ui.heading("Flux serveur");
            ui.set_width(400.0);

            if let Ok(mut msg) = self.rx.try_recv() {
                if msg.len() != self.lines.len() || self.lines.is_empty() {
                    for (i, debug) in msg.iter_mut().enumerate() {
                        println!("MSG reçut {}", debug.print());
                        debug.set_index(i.try_into().unwrap());
                    }

                    self.lines = msg;
                }
            }

            if ctx.input().key_pressed(Key::ArrowDown)
                && self.index_sel + 1 < self.lines.len() as u64
            {
                self.index_sel += 1;
            }

            if ctx.input().key_pressed(Key::ArrowUp) && self.index_sel > 0 {
                self.index_sel -= 1;
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (actual, line) in self.lines.iter().enumerate() {
                    if self.index_sel == actual as u64 {
                        ui.label(RichText::new(line.print()).color(Color32::GREEN));
                    } else {
                        ui.label(RichText::new(line.print()).color(Color32::WHITE));
                    }
                }
            });
        });

        SidePanel::left("Opérations de compte").show(ctx, |ui| {
            left_panel(ui, self);
        });

        SidePanel::right("Vue du compte").show(ctx, |ui| {
            central_panel(ui, self);
        });
    }
}
