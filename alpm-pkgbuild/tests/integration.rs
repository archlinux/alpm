//! This test file contains basic tests to ensure that the alpm-pkgbuild CLI behaves as expected.
use std::{fs::File, io::Write};

use assert_cmd::Command;
use testresult::TestResult;

pub const VALID_PKGBUILD: &str = r#"pkgname=(example)
pkgver=0.1.0
pkgrel=1
epoch=1
arch=(x86_64 aarch64)

pkgdesc="A example with all pkgbase properties set."
url="https://archlinux.org/"
license=(MIT)
changelog=changelog.stub
install=install.sh.stub
groups=(
    group
    group_2
)
backup=(etc/pacman.conf)
options=("!lto")

depends=(default_dep)
optdepends=(default_optdep)
provides=(default_provides)
conflicts=(default_conflict)
replaces=(default_replaces)

# x86_64 specific stuff
# This should show up in the test
depends_x86_64=(arch_default_dep)
optdepends_x86_64=(arch_default_optdep)
provides_x86_64=(arch_default_provides)
conflicts_x86_64=(arch_default_conflict)
replaces_x86_64=(arch_default_replaces)

package_example() {
    echo "Building something"
}
"#;

mod srcinfo_run_bridge {
    use tempfile::tempdir;

    use super::*;

    /// Execute the `run_bridge` subcommand, which is used to generate the intermediate format via
    /// the bridge shell script.
    #[test]
    fn run_bridge() -> TestResult {
        // Write the PKGBUILD to a temporary directory
        let tempdir = tempdir()?;
        let path = tempdir.path().join("PKGBUILD");
        let mut file = File::create_new(&path)?;
        file.write_all(VALID_PKGBUILD.as_bytes())?;

        // Call the bridge on the that PKGBUILD file.
        let mut cmd = Command::cargo_bin("alpm-pkgbuild")?;
        cmd.args(vec![
            "srcinfo".into(),
            "run-bridge".into(),
            path.to_string_lossy().to_string(),
        ]);

        // Make sure the command was successful and get the output.
        let output = cmd.assert().success();
        let output = String::from_utf8(output.get_output().stdout.to_vec())?;
        println!("Output:\n{output}");

        assert!(
            output.contains(r#"VAR GLOBAL ARRAY arch "x86_64" "aarch64""#),
            "Got unexpected output:\n{output}"
        );

        Ok(())
    }
}
