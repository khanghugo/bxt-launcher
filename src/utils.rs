use std::{
    process::{Command, Output},
    thread::{self, JoinHandle},
};

use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

use crate::config::Config;

// from gchimp
pub fn preview_file_being_dropped(ctx: &egui::Context) {
    preview_files_being_dropped_min_max_file(ctx, 1, 1);
}

pub fn preview_files_being_dropped_min_max_file(ctx: &egui::Context, min: usize, max: usize) {
    if ctx.input(|i| min <= i.raw.hovered_files.len() && i.raw.hovered_files.len() <= max) {
        let painter =
            ctx.layer_painter(LayerId::new(Order::Foreground, Id::new("file_drop_target")));

        let content_rect = ctx.content_rect();
        painter.rect_filled(content_rect, 0.0, Color32::from_black_alpha(192));
        painter.text(
            content_rect.center(),
            Align2::CENTER_CENTER,
            "Drag-n-Drop",
            TextStyle::Heading.resolve(&ctx.style()),
            Color32::WHITE,
        );
    }
}

pub fn run_bxt(config: &Config) -> JoinHandle<eyre::Result<Output>> {
    let Config {
        injector,
        hlexe,
        gamemod,
        extras,
    } = config.clone();

    let my_args = format!("{hlexe} -game {gamemod} {extras}");
    let args_iter: Vec<String> = my_args
        .split_ascii_whitespace()
        .map(|x| x.to_owned())
        .collect();

    thread::spawn(move || Ok(Command::new(injector).args(args_iter).output()?))
}
