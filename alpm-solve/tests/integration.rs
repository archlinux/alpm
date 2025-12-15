//! Integration tests for `alpm-solve`.

use std::{collections::HashSet, str::FromStr};

use alpm_db::desc::{DbDescFile, DbDescFileV1};
use alpm_solve::{ALPMDependencyProvider, Error, Solver};
use alpm_types::{
    Architecture,
    BuildDate,
    FullVersion,
    InstalledSize,
    License,
    Name,
    PackageBaseName,
    PackageDescription,
    PackageInstallReason,
    PackageRelation,
    PackageValidation,
    Packager,
    RelationOrSoname,
    RepositoryName,
    Url,
};
use insta::assert_snapshot;
use rstest::rstest;

fn create_test_package(
    name: &str,
    version: &str,
    depends: Vec<&str>,
    provides: Vec<&str>,
    conflicts: Vec<&str>,
) -> DbDescFile {
    DbDescFile::V1(DbDescFileV1 {
        name: Name::from_str(name).unwrap(),
        version: FullVersion::from_str(version).unwrap(),
        base: PackageBaseName::from_str(name).unwrap(),
        description: PackageDescription::from_str("Test package").unwrap(),
        url: Some(Url::from_str("https://example.org").unwrap()),
        arch: Architecture::from_str("x86_64").unwrap(),
        builddate: BuildDate::from_str("1733737242").unwrap(),
        installdate: BuildDate::from_str("1733737243").unwrap(),
        packager: Packager::from_str("Test User <test@example.org>").unwrap(),
        size: InstalledSize::from_str("123").unwrap(),
        groups: vec![],
        reason: PackageInstallReason::Explicit,
        license: vec![License::from_str("MIT").unwrap()],
        validation: vec![PackageValidation::from_str("none").unwrap()],
        replaces: vec![],
        depends: depends
            .into_iter()
            .map(|d| RelationOrSoname::from_str(d).unwrap())
            .collect(),
        optdepends: vec![],
        conflicts: conflicts
            .into_iter()
            .map(|d| PackageRelation::from_str(d).unwrap())
            .collect(),
        provides: provides
            .into_iter()
            .map(|d| RelationOrSoname::from_str(d).unwrap())
            .collect(),
    })
}

fn create_test_dep(
    name: &str,
    version: &str,
    depends: Vec<&str>,
    provides: Vec<&str>,
    conflicts: Vec<&str>,
) -> DbDescFile {
    DbDescFile::V1(DbDescFileV1 {
        name: Name::from_str(name).unwrap(),
        version: FullVersion::from_str(version).unwrap(),
        base: PackageBaseName::from_str(name).unwrap(),
        description: PackageDescription::from_str("Test package").unwrap(),
        url: Some(Url::from_str("https://example.org").unwrap()),
        arch: Architecture::from_str("x86_64").unwrap(),
        builddate: BuildDate::from_str("1733737242").unwrap(),
        installdate: BuildDate::from_str("1733737243").unwrap(),
        packager: Packager::from_str("Test User <test@example.org>").unwrap(),
        size: InstalledSize::from_str("123").unwrap(),
        groups: vec![],
        reason: PackageInstallReason::Depend,
        license: vec![License::from_str("MIT").unwrap()],
        validation: vec![PackageValidation::from_str("none").unwrap()],
        replaces: vec![],
        depends: depends
            .into_iter()
            .map(|d| RelationOrSoname::from_str(d).unwrap())
            .collect(),
        optdepends: vec![],
        conflicts: conflicts
            .into_iter()
            .map(|d| PackageRelation::from_str(d).unwrap())
            .collect(),
        provides: provides
            .into_iter()
            .map(|d| RelationOrSoname::from_str(d).unwrap())
            .collect(),
    })
}

macro_rules! assert_solution {
    ($solution:expr, $name:expr, $enforce:expr) => {
        let suffix = if $enforce { "full" } else { "partial" };
        match $solution {
            Ok(success) => {
                assert_snapshot!([$name, suffix].join("_"), success);
            }
            Err(Error::Unsatisfiable(fail)) => {
                assert_snapshot!([$name, suffix].join("_"), fail);
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    };
    ($solution:expr, $name:expr) => {
        match $solution {
            Ok(success) => {
                assert_snapshot!($name, success);
            }
            Err(Error::Unsatisfiable(fail)) => {
                assert_snapshot!($name, fail);
            }
            Err(e) => {
                panic!("Unexpected error: {}", e);
            }
        }
    };
}

#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package(
            "glibc",
            "2.39-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "14.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.3.1-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "openssl",
            "3.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libssl.so.3-64", "libcrypto.so.3-64"],
            vec![],
        ),
        create_test_package(
            "ncurses",
            "6.4-1",
            vec!["libc.so.6-64"],
            vec!["libncurses.so.6-64"],
            vec![],
        ),
        create_test_package(
            "readline",
            "8.2-1",
            vec!["libncurses.so.6-64"],
            vec!["libreadline.so.8-64"],
            vec![],
        ),
        create_test_package(
            "bash",
            "5.2.21-1",
            vec!["libreadline.so.8-64", "libfail.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package("coreutils", "9.4-1", vec!["libc.so.6-64"], vec![], vec![]),
        create_test_package(
            "epic-fail",
            "3-1",
            vec![],
            vec!["libfail.so.1-64"],
            vec!["coreutils>9"],
        ),
    ];

    let extra = vec![
        create_test_package(
            "sqlite",
            "3.45.0-1",
            vec!["libc.so.6-64"],
            vec!["libsqlite3.so.0-64"],
            vec![],
        ),
        create_test_package(
            "libxml2",
            "2.12.0-1",
            vec!["libc.so.6-64", "libz.so.1-64"],
            vec!["libxml2.so.2-64"],
            vec![],
        ),
        create_test_package(
            "libxslt",
            "1.1.39-1",
            vec!["libxml2.so.2-64"],
            vec!["libxslt.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python",
            "3.12.1-1",
            vec![
                "libssl.so.3-64",
                "libsqlite3.so.0-64",
                "libreadline.so.8-64",
            ],
            vec!["python=3.12", "python3"],
            vec![],
        ),
        create_test_package(
            "ruby",
            "3.3.0-1",
            vec!["libssl.so.3-64", "libreadline.so.8-64"],
            vec!["ruby=3.3"],
            vec![],
        ),
        create_test_package(
            "git",
            "2.43.0-1",
            vec!["libssl.so.3-64", "libz.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "newdep",
            "1-1",
            vec!["libssl.so.3-64", "libz.so.1-64"],
            vec!["libnewdep.so.1-64"],
            vec![],
        ),
    ];

    let custom = vec![
        create_test_package("python-pip", "24.0-1", vec!["python>=3.9"], vec![], vec![]),
        create_test_package(
            "python-virtualenv",
            "20.25.0-1",
            vec!["python>=3.8"],
            vec![],
            vec![],
        ),
        create_test_package("ruby-bundler", "2.5.0-1", vec!["ruby>=3.0"], vec![], vec![]),
        create_test_package(
            "neovim",
            "0.9.4-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.4-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.3.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
    ];

    let cache = vec![create_test_package(
        "epic-fail",
        "2-1",
        vec!["libnewdep.so.1-64"],
        vec!["libfail.so.1-64"],
        vec![],
    )];

    let system_state = vec![
        create_test_package(
            "glibc",
            "2.38-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "13.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.3.0-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "openssl",
            "3.1.0-1",
            vec!["libc.so.6-64"],
            vec!["libssl.so.3-64", "libcrypto.so.3-64"],
            vec![],
        ),
        create_test_package(
            "ncurses",
            "6.3-1",
            vec!["libc.so.6-64"],
            vec!["libncurses.so.6-64"],
            vec![],
        ),
        create_test_package(
            "readline",
            "8.1-1",
            vec!["libncurses.so.6-64"],
            vec!["libreadline.so.8-64"],
            vec![],
        ),
        create_test_package(
            "bash",
            "5.2.15-1",
            vec!["libreadline.so.8-64"],
            vec![],
            vec![],
        ),
        create_test_package("coreutils", "9.3-1", vec!["libc.so.6-64"], vec![], vec![]),
        create_test_package(
            "sqlite",
            "3.44.0-1",
            vec!["libc.so.6-64"],
            vec!["libsqlite3.so.0-64"],
            vec![],
        ),
        create_test_package(
            "libxml2",
            "2.11.0-1",
            vec!["libc.so.6-64", "libz.so.1-64"],
            vec!["libxml2.so.2-64"],
            vec![],
        ),
        create_test_package(
            "libxslt",
            "1.1.38-1",
            vec!["libxml2.so.2-64"],
            vec!["libxslt.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python",
            "3.11.0-1",
            vec![
                "libssl.so.3-64",
                "libsqlite3.so.0-64",
                "libreadline.so.8-64",
            ],
            vec!["python=3.11", "python3"],
            vec![],
        ),
        create_test_package(
            "ruby",
            "3.2.0-1",
            vec!["libssl.so.3-64", "libreadline.so.8-64"],
            vec!["ruby=3.2"],
            vec![],
        ),
        create_test_package(
            "git",
            "2.42.0-1",
            vec!["libssl.so.3-64", "libz.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package("python-pip", "23.0-1", vec!["python>=3.8"], vec![], vec![]),
        create_test_package(
            "python-virtualenv",
            "20.24.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package("ruby-bundler", "2.4.0-1", vec!["ruby>=2.7"], vec![], vec![]),
        create_test_package(
            "neovim",
            "0.9.4-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.3-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.2.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
        create_test_package("epic-fail", "1-1", vec![], vec!["libfail.so.1-64"], vec![]),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);

    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_package_repository(RepositoryName::from_str("extra").unwrap(), -1, extra);
    provider.add_package_repository(RepositoryName::from_str("custom").unwrap(), -2, custom);
    provider.add_package_cache(cache);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(solution, "test_upgrade", enforce_full);
}

/// Tests an edge case, where package provides a virtual component and conflicts with a package with
/// the same name.
#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_conflicts_with_provide(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package("rustup", "2-1", vec![], vec!["cargo"], vec!["cargo"]),
        create_test_package("cargo", "2-1", vec![], vec![], vec!["rustup"]),
    ];

    let system_state = vec![create_test_package(
        "rustup",
        "1-1",
        vec![],
        vec!["cargo"],
        vec!["cargo"],
    )];

    let mut provider = ALPMDependencyProvider::new(&system_state);

    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(
        solution,
        "test_upgrade_conflicts_with_provide",
        enforce_full
    );
}

/// Tests an edge case, where exact dependency has no pkgrel.
#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_depends_no_pkgrel(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package("a", "2-1", vec!["b=1"], vec![], vec![]),
        create_test_package("b", "1-1", vec![], vec![], vec![]),
    ];

    let system_state = vec![create_test_package("a", "1-1", vec![], vec![], vec![])];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(solution, "test_upgrade_depends_no_pkgrel", enforce_full);
}

/// Tests upgrade with conflicting dependencies.
#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_with_conflict(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package("a", "2-1", vec!["c"], vec![], vec![]),
        create_test_package("b", "2-1", vec![], vec![], vec![]),
        create_test_package("c", "1-1", vec![], vec![], vec!["b"]),
    ];

    let system_state = vec![
        create_test_package("a", "1-1", vec!["b"], vec![], vec![]),
        create_test_dep("b", "1-1", vec![], vec![], vec![]),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(solution, "test_upgrade_with_conflict", enforce_full);
}

#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_versioned_soname(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package("a", "2-1", vec!["libb.so>=14"], vec![], vec![]),
        create_test_package("b", "2-1", vec![], vec!["libb.so=15-64"], vec![]),
    ];

    let system_state = vec![
        create_test_package("a", "1-1", vec!["libb.so>=14"], vec![], vec![]),
        create_test_dep("b", "1-1", vec![], vec!["libb.so=15-64"], vec![]),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(solution, "test_upgrade_versioned_soname", enforce_full);
}

#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_does_not_switch_providers(#[case] enforce_full: bool) {
    let core = vec![
        create_test_package("a", "2-1", vec!["dep-impl"], vec![], vec![]),
        create_test_package(
            "dep-b",
            "2-1",
            vec!["e"],
            vec!["dep-impl"],
            vec!["dep-c", "dep-d"],
        ),
        create_test_package(
            "dep-c",
            "2-1",
            vec!["e"],
            vec!["dep-impl"],
            vec!["dep-b", "dep-d"],
        ),
        create_test_package(
            "dep-d",
            "2-1",
            vec!["e"],
            vec!["dep-impl"],
            vec!["dep-b", "dep-c"],
        ),
        create_test_package("e", "2-1", vec![], vec![], vec!["dep-d", "dep-b"]),
    ];

    let system_state = vec![
        create_test_package("a", "1-1", vec!["dep-impl"], vec![], vec![]),
        create_test_dep(
            "dep-c",
            "1-1",
            vec![],
            vec!["dep-impl"],
            vec!["dep-b", "dep-d"],
        ),
        create_test_dep("e", "1-1", vec![], vec![], vec!["dep-d", "dep-b"]),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(
        solution,
        "test_upgrade_does_not_switch_providers",
        enforce_full
    );
}

#[rstest]
#[case::full(true)]
#[case::partial(false)]
fn test_upgrade_multilib(#[case] enforce_full: bool) {
    let core = vec![create_test_package(
        "a",
        "2-1",
        vec![],
        vec!["liba.so=0-64"],
        vec![],
    )];
    let multilib = vec![create_test_package(
        "lib32-a",
        "2-1",
        vec![],
        vec!["liba.so=0-32"],
        vec![],
    )];

    let system_state = vec![
        create_test_package("a", "1-1", vec![], vec!["liba.so=0-64"], vec![]),
        create_test_package("lib32-a", "1-1", vec![], vec!["liba.so=0-32"], vec![]),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), 0, core);
    provider.add_package_repository(RepositoryName::from_str("core").unwrap(), -1, multilib);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();

    let solution = solver.upgrade(system_state, HashSet::default(), enforce_full);

    assert_solution!(solution, "test_upgrade_multilib", enforce_full);
}

#[test]
fn test_downgrade_success() {
    let cache = vec![
        create_test_package("epic-fail", "2-1", vec![], vec!["libfail.so.1-64"], vec![]),
        create_test_package(
            "glibc",
            "2.37-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "12.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python-virtualenv",
            "20.23.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package(
            "ruby-bundler",
            "2.4.0-1",
            vec!["ruby>=2.7", "libfail.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "neovim",
            "0.9.3-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.2-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.1.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
    ];

    let system_state = vec![
        create_test_package(
            "glibc",
            "2.38-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "13.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.3.0-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "openssl",
            "3.1.0-1",
            vec!["libc.so.6-64"],
            vec!["libssl.so.3-64", "libcrypto.so.3-64"],
            vec![],
        ),
        create_test_package(
            "ncurses",
            "6.3-1",
            vec!["libc.so.6-64"],
            vec!["libncurses.so.6-64"],
            vec![],
        ),
        create_test_package(
            "readline",
            "8.1-1",
            vec!["libncurses.so.6-64"],
            vec!["libreadline.so.8-64"],
            vec![],
        ),
        create_test_package(
            "bash",
            "5.2.15-1",
            vec!["libreadline.so.8-64"],
            vec![],
            vec![],
        ),
        create_test_package("coreutils", "9.3-1", vec!["libc.so.6-64"], vec![], vec![]),
        create_test_package(
            "sqlite",
            "3.44.0-1",
            vec!["libc.so.6-64"],
            vec!["libsqlite3.so.0-64"],
            vec![],
        ),
        create_test_package(
            "libxml2",
            "2.11.0-1",
            vec!["libc.so.6-64", "libz.so.1-64"],
            vec!["libxml2.so.2-64"],
            vec![],
        ),
        create_test_package(
            "libxslt",
            "1.1.38-1",
            vec!["libxml2.so.2-64"],
            vec!["libxslt.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python",
            "3.11.0-1",
            vec![
                "libssl.so.3-64",
                "libsqlite3.so.0-64",
                "libreadline.so.8-64",
            ],
            vec!["python=3.11", "python3"],
            vec![],
        ),
        create_test_package(
            "ruby",
            "3.2.0-1",
            vec!["libssl.so.3-64", "libreadline.so.8-64"],
            vec!["ruby=3.2"],
            vec![],
        ),
        create_test_package(
            "git",
            "2.42.0-1",
            vec!["libssl.so.3-64", "libz.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package("python-pip", "23.0-1", vec!["python>=3.8"], vec![], vec![]),
        create_test_package(
            "python-virtualenv",
            "20.24.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package("ruby-bundler", "2.4.0-1", vec!["ruby>=2.7"], vec![], vec![]),
        create_test_package(
            "neovim",
            "0.9.4-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.3-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.2.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "epic-fail",
            "4-1",
            vec![],
            vec!["libfail.so.1-64"],
            vec!["neovim<=0.9.3-1"],
        ),
    ];

    let downgrade_set = vec![
        create_test_package(
            "python-virtualenv",
            "20.23.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package(
            "ruby-bundler",
            "2.4.0-1",
            vec!["ruby>=2.7", "libfail.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "neovim",
            "0.9.3-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.2-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.1.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_cache(cache);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();
    let solution = solver.downgrade(system_state, downgrade_set);

    assert_solution!(solution, "test_downgrade_success");
}

#[test]
fn test_downgrade_fail() {
    let cache = vec![
        create_test_package(
            "epic-fail",
            "2-1",
            vec!["libnewdep.so.1-64"],
            vec!["libfail.so.1-64"],
            vec![],
        ),
        create_test_package(
            "glibc",
            "2.37-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "12.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python-virtualenv",
            "20.23.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package(
            "ruby-bundler",
            "2.4.0-1",
            vec!["ruby>=2.7", "libfail.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "neovim",
            "0.9.3-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.2-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.1.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
    ];

    let system_state = vec![
        create_test_package(
            "glibc",
            "2.38-1",
            vec![],
            vec!["libc.so.6-64", "ld-linux-x86-64.so.2-64"],
            vec![],
        ),
        create_test_package(
            "gcc-libs",
            "13.2.0-1",
            vec!["libc.so.6-64"],
            vec!["libgcc_s.so.1-64", "libstdc++.so.6-64"],
            vec![],
        ),
        create_test_package(
            "zlib",
            "1.3.0-1",
            vec!["libc.so.6-64"],
            vec!["libz.so.1-64"],
            vec![],
        ),
        create_test_package(
            "openssl",
            "3.1.0-1",
            vec!["libc.so.6-64"],
            vec!["libssl.so.3-64", "libcrypto.so.3-64"],
            vec![],
        ),
        create_test_package(
            "ncurses",
            "6.3-1",
            vec!["libc.so.6-64"],
            vec!["libncurses.so.6-64"],
            vec![],
        ),
        create_test_package(
            "readline",
            "8.1-1",
            vec!["libncurses.so.6-64"],
            vec!["libreadline.so.8-64"],
            vec![],
        ),
        create_test_package(
            "bash",
            "5.2.15-1",
            vec!["libreadline.so.8-64"],
            vec![],
            vec![],
        ),
        create_test_package("coreutils", "9.3-1", vec!["libc.so.6-64"], vec![], vec![]),
        create_test_package(
            "sqlite",
            "3.44.0-1",
            vec!["libc.so.6-64"],
            vec!["libsqlite3.so.0-64"],
            vec![],
        ),
        create_test_package(
            "libxml2",
            "2.11.0-1",
            vec!["libc.so.6-64", "libz.so.1-64"],
            vec!["libxml2.so.2-64"],
            vec![],
        ),
        create_test_package(
            "libxslt",
            "1.1.38-1",
            vec!["libxml2.so.2-64"],
            vec!["libxslt.so.1-64"],
            vec![],
        ),
        create_test_package(
            "python",
            "3.11.0-1",
            vec![
                "libssl.so.3-64",
                "libsqlite3.so.0-64",
                "libreadline.so.8-64",
            ],
            vec!["python=3.11", "python3"],
            vec![],
        ),
        create_test_package(
            "ruby",
            "3.2.0-1",
            vec!["libssl.so.3-64", "libreadline.so.8-64"],
            vec!["ruby=3.2"],
            vec![],
        ),
        create_test_package(
            "git",
            "2.42.0-1",
            vec!["libssl.so.3-64", "libz.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package("python-pip", "23.0-1", vec!["python>=3.8"], vec![], vec![]),
        create_test_package(
            "python-virtualenv",
            "20.24.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package("ruby-bundler", "2.4.0-1", vec!["ruby>=2.7"], vec![], vec![]),
        create_test_package(
            "neovim",
            "0.9.4-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.3-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.2.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "epic-fail",
            "4-1",
            vec![],
            vec!["libfail.so.1-64"],
            vec!["neovim<=0.9.3-1"],
        ),
    ];

    let downgrade_set = vec![
        create_test_package(
            "python-virtualenv",
            "20.23.0-1",
            vec!["python>=3.7"],
            vec![],
            vec![],
        ),
        create_test_package(
            "ruby-bundler",
            "2.4.0-1",
            vec!["ruby>=2.7", "libfail.so.1-64"],
            vec![],
            vec![],
        ),
        create_test_package(
            "neovim",
            "0.9.3-1",
            vec!["libstdc++.so.6-64"],
            vec!["vi"],
            vec!["vim"],
        ),
        create_test_package("tmux", "3.2-1", vec!["libncurses.so.6-64"], vec![], vec![]),
        create_test_package(
            "htop",
            "3.1.0-1",
            vec!["libncurses.so.6-64"],
            vec![],
            vec![],
        ),
    ];

    let mut provider = ALPMDependencyProvider::new(&system_state);
    provider.add_package_cache(cache);
    provider.add_installed(system_state.clone());

    let mut solver: Solver = provider.into();
    let solution = solver.downgrade(system_state, downgrade_set);

    assert_solution!(solution, "test_downgrade_fail");
}
