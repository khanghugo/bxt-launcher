#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::egui;
use egui::mutex::Mutex;

use crate::{
    config::Config,
    utils::{preview_file_being_dropped, run_bxt},
};

mod config;
mod utils;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([384.0, 200.0]),
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
}

impl BxtLauncher {
    fn new(config: Arc<Mutex<Config>>) -> Self {
        Self { config }
    }
}

impl eframe::App for BxtLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(
                "Make sure BunnymodXT.dll and bxt_rs.dll are in the same folder as Injector.exe",
            );

            ui.separator();

            let mut config = self.config.lock();

            egui::Grid::new("ui grid")
                .num_columns(3)
                .max_col_width(260.)
                // .min_col_width(100.)
                .show(ui, |ui| {
                    ui.label("hl.exe");
                    ui.text_edit_singleline(&mut config.hlexe);
                    if ui.button("+").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new().set_file_name("hl.exe").pick_file()
                        {
                            if path.extension().is_some_and(|ext| ext == "exe") {
                                config.hlexe = path.display().to_string();
                            }
                        }
                    }
                    ui.end_row();

                    ui.label("Injector.exe");
                    ui.text_edit_singleline(&mut config.injector);
                    if ui.button("+").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name("Injector.exe")
                            .pick_file()
                        {
                            if path.extension().is_some_and(|ext| ext == "exe") {
                                config.injector = path.display().to_string();
                            }
                        }
                    }
                    ui.end_row();

                    ui.label("Gamemod");

                    ui.add(egui::TextEdit::singleline(&mut config.gamemod).hint_text("valve"));
                    ui.end_row();

                    ui.label("Extra options");
                    ui.text_edit_singleline(&mut config.extras);
                    ui.end_row();

                    ui.end_row();
                    if ui.button("Run").clicked() {
                        println!("this runs");
                        run_bxt(&config);
                    }
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
                            .is_some_and(|filename| filename == "hl.exe")
                        {
                            config.hlexe = item.to_str().unwrap().to_string();
                        }

                        if item
                            .file_name()
                            .is_some_and(|filename| filename == "Injector.exe")
                        {
                            config.injector = item.to_str().unwrap().to_string();
                        }
                    }
                }
            });
        });
    }
}
