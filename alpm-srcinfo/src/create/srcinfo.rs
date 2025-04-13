use crate::create::pkgbuild::{ArchVecs, Package, Pkgbuild};
use std::fmt::Display;
use std::io;
use std::io::Write;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SrcinfoError {
    #[error("srcinfo io error: {0:#}")]
    IOError(#[from] io::Error),
}

impl Pkgbuild {
    pub fn srcinfo_file_name() -> &'static str {
        ".SRCINFO"
    }

    fn write_arch_arrays<W, D>(
        &self,
        name: &str,
        arrs: &ArchVecs<D>,
        w: &mut W,
    ) -> Result<(), SrcinfoError>
    where
        W: Write,
        D: Display,
    {
        for arr in &arrs.values {
            self.write_arch_val(name, arr.arch.as_deref(), &arr.values, w)?;
        }
        Ok(())
    }

    fn write_arch_val<W, D, I>(
        &self,
        n: &str,
        arch: Option<&str>,
        arr: I,
        w: &mut W,
    ) -> Result<(), SrcinfoError>
    where
        W: Write,
        D: Display,
        I: IntoIterator<Item = D>,
    {
        for val in arr {
            if let Some(arch) = arch {
                writeln!(w, "\t{}_{} = {}", n, arch, val)?;
            } else {
                writeln!(w, "\t{} = {}", n, val)?;
            }
        }
        Ok(())
    }

    fn write_val<W, D, I>(&self, name: &str, arr: I, w: &mut W) -> Result<(), SrcinfoError>
    where
        W: Write,
        D: Display,
        I: IntoIterator<Item = D>,
    {
        self.write_arch_val(name, None, arr, w)
    }

    fn write_arch_array_overriddes<W: Write, D: Display>(
        &self,
        package: &Package,
        name: &str,
        arrs: &ArchVecs<D>,
        w: &mut W,
    ) -> Result<(), SrcinfoError> {
        for arr in &arrs.values {
            if !package.is_overridden(name, arr.arch.as_deref()) {
                continue;
            }
            let mut arrs = arrs.values.iter().peekable();
            if arrs.peek().is_none() {
                writeln!(w, "\t{} =", name)?;
                break;
            }
            self.write_arch_val(name, arr.arch.as_deref(), &arr.values, w)?;
        }

        Ok(())
    }

    fn write_overriddes<W: Write, D: Display, I: IntoIterator<Item = D>>(
        &self,
        package: &Package,
        name: &str,
        vals: I,
        w: &mut W,
    ) -> Result<(), SrcinfoError>
    where
        I::IntoIter: ExactSizeIterator,
    {
        if !package.is_overridden(name, None) {
            return Ok(());
        }
        let mut vals = vals.into_iter().peekable();
        if vals.peek().is_none() {
            writeln!(w, "\t{} =", name)?;
            return Ok(());
        }

        for val in vals {
            writeln!(w, "\t{} = {}", name, val)?;
        }
        Ok(())
    }

    pub fn srcinfo(&self) -> String {
        let mut s = Vec::new();
        self.write_srcinfo(&mut s).unwrap();
        String::from_utf8(s).unwrap()
    }

    pub fn write_srcinfo<W: Write>(&self, w: &mut W) -> Result<(), SrcinfoError> {
        writeln!(w, "pkgbase = {}", self.pkgbase)?;
        self.write_val("pkgdesc", &self.pkgdesc, w)?;
        writeln!(w, "\tpkgver = {}", self.pkgver)?;
        writeln!(w, "\tpkgrel = {}", self.pkgrel)?;
        self.write_val("epoch", &self.epoch, w)?;
        self.write_val("url", &self.url, w)?;
        self.write_val("install", &self.install, w)?;
        self.write_val("changelog", &self.changelog, w)?;
        self.write_val("arch", &self.arch, w)?;
        self.write_val("groups", &self.groups, w)?;
        self.write_val("license", &self.license, w)?;
        self.write_arch_arrays("checkdepends", &self.checkdepends, w)?;
        self.write_arch_arrays("makedepends", &self.makedepends, w)?;
        self.write_arch_arrays("depends", &self.depends, w)?;
        self.write_arch_arrays("optdepends", &self.optdepends, w)?;
        self.write_arch_arrays("provides", &self.provides, w)?;
        self.write_arch_arrays("conflicts", &self.conflicts, w)?;
        self.write_arch_arrays("replaces", &self.replaces, w)?;
        self.write_val("noextract", &self.noextract, w)?;
        self.write_val("options", &self.options.values, w)?;
        self.write_val("backup", &self.backup, w)?;
        self.write_arch_arrays("source", &self.source, w)?;
        self.write_val("validpgpkeys", &self.validpgpkeys, w)?;
        self.write_arch_arrays("md5sums", &self.md5sums, w)?;
        self.write_arch_arrays("sha1sums", &self.sha1sums, w)?;
        self.write_arch_arrays("sha224sums", &self.sha224sums, w)?;
        self.write_arch_arrays("sha256sums", &self.sha256sums, w)?;
        self.write_arch_arrays("sha384sums", &self.sha384sums, w)?;
        self.write_arch_arrays("sha512sums", &self.sha512sums, w)?;
        self.write_arch_arrays("b2sums", &self.b2sums, w)?;

        for package in &self.packages {
            self.write_srcinfo_pkg(package, w)?;
        }

        Ok(())
    }

    fn write_srcinfo_pkg<W: Write>(&self, pkg: &Package, w: &mut W) -> Result<(), SrcinfoError> {
        writeln!(w, "\npkgname = {}", pkg.pkgname)?;
        self.write_overriddes(pkg, "pkgdesc", &pkg.pkgdesc, w)?;
        self.write_overriddes(pkg, "url", &pkg.url, w)?;
        self.write_overriddes(pkg, "install", &pkg.install, w)?;
        self.write_overriddes(pkg, "changelog", &pkg.changelog, w)?;
        self.write_overriddes(pkg, "arch", &pkg.arch, w)?;
        self.write_overriddes(pkg, "groups", &pkg.groups, w)?;
        self.write_overriddes(pkg, "license", &pkg.license, w)?;
        self.write_arch_array_overriddes(pkg, "depends", &pkg.depends, w)?;
        self.write_arch_array_overriddes(pkg, "optdepends", &pkg.optdepends, w)?;
        self.write_arch_array_overriddes(pkg, "provides", &pkg.provides, w)?;
        self.write_arch_array_overriddes(pkg, "conflicts", &pkg.conflicts, w)?;
        self.write_arch_array_overriddes(pkg, "replaces", &pkg.replaces, w)?;
        self.write_overriddes(pkg, "options", &pkg.options.values, w)?;
        self.write_overriddes(pkg, "backup", &pkg.backup, w)?;
        Ok(())
    }

    pub fn write_srcinfo_file(&self) -> Result<(), SrcinfoError> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(Pkgbuild::srcinfo_file_name())?;
        self.write_srcinfo(&mut file)?;

        Ok(())
    }
}
