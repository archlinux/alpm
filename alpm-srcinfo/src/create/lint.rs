use crate::create::types::ArchVec;
use std::{collections::HashSet, iter, path::Path};

use crate::create::pkgbuild::{ArchVecs, Function, Pkgbuild, Source};
use crate::create::raw::{LintKind, RawPkgbuild, Value, Variable};

static PKGBUILD_PACKAGE_OVERRIDES: &[&str] = &[
    "pkgdesc",
    "arch",
    "url",
    "license",
    "groups",
    "depends",
    "optdepends",
    "provides",
    "conflicts",
    "replaces",
    "backup",
    "options",
    "install",
    "changelog",
];

static PKGBUILD_ARCH_ARRAYS: &[&str] = &[
    "checkdepends",
    "conflicts",
    "depends",
    "makedepends",
    "optdepends",
    "provides",
    "replaces",
    "source",
    "cksums",
    "md5sums",
    "sha1sums",
    "sha224sums",
    "sha256sums",
    "sha384sums",
    "sha512sums",
    "b2sums",
];

impl ArchVecs<String> {
    pub(crate) fn lint_merge(&mut self, other: Variable, lints: &mut Vec<LintKind>) {
        if let Err(e) = self.merge(other) {
            lints.push(e);
        }
    }
}

impl Variable {
    pub(crate) fn lint_no_arch(&self, lints: &mut Vec<LintKind>) {
        if let Err(e) = self.assert_no_arch() {
            lints.push(e);
        }
    }

    pub(crate) fn lint_string(self, lints: &mut Vec<LintKind>) -> String {
        match self.get_string() {
            Ok(s) => s,
            Err(e) => {
                lints.push(e);
                String::new()
            }
        }
    }

    pub(crate) fn lint_array(self, lints: &mut Vec<LintKind>) -> Vec<String> {
        match self.get_array() {
            Ok(s) => s,
            Err(e) => {
                lints.push(e);
                Vec::new()
            }
        }
    }

    pub(crate) fn lint_arch_array(self, lints: &mut Vec<LintKind>) -> ArchVec<String> {
        match self.get_arch_array() {
            Ok(s) => s,
            Err(e) => {
                lints.push(e);
                Default::default()
            }
        }
    }
}

impl RawPkgbuild {
    pub(crate) fn lint(&self, lints: &mut Vec<LintKind>) {
        self.lint_arch_specific(lints);
        self.lint_package_function_variables(lints);
        lint_arrays(self.all_variables(), lints);
        lint_newline(self.all_variables(), lints);
    }

    fn lint_arch_specific(&self, lints: &mut Vec<LintKind>) {
        let arch_arrays: HashSet<&str> = PKGBUILD_ARCH_ARRAYS.iter().copied().collect();

        for var in self.all_variables() {
            if let Some(arch) = &var.arch {
                if !arch_arrays.contains(var.name.as_str()) {
                    lints.push(LintKind::CantBeArchitectureSpecific(
                        var.name.clone(),
                        var.name_arch(),
                    ))
                }

                if arch == "any" {
                    lints.push(LintKind::CantBeArchitectureSpecificAny);
                }
            }
        }
    }

    fn lint_package_function_variables(&self, lints: &mut Vec<LintKind>) {
        let allowed_in_function: HashSet<&str> =
            PKGBUILD_PACKAGE_OVERRIDES.iter().copied().collect();

        for func in &self.function_variables {
            for var in &func.variables {
                if !allowed_in_function.contains(var.name.as_str()) {
                    lints.push(LintKind::VariableCantBeInPackageFunction(var.name_arch()));
                }
            }
        }
    }

    fn all_variables(&self) -> impl Iterator<Item = &Variable> {
        self.variables
            .iter()
            .chain(self.function_variables.iter().flat_map(|p| &p.variables))
    }
}

fn lint_newline<'a, I: Iterator<Item = &'a Variable>>(iter: I, lints: &mut Vec<LintKind>) {
    for var in iter {
        match &var.value {
            Value::Array(a) => {
                if a.iter().any(|v| v.contains('\n')) {
                    lints.push(LintKind::VariabeContainsNewlines(var.name.clone()))
                }
            }
            Value::String(s) => {
                if s.contains('\n') {
                    lints.push(LintKind::VariabeContainsNewlines(var.name.clone()))
                }
            }
            _ => (),
        }
    }
}

fn lint_arrays<'a, I: Iterator<Item = &'a Variable>>(iter: I, lints: &mut Vec<LintKind>) {
    for var in iter {
        if let Value::Array(a) = &var.value {
            if a.iter().any(|v| v.is_empty()) {
                lints.push(LintKind::VariabeContainsEmptyString(var.name.clone()))
            }
        }
    }
}

impl Pkgbuild {
    pub(crate) fn lint(&self, lints: &mut Vec<LintKind>) {
        self.lint_pkgbase(lints);
        self.lint_arch(lints);

        self.lint_epoch(lints);
        self.lint_pkgrel(lints);
        self.lint_pkgver(lints);

        self.lint_depends(lints);
        self.lint_makedepends(lints);
        self.lint_optdepends(lints);
        self.lint_checkdepends(lints);

        self.lint_conflicts(lints);
        self.lint_provides(lints);
        self.lint_replaces(lints);
        self.lint_package_function(lints);

        self.lint_backup(lints);
        self.lint_changelog(lints);
        self.lint_sources(lints);
    }

    fn lint_pkgbase(&self, lints: &mut Vec<LintKind>) {
        check_pkgname(&self.pkgbase, "pkgbase", lints)
    }

    fn lint_package_function(&self, lints: &mut Vec<LintKind>) {
        if self.packages.len() == 1 {
            if self.package_functions.iter().any(|f| f == "package")
                && self
                    .package_functions
                    .iter()
                    .any(|f| f.starts_with("package_"))
            {
                lints.push(LintKind::ConflictingPackageFunctions);
            }

            if self.has_function(Function::Build) && !self.has_function(Function::Package) {
                lints.push(LintKind::MissingPackageFunction(
                    self.packages[0].pkgname.to_string(),
                ));
            }
        } else {
            if self.package_functions.iter().any(|p| p == "package") {
                lints.push(LintKind::WrongPackgeFunctionFormat);
            }
            for pkg in self.packages() {
                if !self
                    .package_functions
                    .iter()
                    .any(|f| f.trim_start_matches("package_") == pkg.pkgname)
                {
                    lints.push(LintKind::MissingPackageFunction(pkg.pkgname.to_string()));
                }
            }
        }
    }

    fn lint_makedepends(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self.makedepends.all() {
            check_depend(fulldep, "makedepends", lints);
        }
    }

    fn lint_depends(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self
            .depends
            .all()
            .chain(self.packages().flat_map(|p| p.depends.all()))
        {
            check_depend(fulldep, "depends", lints);
        }
    }

    fn lint_optdepends(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self
            .optdepends
            .all()
            .chain(self.packages().flat_map(|p| p.optdepends.all()))
        {
            let fulldep = fulldep.split(": ").next().unwrap();
            check_depend(fulldep, "optdepends", lints)
        }
    }

    fn lint_provides(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self
            .provides
            .all()
            .chain(self.packages().flat_map(|p| p.provides.all()))
        {
            check_depend(fulldep, "provides", lints);
        }
    }

    fn lint_replaces(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self
            .replaces
            .all()
            .chain(self.packages().flat_map(|p| p.replaces.all()))
        {
            check_depend(fulldep, "replaces", lints);
        }
    }

    fn lint_conflicts(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self
            .conflicts
            .all()
            .chain(self.packages().flat_map(|p| p.conflicts.all()))
        {
            check_depend(fulldep, "conflicts", lints);
        }
    }

    fn lint_checkdepends(&self, lints: &mut Vec<LintKind>) {
        for fulldep in self.checkdepends.all() {
            check_depend(fulldep, "checkdepends", lints);
        }
    }

    fn lint_pkgrel(&self, lints: &mut Vec<LintKind>) {
        check_pkgrel(&self.pkgrel, lints)
    }

    fn lint_epoch(&self, lints: &mut Vec<LintKind>) {
        if let Some(epoch) = &self.epoch {
            check_epock(epoch, lints)
        }
    }

    pub fn lint_pkgver(&self, lints: &mut Vec<LintKind>) {
        check_pkgver(&self.pkgver, "pkgver", lints)
    }

    fn lint_changelog(&self, lints: &mut Vec<LintKind>) {
        for file in self
            .changelog
            .iter()
            .chain(self.packages().flat_map(|p| &p.changelog))
        {
            if !Path::new(file).exists() {
                lints.push(LintKind::MissingFile(
                    "changelog".to_string(),
                    file.to_string(),
                ))
            }
        }
    }

    fn lint_arch(&self, lints: &mut Vec<LintKind>) {
        for arches in iter::once(&self.arch).chain(self.packages().map(|p| &p.arch)) {
            if arches.len() > 1 && arches.iter().any(|a| a == "any") {
                lints.push(LintKind::CantBeArchitectureSpecificAny);
            }
        }
    }

    fn lint_backup(&self, lints: &mut Vec<LintKind>) {
        for backup in self
            .packages()
            .filter(|p| p.is_overridden("backup", None))
            .flat_map(|p| &p.backup)
            .chain(&self.backup)
        {
            if backup.starts_with('/') {
                lints.push(LintKind::BackupHasLeadingSlash(backup.to_string()));
            }
        }
    }

    fn lint_sources(&self, lints: &mut Vec<LintKind>) {
        for arch in &self.source.values {
            let arch = arch.arch.as_deref();

            if self.md5sums.get(arch).is_none()
                && self.sha1sums.get(arch).is_none()
                && self.sha224sums.get(arch).is_none()
                && self.sha256sums.get(arch).is_none()
                && self.sha384sums.get(arch).is_none()
                && self.sha512sums.get(arch).is_none()
                && self.b2sums.get(arch).is_none()
            {
                lints.push(LintKind::IntegrityChecksMissing(name_arch("source", arch)));
            }
        }

        check_integ(&self.source, "md5sums", &self.md5sums, lints);
        check_integ(&self.source, "sha1sums", &self.sha1sums, lints);
        check_integ(&self.source, "sha224sums", &self.sha224sums, lints);
        check_integ(&self.source, "sha256sums", &self.sha256sums, lints);
        check_integ(&self.source, "sha384sums", &self.sha384sums, lints);
        check_integ(&self.source, "sha512sums", &self.sha512sums, lints);
        check_integ(&self.source, "b2sums", &self.b2sums, lints);
    }
}

fn dep_chars(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '+' | '_' | '.' | '@' | '-')
}

fn check_empty(tp: &str, value: &str, lints: &mut Vec<LintKind>) {
    if value.is_empty() {
        lints.push(LintKind::VariabeContainsEmptyString(tp.to_string()));
    }
}

fn check_invalid_chars<F: Fn(char) -> bool>(tp: &str, s: &str, f: F, lints: &mut Vec<LintKind>) {
    let invalid = s.chars().filter(|c| !f(*c)).collect::<String>();
    if !invalid.is_empty() {
        lints.push(LintKind::InvalidChars(tp.to_string(), invalid));
    }
}

fn check_depend(fulldep: &str, tp: &str, lints: &mut Vec<LintKind>) {
    if let Some((dep, rest)) = fulldep.split_once(['<', '>', '=']) {
        check_pkgname(dep, tp, lints);
        let rest = rest.trim_start_matches(['<', '>', '=']);
        if !rest.is_empty() {
            check_fullpkgver(rest, tp, lints);
        }
    } else {
        check_pkgname(fulldep, tp, lints);
    };
}

fn check_pkgname(name: &str, tp: &str, lints: &mut Vec<LintKind>) {
    check_empty(tp, name, lints);

    if name.starts_with('-') {
        lints.push(LintKind::StartsWithInvalid(tp.to_string(), "-".to_string()));
    }

    if name.starts_with('.') {
        lints.push(LintKind::StartsWithInvalid(tp.to_string(), ".".to_string()));
    }

    check_invalid_chars(tp, name, dep_chars, lints);
}

fn check_epock(epoch: &str, lints: &mut Vec<LintKind>) {
    check_invalid_chars("epoch", epoch, |c| c.is_ascii_digit(), lints)
}

pub fn check_pkgrel(pkgrel: &str, lints: &mut Vec<LintKind>) {
    check_empty("pkgrel", pkgrel, lints);
    if pkgrel.chars().filter(|c| *c == '.').count() > 1
        || !pkgrel.chars().all(|c| c.is_ascii_digit() || c == '.')
        || pkgrel.starts_with('.')
        || pkgrel.ends_with('.')
    {
        lints.push(LintKind::InvalidPkgrel(pkgrel.to_string()));
    }
}

fn check_fullpkgver(val: &str, tp: &str, lints: &mut Vec<LintKind>) {
    let (epoch, rest) = match val.split_once(':') {
        Some((epoch, rest)) => (Some(epoch), rest),
        None => (None, val),
    };

    if let Some(epoch) = epoch {
        check_epock(epoch, lints);
    }

    let (pkgver, pkgrel) = match rest.rsplit_once('-') {
        Some((pkgver, pkgrel)) => (pkgver, Some(pkgrel)),
        None => (rest, None),
    };

    check_pkgver(pkgver, tp, lints);

    if let Some(pkgrel) = pkgrel {
        check_pkgrel(pkgrel, lints);
    }
}

pub(crate) fn check_pkgver(val: &str, tp: &str, lints: &mut Vec<LintKind>) {
    check_empty(tp, val, lints);

    if val.contains([':', '/', '-']) || val.contains(char::is_whitespace) {
        lints.push(LintKind::InvalidPkgver(tp.to_string()));
    }

    if !val.chars().all(|c| c.is_ascii()) {
        lints.push(LintKind::AsciiOnly("pkgver".to_string(), tp.to_string()));
    }
}

fn check_integ(
    source: &ArchVecs<Source>,
    name: &str,
    integ: &ArchVecs<String>,
    lints: &mut Vec<LintKind>,
) {
    for arch in &source.values {
        if let Some(integ) = integ.get(arch.arch.as_deref()) {
            if arch.values.len() != integ.values.len() {
                lints.push(LintKind::IntegrityChecksDifferentSize(
                    name_arch("source", arch.arch.as_deref()),
                    name_arch(name, integ.arch.as_deref()),
                ))
            }
        }
    }

    for arch in &integ.values {
        if source.get(arch.arch.as_deref()).is_none() {
            lints.push(LintKind::IntegrityChecksDifferentSize(
                name_arch("source", arch.arch.as_deref()),
                name_arch(name, arch.arch.as_deref()),
            ))
        }
    }
}

fn name_arch(name: &str, arch: Option<&str>) -> String {
    if let Some(arch) = arch {
        format!("{}_{}", name, arch)
    } else {
        name.to_string()
    }
}
