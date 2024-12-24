use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use eframe::{self, egui};

#[derive(Serialize, Deserialize, Debug, Default)]
struct PasswordEntry {
    site: String,
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize, Default)]
struct PasswordDatabase {
    entries: Vec<PasswordEntry>,
}

impl PasswordDatabase {
    fn load_from_file(file_path: &str) -> io::Result<Self> {
        let file_content = fs::read_to_string(file_path).unwrap_or_else(|_| "{}".to_string());
        let database: PasswordDatabase = serde_json::from_str(&file_content).unwrap_or_default();
        Ok(database)
    }

    fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(self)?;
        let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    fn add_entry(&mut self, site: String, username: String, password: String) {
        self.entries.push(PasswordEntry { site, username, password });
    }

    fn delete_entry(&mut self, index: usize) {
        if index < self.entries.len() {
            self.entries.remove(index);
        }
    }
}

struct PasswordManagerApp {
    db: PasswordDatabase,
    db_path: String,
    new_site: String,
    new_username: String,
    new_password: String,
}

impl PasswordManagerApp {
    fn new(db_path: String) -> Self {
        let db = PasswordDatabase::load_from_file(&db_path).unwrap_or_default();
        Self {
            db,
            db_path,
            new_site: String::new(),
            new_username: String::new(),
            new_password: String::new(),
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Password Manager");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label("Created by aKaan47_");
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.label("Add New Password");
                ui.horizontal(|ui| {
                    ui.label("Site:");
                    ui.text_edit_singleline(&mut self.new_site);
                });
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut self.new_username);
                });
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.text_edit_singleline(&mut self.new_password);
                });

                if ui.button("Add Entry").clicked() {
                    self.db.add_entry(
                        self.new_site.clone(),
                        self.new_username.clone(),
                        self.new_password.clone(),
                    );
                    self.db.save_to_file(&self.db_path).unwrap();
                    self.new_site.clear();
                    self.new_username.clear();
                    self.new_password.clear();
                }
            });

            ui.separator();
            ui.label("Stored Passwords:");
            ui.add_space(5.0);

            let mut indices_to_remove = Vec::new();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, entry) in self.db.entries.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(&entry.site);
                        ui.label(&entry.username);
                        ui.label(&entry.password);
                        if ui.button("Delete").clicked() {
                            indices_to_remove.push(i);
                        }
                    });
                }
            });

            for &index in indices_to_remove.iter().rev() {
                self.db.delete_entry(index);
            }
            self.db.save_to_file(&self.db_path).unwrap();
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let db_path = "passwords.json".to_string();
    let app = PasswordManagerApp::new(db_path);
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(600.0, 400.0)),
        ..Default::default()
    };
    eframe::run_native("Password Manager", options, Box::new(|_cc| Box::new(app)))
}
