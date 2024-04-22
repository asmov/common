use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct InstallPathSuffix {
    pub config_dir: &'static str,
    pub data_dir: &'static str
}


/// Provides common directory paths depending on how the application is installed.
#[derive(Debug)]
pub enum InstallPath {
    /// Application is running out of its project directory.
    /// Configuration and data live in the project's ".local/config" and ".local/share" directories, respectively.
    Project(InstallPathSuffix),
    /// Application is running from the /usr/local.
    /// Configuration lives in /usr/local/etc and data lives in /var/local.
    SystemLocal(InstallPathSuffix),
    /// Application is running from the /usr.
    /// Configuration lives in /etc and data lives in /var.
    SystemGlobal(InstallPathSuffix),
    /// Application is running from a non-standard location.
    /// Configuration and data live in the user's home per XDG specs: ~/.config and ~/.local/share respectively.
    Home(InstallPathSuffix),
    /// Application is running from a non-standard location.
    /// Configuration and data live in the CWD's ".local/config" and ".local/share" directories, respectively.
    Working(InstallPathSuffix),
}

impl InstallPath {
    const USR_LOCAL_PREFIX: &'static str = "/usr/local/";
    const USR_PREFIX: &'static str = "/usr/";
    const USR_LOCAL: &'static str = "/usr/local";
    const ROOT: &'static str = "/";
    const ETC: &'static str = "etc";
    const VAR_LOCAL: &'static str = "/var/local";
    const VAR: &'static str = "/var";
    const DOT_CONFIG: &'static str = ".config";
    const DOT_LOCAL: &'static str = ".local";
    const CONFIG: &'static str = "config";
    const SHARE: &'static str = "share";
    const SECRETS: &'static str = "secrets";

    /// The directory containing the currently running executable.
    pub fn executable_dir() -> PathBuf {
        std::env::current_exe().unwrap()
            .parent().unwrap()
            .to_path_buf()
    }

    /// The current working directory.
    pub fn working_dir() -> PathBuf {
        std::env::current_dir().unwrap()
    }

    /// Determine everything from the currently running executable's installation path.
    pub fn from_executable(suffix: InstallPathSuffix) -> InstallPath {
        Self::from_dir(suffix, &Self::executable_dir())
    }

    /// The project directory containing the Cargo.toml file.
    pub fn project_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent().unwrap()
            .parent().unwrap()
            .to_path_buf()
    }

    /// The user environment's $HOME directory.
    pub fn home_dir() -> PathBuf {
        PathBuf::from(env!("HOME"))
    }

    /// Determine everything from the specified directory path.
    pub fn from_dir(suffix: InstallPathSuffix, dir: &Path) -> InstallPath {
        if std::env::current_dir().unwrap().starts_with(Self::project_dir()) {
            // overrides anything provided by dir parameter
            Self::Project(suffix) 
        } else if dir.starts_with(Self::USR_LOCAL_PREFIX) {
            Self::SystemLocal(suffix)
        } else if dir.starts_with(Self::USR_PREFIX) {
            Self::SystemGlobal(suffix)
        } else {
            if let Some(home_env) = option_env!("HOME") {
                if Path::new(home_env).join(Self::DOT_CONFIG).join(suffix.config_dir).exists() {
                    Self::Home(suffix)
                } else {
                    Self::Working(suffix)
                }
            } else {
                Self::Working(suffix)
            }
        }
    }

    pub fn config_dir(&self) -> PathBuf {
        match self {
            Self::Project(suffix) => Self::project_dir().join(Self::DOT_LOCAL).join(Self::CONFIG).join(suffix.config_dir),
            Self::SystemLocal(suffix) => Path::new(Self::USR_LOCAL).join(Self::ETC).join(suffix.config_dir),
            Self::SystemGlobal(suffix) => Path::new(Self::ROOT).join(Self::ETC).join(suffix.config_dir),
            Self::Home(suffix) => Self::home_dir().join(Self::DOT_CONFIG).join(suffix.config_dir),
            Self::Working(suffix) => Self::working_dir().join(Self::DOT_LOCAL).join(Self::CONFIG).join(suffix.config_dir),
        }
    }

    /// The directory containing the application's secrets; passwords, private keys, etc.
    pub fn config_secrets_dir(&self) -> PathBuf {
        self.config_dir().join(Self::SECRETS)
    }

    pub fn data_dir(&self) -> PathBuf {
        match self {
            Self::Project(suffix) => Self::project_dir().join(Self::DOT_LOCAL).join(Self::SHARE).join(suffix.data_dir),
            Self::SystemLocal(suffix) => PathBuf::from(Self::VAR_LOCAL).join(suffix.data_dir),
            Self::SystemGlobal(suffix) => PathBuf::from(Self::VAR).join(suffix.data_dir),
            Self::Home(suffix) => Self::home_dir().join(Self::DOT_LOCAL).join(Self::SHARE).join(suffix.data_dir),
            Self::Working(suffix) => Self::working_dir().join(Self::DOT_LOCAL).join(Self::SHARE).join(suffix.data_dir),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SUFFIX: InstallPathSuffix = InstallPathSuffix {
        config_dir: "asmov/test",
        data_dir: "asmov/test",
    };

    #[test]
    fn test_install_path() {
        assert_eq!("/usr/local/etc/asmov/test", InstallPath::SystemLocal(SUFFIX).config_dir().to_str().unwrap());
        assert_eq!("/etc/asmov/test", InstallPath::SystemGlobal(SUFFIX).config_dir().to_str().unwrap());
    }
}