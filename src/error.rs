use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum LauncherError {
    #[error("No given Half-Life executable path")]
    NoHLExe,
    #[cfg(windows)]
    #[error("Windows error: {source}")]
    WindowsAPI {
        #[source]
        source: windows::core::Error,
    },
    #[cfg(windows)]
    #[error("Inject error: {source}")]
    InjectError {
        #[source]
        source: dll_syringe::error::InjectError,
    },
    #[error("IO error: {source}")]
    IOError {
        #[source]
        source: std::io::Error,
    },
    #[cfg(windows)]
    #[error("Injection fails: {reason}")]
    InjectionFailed { reason: String },
    #[error("Config parsing error: {source}")]
    TomlParsingError { source: toml::de::Error },
    #[error("Config writing error: {source}")]
    TomlWritingError { source: toml::ser::Error },
    #[error("File does not exist: {path}")]
    FileDoesNotExist { path: PathBuf },
}

#[cfg(windows)]
impl From<windows::core::Error> for LauncherError {
    fn from(value: windows::core::Error) -> Self {
        LauncherError::WindowsAPI { source: value }
    }
}

impl From<std::io::Error> for LauncherError {
    fn from(value: std::io::Error) -> Self {
        LauncherError::IOError { source: value }
    }
}

#[cfg(windows)]
impl From<dll_syringe::error::InjectError> for LauncherError {
    fn from(value: dll_syringe::error::InjectError) -> Self {
        LauncherError::InjectError { source: value }
    }
}

impl From<toml::de::Error> for LauncherError {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlParsingError { source: value }
    }
}

impl From<toml::ser::Error> for LauncherError {
    fn from(value: toml::ser::Error) -> Self {
        Self::TomlWritingError { source: value }
    }
}
