use alpm_lint_config::LintConfiguration;
use config::{Config, File};
use dirs::home_dir;
use serde::{Deserialize, Serialize};

use crate::Error;

/// The global configuration struct containing all package specific settings of the ALPM ecosystem.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PackageConfig {
    lint: LintConfiguration,
}

impl PackageConfig {
    /// Read the configuration from disk
    ///
    /// Follow a hierarchical approach with later configs potentially overwriting values from the
    /// earlier configuration files:
    ///
    /// - "/etc/alpm/package.toml"
    /// - "$XDG_CONFIG_DIR/alpm/package.toml" (Fallback to "~/.config/alpm/alpm.toml")
    /// - "./package.toml"
    pub fn new() -> Result<Self, Error> {
        let user_config_path = dirs::config_dir()
            .unwrap_or(home_dir().ok_or(Error::NoHomeDirectory)?.join(".config"))
            .join("alpm")
            .join("package.toml");

        let s = Config::builder()
            .add_source(File::with_name("/etc/alpm/package.toml").required(false))
            .add_source(File::from(user_config_path).required(false))
            .add_source(File::with_name("./package.toml").required(false))
            .build()?;

        Ok(s.try_deserialize()?)
    }
}
