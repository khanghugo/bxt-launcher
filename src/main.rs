#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::Arc;

use eframe::egui;
use egui::mutex::Mutex;

use crate::{
    config::ConfigWithProfiles,
    utils::{preview_file_being_dropped, run_bxt},
};

mod config;
mod error;
mod utils;

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([432.0 * ZOOM_FACTOR, 182.0 * ZOOM_FACTOR])
            .with_resizable(false),
        ..Default::default()
    };

    // load config
    let config = Arc::new(Mutex::new(
        ConfigWithProfiles::load_from_default().unwrap_or_default(),
    ));

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
    config: Arc<Mutex<ConfigWithProfiles>>,
    status: String,
    save_timer: std::time::Instant,
}

impl BxtLauncher {
    fn new(config: Arc<Mutex<ConfigWithProfiles>>) -> Self {
        Self {
            config,
            status: String::from("Idle"),
            save_timer: std::time::Instant::now(),
        }
    }
}

const BXT_FILE_NAME_WINDOWS: &str = "BunnymodXT.dll";
const BXT_FILE_NAME_LINUX: &str = "libBunnymodXT.so";

const BXT_RS_FILE_NAME_WINDOWS: &str = "bxt_rs.dll";
const BXT_RS_FILE_NAME_LINUX: &str = "libbxt_rs.so";

const HL_EXE_FILE_NAME_WINDOWS: &str = "hl.exe";
const HL_EXE_FILE_NAME_LINUX: &str = "hl_linux";

const ZOOM_FACTOR: f32 = 1.25;
const SAVE_PERIOD: f32 = 30.;

impl eframe::App for BxtLauncher {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(ZOOM_FACTOR);

        // occasionally save the file so people don't forget
        {
            let now = std::time::Instant::now();
            if now.duration_since(self.save_timer).as_secs_f32() >= SAVE_PERIOD {
                match self.config.lock().write_to_default() {
                    Ok(_) => (),
                    Err(err) => self.status = err.to_string(),
                }
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut configs = self.config.lock();
            let profle_count = configs.configs.len();
            let current_profile = configs.current_profile;

            let config = &mut configs.configs[current_profile];

            #[cfg(not(windows))]
            let use_windows_files = config.use_wine && false;

            #[cfg(windows)]
            let use_windows_files = true;

            let hl_exe_file_name = if cfg!(windows) || use_windows_files {
                HL_EXE_FILE_NAME_WINDOWS
            } else {
                HL_EXE_FILE_NAME_LINUX
            };

            let bxt_file_name = if cfg!(windows) || use_windows_files {
                BXT_FILE_NAME_WINDOWS
            } else {
                BXT_FILE_NAME_LINUX
            };

            let bxt_rs_file_name = if cfg!(windows) || use_windows_files {
                BXT_RS_FILE_NAME_WINDOWS
            } else {
                BXT_RS_FILE_NAME_LINUX
            };

            egui::Grid::new("ui grid")
                .num_columns(4)
                .max_col_width(260. * ZOOM_FACTOR)
                .min_col_width(8.)
                .show(ui, |ui| {
                    ui.label(hl_exe_file_name);
                    ui.add(
                        egui::TextEdit::singleline(&mut config.hlexe)
                            .hint_text(format!("Drag-and-drop {}", hl_exe_file_name)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) =
                            rfd::FileDialog::new().set_file_name("hl.exe").pick_file()
                        {
                            if path
                                .file_name()
                                .is_some_and(|filename| filename == hl_exe_file_name)
                            {
                                config.hlexe = path.display().to_string();
                            }
                        }
                    }

                    // unused
                    // #[cfg(not(windows))]
                    // {
                    //     ui.checkbox(&mut config.use_wine, "")
                    //         .on_hover_text("Toggle running with Wine");
                    // }
                    ui.end_row();

                    ui.label("BunnymodXT");
                    ui.add(
                        egui::TextEdit::singleline(&mut config.bxt)
                            .hint_text(format!("Drag-and-drop {}", bxt_file_name)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(bxt_file_name)
                            .pick_file()
                        {
                            if path.file_name().is_some_and(|name| name == bxt_file_name) {
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
                            .hint_text(format!("Drag-and-drop {}", bxt_rs_file_name)),
                    );
                    if ui.button("+").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_file_name(bxt_rs_file_name)
                            .pick_file()
                        {
                            if path
                                .file_name()
                                .is_some_and(|name| name == bxt_rs_file_name)
                            {
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
                });

            ui.separator();

            // drop config
            let _ = config;

            // generational
            ui.horizontal(|ui| {
                for x in 0..profle_count {
                    if ui
                        .selectable_label(
                            configs.current_profile == x,
                            format!("Profile {}", x + 1),
                        )
                        .clicked()
                    {
                        configs.current_profile = x;

                        // just save any time i can
                        if let Err(err) = configs.write_to_default() {
                            self.status = err.to_string();
                        }
                    }
                }
            });

            // pick config up again
            let current_profile = configs.current_profile;
            let config = &mut configs.configs[current_profile];

            ui.separator();

            ui.horizontal(|ui| {
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
                            .is_some_and(|filename| filename == hl_exe_file_name)
                        {
                            config.hlexe = item.to_str().unwrap().to_string();
                        }

                        if item
                            .file_name()
                            .is_some_and(|filename| filename == bxt_file_name)
                        {
                            config.bxt = item.to_str().unwrap().to_string();
                        }

                        if item
                            .file_name()
                            .is_some_and(|filename| filename == bxt_rs_file_name)
                        {
                            config.bxt_rs = item.to_str().unwrap().to_string();
                        }
                    }
                }
            });
        });
    }
}
