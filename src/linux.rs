use std::{env, path::PathBuf};

use crate::{config::Config, error::LauncherError};

pub fn run_bxt(config: &Config) -> Result<(), LauncherError> {
    use std::{env, path::Path, process::Command};

    use crate::linux::get_steam_run;

    let config = config.trim();

    config.validate()?;

    let Config {
        hlexe,
        bxt,
        bxt_rs,
        gamemod,
        extras,
        enable_bxt,
        enable_bxt_rs,
        use_wine,
    } = config;

    let Some(steam_run_path) = get_steam_run() else {
        return Err(LauncherError::CannotFindSteam);
    };

    if use_wine && !Path::new("/usr/bin/wine").exists() {
        return Err(LauncherError::NoWine);
    }

    let mut cmd = Command::new(steam_run_path);

    // must have hl
    cmd.arg(&hlexe);

    if !gamemod.is_empty() {
        cmd.arg("-game").arg(gamemod);
    }

    if !extras.is_empty() {
        cmd.arg(extras);
    }

    let hl_root = Path::new(&hlexe).parent().unwrap();

    let library_path = env::var("LD_LIBRARY_PATH").unwrap_or("".to_owned());
    let library_path = format!("{}:{}", hl_root.display(), library_path);

    let mut preload = env::var("LD_PRELOAD").unwrap_or("".to_owned());

    if enable_bxt_rs {
        preload = format!("{}:{}", preload, bxt_rs);
    }

    if enable_bxt {
        preload = format!("{}:{}", preload, bxt);
    }

    cmd.env("LD_PRELOAD", preload);
    cmd.env("LD_LIBRARY_PATH", library_path);
    cmd.env("SteamEnv", "1");

    // must change to hl root for things to work, apparently
    cmd.current_dir(hl_root);

    let _ = cmd.spawn();

    Ok(())
}

const PATHS_TO_CHECK: &[&str] = &[
    "~/.steam/bin/steam-runtime/run.sh",
    "~/.var/app/com.valvesoftware.Steam/.local/share/Steam/ubuntu12_32/steam-runtime/run.sh",
    "~/.local/share/Steam/ubuntu12_32/steam-runtime/run.sh",
];

pub fn get_steam_run() -> Option<PathBuf> {
    let home = env::var("HOME").ok()?;

    for path_str in PATHS_TO_CHECK {
        let path_str = path_str.strip_prefix("~/").unwrap_or(*path_str);
        let path = PathBuf::from(&home).join(path_str);

        if path.exists() {
            return Some(path);
        }
    }

    None
}
