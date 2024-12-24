use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use serde::{Deserialize, Serialize};
use eframe::{self, egui};
use egui::{Color32, RichText, Rounding, Vec2};
use arboard::Clipboard;

// Palette de couleurs pour le thème sombre
const BACKGROUND: Color32 = Color32::from_rgb(28, 28, 35);
const SURFACE: Color32 = Color32::from_rgb(38, 38, 45);
const PRIMARY: Color32 = Color32::from_rgb(130, 170, 255);
const TEXT: Color32 = Color32::from_rgb(230, 230, 240);
const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 190);
const ACCENT: Color32 = Color32::from_rgb(130, 130, 255);
const BUTTON_BG: Color32 = Color32::from_rgb(45, 45, 60);
const BUTTON_HOVER: Color32 = Color32::from_rgb(55, 55, 70);

// Le reste des structures reste identique
#[derive(Serialize, Deserialize, Debug, Default)]
struct PasswordEntry {
    site: String,
    username: String,
    password: String,
    is_password_visible: bool,
}

#[derive(Serialize, Deserialize, Default)]
struct PasswordDatabase {
    entries: Vec<PasswordEntry>,
}

// Implémentations de PasswordDatabase inchangées
impl PasswordDatabase {
    fn load_from_file(file_path: &str) -> io::Result<Self> {
        let file_content = fs::read_to_string(file_path).unwrap_or_else(|_| "{}".to_string());
        let mut database: PasswordDatabase = serde_json::from_str(&file_content).unwrap_or_default();
        for entry in &mut database.entries {
            entry.is_password_visible = false;
        }
        Ok(database)
    }

    fn save_to_file(&self, file_path: &str) -> io::Result<()> {
        let json_data = serde_json::to_string_pretty(self)?;
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(file_path)?;
        file.write_all(json_data.as_bytes())?;
        Ok(())
    }

    fn add_entry(&mut self, site: String, username: String, password: String) {
        self.entries.push(PasswordEntry {
            site,
            username,
            password,
            is_password_visible: false,
        });
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
    search_query: String,
    clipboard: Clipboard,
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
            search_query: String::new(),
            clipboard: Clipboard::new().unwrap(),
        }
    }
}

impl eframe::App for PasswordManagerApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Configuration du style global
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 10.0);
        style.visuals.window_rounding = Rounding::same(12.0);
        style.visuals.widgets.noninteractive.rounding = Rounding::same(8.0);
        style.visuals.widgets.inactive.rounding = Rounding::same(8.0);
        style.visuals.window_fill = BACKGROUND;
        style.visuals.panel_fill = SURFACE;
        style.visuals.widgets.inactive.bg_fill = BUTTON_BG;
        style.visuals.widgets.hovered.bg_fill = BUTTON_HOVER;
        style.visuals.widgets.active.bg_fill = BUTTON_HOVER;
        style.visuals.widgets.inactive.fg_stroke.color = TEXT;
        style.visuals.widgets.hovered.fg_stroke.color = TEXT;
        style.visuals.widgets.active.fg_stroke.color = TEXT;
        ctx.set_style(style);

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(15.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("Password Manager")
                    .size(32.0)
                    .color(PRIMARY)
                    .strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(RichText::new("Created by aKaan47_")
                        .italics()
                        .size(16.0)
                        .color(TEXT_SECONDARY));
                });
            });
            ui.add_space(15.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(20.0);
            
            // Section d'ajout de mot de passe
            egui::Frame::none()
                .fill(SURFACE)
                .rounding(Rounding::same(16.0))
                .shadow(egui::epaint::Shadow::small_dark())
                .show(ui, |ui| {
                    ui.add_space(20.0);
                    ui.heading(RichText::new("Add New Password")
                        .size(24.0)
                        .color(PRIMARY));
                    ui.add_space(15.0);
                    
                    let text_edit_width = 300.0;
                    let label_size = 16.0;
                    
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Site:").size(label_size).color(TEXT));
                        ui.add(egui::TextEdit::singleline(&mut self.new_site)
                            .desired_width(text_edit_width));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Username:").size(label_size).color(TEXT));
                        ui.add(egui::TextEdit::singleline(&mut self.new_username)
                            .desired_width(text_edit_width));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        ui.label(RichText::new("Password:").size(label_size).color(TEXT));
                        ui.add(egui::TextEdit::singleline(&mut self.new_password)
                            .password(true)
                            .desired_width(text_edit_width));
                    });

                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        ui.add_space(10.0);
                        if ui.add_sized([140.0, 35.0], egui::Button::new(
                            RichText::new("Add Entry").size(16.0)
                        )).clicked() && !self.new_site.is_empty() {
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
                    ui.add_space(15.0);
                });

            ui.add_space(25.0);
            
            // Barre de recherche
            ui.horizontal(|ui| {
                ui.add_space(10.0);
                ui.label(RichText::new("Search:").size(16.0).color(TEXT));
                ui.add(egui::TextEdit::singleline(&mut self.search_query)
                    .desired_width(300.0));
            });

            ui.add_space(20.0);

            // Section des mots de passe stockés
            ui.heading(RichText::new("Stored Passwords")
                .size(24.0)
                .color(PRIMARY));
            ui.add_space(15.0);

            let mut indices_to_remove = Vec::new();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, entry) in self.db.entries.iter_mut().enumerate() {
                    if self.search_query.is_empty() || 
                       entry.site.to_lowercase().contains(&self.search_query.to_lowercase()) ||
                       entry.username.to_lowercase().contains(&self.search_query.to_lowercase()) {
                        
                        egui::Frame::none()
                            .fill(SURFACE)
                            .rounding(Rounding::same(12.0))
                            .shadow(egui::epaint::Shadow::small_dark())
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(15.0);
                                    ui.label(RichText::new(&entry.site)
                                        .size(16.0)
                                        .strong()
                                        .color(ACCENT));
                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);
                                    ui.label(RichText::new(&entry.username)
                                        .size(16.0)
                                        .color(TEXT));
                                    ui.add_space(10.0);
                                    ui.separator();
                                    ui.add_space(10.0);
                                    
                                    if ui.add_sized([70.0, 28.0], egui::Button::new(
                                        RichText::new(if entry.is_password_visible { "Hide" } else { "Show" })
                                            .size(14.0)
                                    )).clicked() {
                                        entry.is_password_visible = !entry.is_password_visible;
                                    }
                                    
                                    ui.add_space(5.0);
                                    let password_text = if entry.is_password_visible {
                                        &entry.password
                                    } else {
                                        "••••••••"
                                    };
                                    ui.label(RichText::new(password_text)
                                        .size(16.0)
                                        .color(TEXT));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.add_sized([35.0, 28.0], egui::Button::new("🗑"))
                                            .clicked() {
                                            indices_to_remove.push(i);
                                        }
                                        ui.add_space(5.0);
                                        if ui.add_sized([80.0, 28.0], egui::Button::new(
                                            RichText::new("Copy").size(14.0)
                                        )).clicked() {
                                            let _ = self.clipboard.set_text(&entry.password);
                                        }
                                        ui.add_space(15.0);
                                    });
                                });
                            });
                        ui.add_space(10.0);
                    }
                }
            });

            for &index in indices_to_remove.iter().rev() {
                self.db.delete_entry(index);
            }
            if !indices_to_remove.is_empty() {
                self.db.save_to_file(&self.db_path).unwrap();
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let db_path = "passwords.json".to_string();
    let app = PasswordManagerApp::new(db_path);
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(900.0, 700.0)),
        min_window_size: Some(egui::vec2(600.0, 400.0)),
        centered: true,
        ..Default::default()
    };
    eframe::run_native(
        "Password Manager",
        options,
        Box::new(|_cc| Box::new(app))
    )
}