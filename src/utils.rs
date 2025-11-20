use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

// use windows::Win32::Foundation::*;
// use windows::Win32::Security::*;
use windows::Win32::System::Threading::*;
use windows::core::*;

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

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(target_os = "windows")]
pub fn run_bxt(config: &Config) -> eyre::Result<()> {
    println!("this runs");
    use std::mem;

    let Config {
        injector,
        hlexe,
        gamemod,
        extras,
    } = config.clone();

    // spawn process
    let process_path = to_wide(&hlexe);

    let mut si = STARTUPINFOW::default();
    si.cb = mem::size_of::<STARTUPINFOW>() as u32;
    let mut pi = PROCESS_INFORMATION::default();

    println!("creating process");
    unsafe {
        CreateProcessW(
            PCWSTR(process_path.as_ptr()), // application
            None,                          // arguments
            None,
            None,
            false,
            // suspended to load bxt-rs at Memory_Init()
            CREATE_SUSPENDED | CREATE_NEW_CONSOLE,
            None,
            None,
            &si,
            &mut pi,
        )?;
    }

    println!("done creating");

    // inject
    let target_process = dll_syringe::process::OwnedProcess::from_pid(pi.dwProcessId)?;
    let syringe = dll_syringe::Syringe::for_process(target_process);
    let injected_payload = syringe
        .inject("/home/khang/bxt/bxt-rs/target/i686-pc-windows-gnu/release/bxt_rs.dll")
        .unwrap();

    // let it go
    unsafe {
        ResumeThread(pi.hThread);
        // CloseHandle(pi.hThread);
        // CloseHandle(pi.hProcess);
    }

    // let my_args = format!("{hlexe} -game {gamemod} {extras}");
    // let args_iter: Vec<String> = my_args
    //     .split_ascii_whitespace()
    //     .map(|x| x.to_owned())
    //     .collect();

    // thread::spawn(move || Ok(Command::new(injector).args(args_iter).output()?))

    Ok(())
}
