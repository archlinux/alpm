use std::{fs::File, io::Read, path::Path};

use serde::{Deserialize, Serialize};

use crate::{Error, LintGroup, LintRuleConfiguration};

/// Configuration options for linting.
///
/// The options allow to
///
/// - configure the general lint rule behavior,
/// - explicitly enable or disable individual lint rules,
/// - and enable non-default lint groups.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct LintConfiguration {
    /// All options that can be used to configure various lint rules.
    pub options: LintRuleConfiguration,
    /// All non-default groups that are additionally enabled.
    pub groups: Vec<LintGroup>,
    /// A list of lint rules that are explicitly disabled.
    pub disabled_rules: Vec<String>,
    /// A list of lint rules that are explicitly enabled.
    pub enabled_rules: Vec<String>,
}

impl LintConfiguration {
    /// Loads a [`LintConfiguration`] from a TOML configuration file.
    ///
    /// Reads the file at the specified path and parses it as TOML to create a configuration
    /// instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::io::Write;
    /// # use tempfile::tempdir;
    /// # use testresult::TestResult;
    /// use alpm_lint_config::LintConfiguration;
    ///
    /// # fn main() -> TestResult {
    /// # let temp_dir = tempdir()?;
    /// # let config_path = temp_dir.path().join("lint_config.toml");
    /// # let mut config_file = std::fs::File::create(&config_path)?;
    /// # let toml_content = toml::to_string(&LintConfiguration::default())?;
    /// # write!(config_file, "{}", toml_content)?;
    /// #
    /// // Load configuration from a TOML file containing the default configuration.
    /// let config = LintConfiguration::from_path(&config_path)?;
    ///
    /// // The configuration is now available for use
    /// assert_eq!(config, LintConfiguration::default());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - the file at `path` cannot be opened for reading,
    /// - the file contents cannot be read,
    /// - or the file contents cannot be parsed as valid TOML.
    pub fn from_path(path: &Path) -> Result<Self, Error> {
        let mut file = File::open(path).map_err(|source| Error::IoPath {
            path: path.to_path_buf(),
            context: "opening the config for reading",
            source,
        })?;
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .map_err(|source| Error::IoPath {
                path: path.to_path_buf(),
                context: "reading config data",
                source,
            })?;

        Ok(toml::from_str(&buf)?)
    }
}
