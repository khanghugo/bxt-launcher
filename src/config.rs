use std::{
    env,
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::error::LauncherError;

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub hlexe: String,
    pub bxt: String,
    pub enable_bxt: bool,
    pub bxt_rs: String,
    pub enable_bxt_rs: bool,
    pub gamemod: String,
    pub extras: String,
}

const CONFIG_FILE_NAME: &str = "bxt_launcher.toml";

impl Default for Config {
    fn default() -> Self {
        Self {
            hlexe: String::new(),
            bxt: String::new(),
            bxt_rs: String::new(),
            gamemod: "valve".to_owned(),
            extras: String::new(),
            enable_bxt: true,
            enable_bxt_rs: true,
        }
    }
}

impl Config {
    fn parse_from_file(path: impl AsRef<Path> + Into<PathBuf>) -> Result<Self, LauncherError> {
        let path = path.as_ref();

        let mut file = OpenOptions::new().read(true).open(path.as_os_str())?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;

        let config: Config = toml::from_str(&buffer)?;

        Ok(config)
    }

    fn write_to_file(&self, path: impl AsRef<Path> + Into<PathBuf>) -> Result<(), LauncherError> {
        let path = path.as_ref();

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        let res = toml::to_string(&self)?;

        file.write_all(res.as_bytes())?;
        file.flush()?;

        Ok(())
    }

    pub fn load_from_default() -> Result<Self, LauncherError> {
        let path = match env::current_exe() {
            Ok(path) => path.parent().unwrap().join(CONFIG_FILE_NAME),
            Err(_) => PathBuf::from(CONFIG_FILE_NAME),
        };

        Self::parse_from_file(path)
    }

    pub fn write_to_default(&self) -> Result<(), LauncherError> {
        let path = match env::current_exe() {
            Ok(path) => path.parent().unwrap().join(CONFIG_FILE_NAME),
            Err(_) => PathBuf::from(CONFIG_FILE_NAME),
        };

        self.write_to_file(path)
    }
}
