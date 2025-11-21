use egui::{Align2, Color32, Id, LayerId, Order, TextStyle};

use crate::{config::Config, error::LauncherError};

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

#[cfg(not(windows))]
pub fn run_bxt(config: &Config) -> Result<(), LauncherError> {
    use std::process::Command;

    let Config {
        hlexe,
        bxt,
        bxt_rs,
        gamemod,
        extras,
        enable_bxt,
        enable_bxt_rs,
        use_wine,
    } = config.clone();

    if hlexe.is_empty() {
        return Err(LauncherError::NoHLExe);
    }

    // if use_wine {
    //     Command::new(hlexe).arg(arg)
    // }

    Ok(())
}

#[cfg(windows)]
pub fn run_bxt(config: &Config) -> Result<(), LauncherError> {
    use std::{mem, path::Path};

    use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
    use windows::Win32::Foundation::*;
    use windows::Win32::System::Threading::*;
    use windows::core::*;

    // bxt-rs and BunnymodXT will fire this event when it is done loading
    const EVENT_NAME: &str = "BunnymodXT-Injector";

    fn to_wide(s: &str) -> Vec<u16> {
        OsStr::new(s)
            .encode_wide()
            // add null terminator
            .chain(std::iter::once(0))
            .collect()
    }

    let Config {
        hlexe,
        bxt,
        bxt_rs,
        gamemod,
        extras,
        enable_bxt,
        enable_bxt_rs,
        use_wine: _,
    } = config.clone();

    if hlexe.is_empty() {
        return Err(LauncherError::NoHLExe);
    }

    let mut bxt_dlls_to_inject: Vec<&Path> = vec![];

    // always inject bxt-rs before BunnymodXT
    // bxt-rs
    if enable_bxt_rs {
        let path = Path::new(bxt_rs.as_str());

        if path.exists() && path.is_file() {
            bxt_dlls_to_inject.push(path);
        } else {
            return Err(LauncherError::FileDoesNotExist { path: path.into() });
        }
    }

    // BunnymodXT
    if enable_bxt {
        let path = Path::new(bxt.as_str());

        if path.exists() && path.is_file() {
            bxt_dlls_to_inject.push(path);
        } else {
            return Err(LauncherError::FileDoesNotExist { path: path.into() });
        }
    }

    // spawn process
    let process_path = to_wide(&hlexe);
    let gamemod = if gamemod.is_empty() {
        "valve"
    } else {
        &gamemod
    };

    let arguments = format!("-game {gamemod} {extras}");
    let mut arguments = to_wide(&arguments); // mutable for mutable pointer

    let mut si = STARTUPINFOW::default();
    si.cb = mem::size_of::<STARTUPINFOW>() as u32;
    let mut pi = PROCESS_INFORMATION::default();

    unsafe {
        CreateProcessW(
            PCWSTR(process_path.as_ptr()),       // application
            Some(PWSTR(arguments.as_mut_ptr())), // arguments
            None,
            None,
            false,
            // suspended to load bxt-rs at Memory_Init()
            CREATE_SUSPENDED | DETACHED_PROCESS,
            None,
            None,
            &si,
            &mut pi,
        )?;
    }

    // needs to wait for DLL to be loaded
    // this event is emitted by BunnymodXT and bxt-rs upon finishing loading
    let event_name = to_wide(EVENT_NAME);
    let resume_event = unsafe { CreateEventW(None, false, false, PCWSTR(event_name.as_ptr())) }?;

    // inject
    for dll in bxt_dlls_to_inject {
        let target_process = dll_syringe::process::OwnedProcess::from_pid(pi.dwProcessId)?;
        let syringe = dll_syringe::Syringe::for_process(target_process);
        let _injected_payload = syringe.inject(dll)?;

        unsafe {
            if WaitForSingleObject(resume_event, INFINITE) == WAIT_FAILED {
                return Err(LauncherError::InjectionFailed {
                    reason: "Failed to wait for resume event".to_owned(),
                });
            }

            // need to reset event so the next dll can use it
            ResetEvent(resume_event)?;
        }
    }

    unsafe {
        ResumeThread(pi.hThread);
        CloseHandle(pi.hThread)?;
        CloseHandle(pi.hProcess)?;
    }

    Ok(())
}
