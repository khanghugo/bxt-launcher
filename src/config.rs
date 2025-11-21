use std::{
    env,
    fs::OpenOptions,
    hash::{DefaultHasher, Hash, Hasher},
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
    // unused features
    pub use_wine: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConfigWithProfiles {
    pub current_profile: usize,
    pub configs: Vec<Config>,
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
            use_wine: false,
        }
    }
}

impl Default for ConfigWithProfiles {
    fn default() -> Self {
        Self {
            current_profile: 0,
            configs: vec![Config::default(); 4],
        }
    }
}

impl ConfigWithProfiles {
    fn parse_from_file(path: impl AsRef<Path> + Into<PathBuf>) -> Result<Self, LauncherError> {
        let path = path.as_ref();

        let mut file = OpenOptions::new().read(true).open(path.as_os_str())?;
        let mut buffer = String::new();

        file.read_to_string(&mut buffer)?;

        let config: ConfigWithProfiles = toml::from_str(&buffer)?;

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

        let res = Self::parse_from_file(&path);

        // if cannot parse the file, then make a backup of the older file before we overwrite it
        // it happens because i mess up the format and i don't want people to lose their data
        let Err(LauncherError::TomlParsingError { .. }) = res else {
            return res;
        };

        {
            let mut hasher = DefaultHasher::new();
            std::time::SystemTime::now().hash(&mut hasher);
            let hash_res = hasher.finish();

            let config_name = format!("{}_{}", hash_res, CONFIG_FILE_NAME);
            std::fs::rename(&path, path.with_file_name(config_name))?;
        }

        return res;
    }

    pub fn write_to_default(&self) -> Result<(), LauncherError> {
        let path = match env::current_exe() {
            Ok(path) => path.parent().unwrap().join(CONFIG_FILE_NAME),
            Err(_) => PathBuf::from(CONFIG_FILE_NAME),
        };

        self.write_to_file(path)
    }
}
