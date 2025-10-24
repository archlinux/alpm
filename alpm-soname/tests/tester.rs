#!/usr/bin/env rust-script
//! This test script verifies that shared object dependencies and provisions
//! are correctly identified in packages.
//!
//! It does so by creating a simple C project with Meson that builds a shared library
//! and a binary that depends on it. The test then creates packages for both the library
//! and the binary, and checks that the binary's dependencies include the library's soname,
//! and that the library's provisions include its own soname.
//!
//! It is designed to be run as a standalone program via [`rust-script`], taking an optional
//! JSON-encoded configuration as a command line argument:
//!
//! ```sh
//! ./tester.rs
//!
//! # or with a custom configuration:
//!
//! ./tester.rs '{"libname":"sotest", ...}'
//! ```
//!
//! [`rust-script`]: https://github.com/fornwall/rust-script
//!
//! ```cargo
//! [dependencies]
//! assert_cmd = "2"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//! tempfile = "3"
//! testresult = "0.4"
//! rstest = "0.17"
//! alpm-compress = { path = "../../alpm-compress" }
//! alpm-package = { path = "../../alpm-package" }
//! alpm-mtree = { path = "../../alpm-mtree" }
//! alpm-types = { path = "../../alpm-types" }
//! alpm-soname = { path = "../../alpm-soname" }
//! ```

use std::{env, path::PathBuf, str::FromStr};

use alpm_types::SonameLookupDirectory;
use serde::Serialize;
use testresult::{TestError, TestResult};

mod shared;

use shared::*;

/// Output structure when run as a standalone script.
#[derive(Serialize)]
struct ScriptOutput {
    /// Path to the created library package.
    lib_package_path: PathBuf,
    /// Path to the created binary package.
    bin_package_path: PathBuf,
}

/// Entry point when run as a script (e.g. ./tester.rs).
///
/// Takes an optional JSON-encoded configuration as a command line argument.
/// If no argument is provided, a default configuration is used.
fn main() -> TestResult {
    let args: Vec<String> = env::args().collect();
    let current_dir = env::current_dir()?;
    let test_files_dir = current_dir.join("test_files");
    if !test_files_dir.exists() {
        return Err(TestError::from(format!(
            "test_files directory not found: {}",
            test_files_dir.display()
        )));
    }

    let output_dir = env::var("OUTPUT_DIR").unwrap_or_else(|_| "output".to_string());
    let path = current_dir.join(output_dir);

    let cfg = if let Some(arg) = args.get(1) {
        serde_json::from_str(arg)?
    } else {
        SotestConfig {
            libname: "example".to_string(),
            lookup: SonameLookupDirectory::from_str("lib:/usr/lib").unwrap(),
            dep: "lib:libexample.so.1".parse()?,
            expect_dep: None,
            expect_provide: None,
        }
    };

    setup_lib(&cfg, &path, &test_files_dir)?;
    let lib = create_lib_package(&path, &cfg)?;
    let bin = create_bin_package(&path, &cfg)?;

    let output = ScriptOutput {
        lib_package_path: lib.to_path_buf(),
        bin_package_path: bin.to_path_buf(),
    };

    println!("{}", serde_json::to_string(&output)?);

    Ok(())
}
