use std::{
    env,
    fs::OpenOptions,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct Config {
    pub injector: String,
    pub hlexe: String,
    pub gamemod: String,
    pub extras: String,
}

const CONFIG_FILE_NAME: &str = "bxt_launcher.toml";

impl Default for Config {
    fn default() -> Self {
        Self {
            injector: Default::default(),
            hlexe: Default::default(),
            gamemod: "cstrike".into(),
            extras: Default::default(),
        }
    }
}

impl Config {
    fn parse_from_file(path: impl AsRef<Path> + Into<PathBuf>) -> eyre::Result<Self> {
        let path = path.as_ref();

        let mut file = OpenOptions::new().read(true).open(path.as_os_str())?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;

        let config: Config = toml::from_str(&buffer)?;

        Ok(config)
    }

    fn write_to_file(&self, path: impl AsRef<Path> + Into<PathBuf>) -> eyre::Result<()> {
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

    pub fn load_from_default() -> eyre::Result<Self> {
        let path = match env::current_exe() {
            Ok(path) => path.parent().unwrap().join(CONFIG_FILE_NAME),
            Err(_) => PathBuf::from(CONFIG_FILE_NAME),
        };

        Self::parse_from_file(path)
    }

    pub fn write_to_default(&self) -> eyre::Result<()> {
        let path = match env::current_exe() {
            Ok(path) => path.parent().unwrap().join(CONFIG_FILE_NAME),
            Err(_) => PathBuf::from(CONFIG_FILE_NAME),
        };

        self.write_to_file(path)
    }
}
