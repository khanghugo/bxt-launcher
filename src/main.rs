#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::egui;
use egui::mutex::Mutex;

use crate::{
    config::Config,
    utils::{preview_file_being_dropped, run_bxt},
};

mod config;
mod error;
mod utils;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([408.0, 174.0]),
        ..Default::default()
    };

    // load config
    let config = Arc::new(Mutex::new(Config::load_from_default().unwrap_or_default()));

    let res = eframe::run_native(
        "bxt-launcher",
        options,
        Box::new(|_cc| {
            // This gives us image support:
            // egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(BxtLauncher::new(config.clone())))
        }),
    );

    // always write when app closes
    config.lock().write_to_default().unwrap();

    res
}

struct BxtLauncher {
    config: Arc<Mutex<Config>>,
    status: String,
}

impl BxtLauncher {
    fn new(config: Arc<Mutex<Config>>) -> Self {
        Self {
            config,
            status: String::from("Idle"),
        }
    }
}

const BXT_FILE_NAME: &str = if cfg!(windows) {
    "BunnymodXT.dll"
} else {
    "libBunnymodXT.so"
};

const BXT_RS_FILE_NAME: &str = if cfg!(windows) {
    "bxt_rs.dll"
} else {
    "libbxt_rs.so"
};

const HL_EXE_FILE_NAME: &str = if cfg!(windows) { "hl.exe" } else { "hl_linux" };

impl eframe::App for BxtLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.separator();

            let mut config = self.config.lock();

            egui::Grid::new("ui grid")
                .num_columns(4)
                .max_col_width(260.)
                .min_col_width(8.)
                .show(ui, |ui| {
                    ui.label(HL_EXE_FILE_NAME);
                    ui.add(
                        egui::TextEdit::singleline(&mut config.hlexe)
                            .hint_text(format!("Drag-and-drop {}", HL_EXE_FILE_NAME)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new().set_file_name("hl.exe").pick_file()
                        {
                            if path
                                .file_name()
                                .is_some_and(|filename| filename == HL_EXE_FILE_NAME)
                            {
                                config.hlexe = path.display().to_string();
                            }
                        }
                    }
                    ui.end_row();

                    ui.label("BunnymodXT");
                    ui.add(
                        egui::TextEdit::singleline(&mut config.bxt)
                            .hint_text(format!("Drag-and-drop {}", BXT_FILE_NAME)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(BXT_FILE_NAME)
                            .pick_file()
                        {
                            if path.extension().is_some_and(|ext| ext == "dll") {
                                config.bxt = path.display().to_string();
                            }
                        }
                    }
                    ui.checkbox(&mut config.enable_bxt, "")
                        .on_hover_text("Toggle BunnymodXT");
                    ui.end_row();

                    ui.label("bxt-rs");
                    ui.add(
                        egui::TextEdit::singleline(&mut config.bxt_rs)
                            .hint_text(format!("Drag-and-drop {}", BXT_RS_FILE_NAME)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(BXT_RS_FILE_NAME)
                            .pick_file()
                        {
                            if path.extension().is_some_and(|ext| ext == "dll") {
                                config.bxt_rs = path.display().to_string();
                            }
                        }
                    }
                    ui.checkbox(&mut config.enable_bxt_rs, "")
                        .on_hover_text("Toggle bxt-rs");
                    ui.end_row();

                    ui.label("Gamemod");

                    ui.add(egui::TextEdit::singleline(&mut config.gamemod).hint_text("valve"));
                    ui.end_row();

                    ui.label("Extra options");
                    ui.add(
                        egui::TextEdit::singleline(&mut config.extras)
                            .hint_text("More launch options"),
                    );
                    ui.end_row();

                    // extra space
                    ui.end_row();

                    // run but on
                    if ui.button("Run").clicked() {
                        match run_bxt(&config) {
                            Ok(_) => self.status = "OK".into(),
                            Err(err) => self.status = err.to_string(),
                        };
                    }

                    // status text
                    let mut text = self.status.as_str();
                    ui.text_edit_singleline(&mut text);
                });

            let ctx = ui.ctx();
            preview_file_being_dropped(ctx);

            // Collect dropped files:
            ctx.input(|i| {
                if i.raw.dropped_files.len() == 1 {
                    let item = i.raw.dropped_files[0].clone();
                    if let Some(item) = item.path {
                        if item
                            .file_name()
                            .is_some_and(|filename| filename == HL_EXE_FILE_NAME)
                        {
                            config.hlexe = item.to_str().unwrap().to_string();
                        }

                        if item
                            .file_name()
                            .is_some_and(|filename| filename == BXT_FILE_NAME)
                        {
                            config.bxt = item.to_str().unwrap().to_string();
                        }

                        if item
                            .file_name()
                            .is_some_and(|filename| filename == BXT_RS_FILE_NAME)
                        {
                            config.bxt_rs = item.to_str().unwrap().to_string();
                        }
                    }
                }
            });
        });
    }
}
