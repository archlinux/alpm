use std::collections::BTreeMap;

use alpm_types::PackageDescription;
use pyo3::prelude::*;

use crate::{
    macros::{btree_convert, impl_from, vec_convert},
    srcinfo::source_info::v1::package::PackageArchitecture,
    types::{
        checksum::{
            SkippableBlake2b512Checksum,
            SkippableCrc32CksumChecksum,
            SkippableMd5Checksum,
            SkippableSha1Checksum,
            SkippableSha224Checksum,
            SkippableSha256Checksum,
            SkippableSha384Checksum,
            SkippableSha512Checksum,
        },
        env::MakepkgOption,
        license::License,
        openpgp::OpenPGPIdentifier,
        path::RelativeFilePath,
        relation::{OptionalDependency, PackageRelation, RelationOrSoname},
        source::Source,
        system::{Architectures, SystemArchitecture},
        url::Url,
        version::FullVersion,
    },
};

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBase(alpm_srcinfo::source_info::v1::package_base::PackageBase);

#[pymethods]
impl PackageBase {
    #[new]
    fn new(name: String, version: FullVersion) -> Result<Self, crate::types::Error> {
        let inner = alpm_srcinfo::source_info::v1::package_base::PackageBase::new_with_defaults(
            alpm_types::Name::new(name.as_str())?,
            version.into(),
        );
        Ok(inner.into())
    }

    #[getter]
    fn get_name(&self) -> String {
        self.0.name.to_string()
    }

    #[setter]
    fn set_name(&mut self, name: String) -> Result<(), crate::types::Error> {
        self.0.name = alpm_types::Name::new(name.as_str())?;
        Ok(())
    }

    #[getter]
    fn get_description(&self) -> Option<String> {
        self.0.description.to_owned().map(|desc| desc.to_string())
    }

    #[setter]
    fn set_description(&mut self, description: Option<String>) {
        self.0.description = description.map(|desc| PackageDescription::new(desc.as_str()));
    }

    #[getter]
    fn get_url(&self) -> Option<Url> {
        self.0.url.to_owned().map(From::from)
    }

    #[setter]
    fn set_url(&mut self, url: Option<Url>) {
        self.0.url = url.map(From::from);
    }

    #[getter]
    fn get_changelog(&self) -> Option<RelativeFilePath> {
        self.0.changelog.to_owned().map(From::from)
    }

    #[setter]
    fn set_changelog(&mut self, changelog: Option<RelativeFilePath>) {
        self.0.changelog = changelog.map(From::from);
    }

    #[getter]
    fn get_licenses(&self) -> Vec<License> {
        vec_convert!(self.0.licenses.clone())
    }

    #[setter]
    fn set_licenses(&mut self, licenses: Vec<License>) {
        self.0.licenses = vec_convert!(licenses);
    }

    #[getter]
    fn get_install(&self) -> Option<RelativeFilePath> {
        self.0.install.clone().map(From::from)
    }

    #[setter]
    fn set_install(&mut self, install: Option<RelativeFilePath>) {
        self.0.install = install.map(From::from);
    }

    #[getter]
    fn get_groups(&self) -> Vec<String> {
        self.0.groups.clone()
    }

    #[setter]
    fn set_groups(&mut self, groups: Vec<String>) {
        self.0.groups = groups;
    }

    #[getter]
    fn get_options(&self) -> Vec<MakepkgOption> {
        vec_convert!(self.0.options.clone())
    }

    #[setter]
    fn set_options(&mut self, options: Vec<MakepkgOption>) {
        self.0.options = vec_convert!(options);
    }

    #[getter]
    fn get_backups(&self) -> Vec<RelativeFilePath> {
        vec_convert!(self.0.backups.clone())
    }

    #[setter]
    fn set_backups(&mut self, backups: Vec<RelativeFilePath>) {
        self.0.backups = vec_convert!(backups);
    }

    #[getter]
    fn get_version(&self) -> FullVersion {
        self.0.version.clone().into()
    }

    #[setter]
    fn set_version(&mut self, version: FullVersion) {
        self.0.version = version.into();
    }

    #[getter]
    fn get_pgp_fingerprints(&self) -> Vec<OpenPGPIdentifier> {
        vec_convert!(self.0.pgp_fingerprints.clone())
    }

    #[setter]
    fn set_pgp_fingerprints(&mut self, pgp_fingerprints: Vec<OpenPGPIdentifier>) {
        self.0.pgp_fingerprints = vec_convert!(pgp_fingerprints);
    }

    #[getter]
    fn get_architectures(&self) -> Architectures {
        self.0.architectures.clone().into()
    }

    #[setter]
    fn set_architectures(&mut self, architectures: Architectures) {
        self.0.architectures = architectures.into();
    }

    #[getter]
    fn get_architecture_properties(&self) -> BTreeMap<SystemArchitecture, PackageBaseArchitecture> {
        btree_convert!(self.0.architecture_properties.clone())
    }

    #[setter]
    fn set_architecture_properties(
        &mut self,
        architecture_properties: BTreeMap<SystemArchitecture, PackageBaseArchitecture>,
    ) {
        self.0.architecture_properties = btree_convert!(architecture_properties);
    }

    #[getter]
    fn get_dependencies(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.dependencies.clone())
    }

    #[setter]
    fn set_dependencies(&mut self, dependencies: Vec<RelationOrSoname>) {
        self.0.dependencies = vec_convert!(dependencies);
    }

    #[getter]
    fn get_optional_dependencies(&self) -> Vec<OptionalDependency> {
        vec_convert!(self.0.optional_dependencies.clone())
    }

    #[setter]
    fn set_optional_dependencies(&mut self, optional_dependencies: Vec<OptionalDependency>) {
        self.0.optional_dependencies = vec_convert!(optional_dependencies);
    }

    #[getter]
    fn get_provides(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.provides.clone())
    }

    #[setter]
    fn set_provides(&mut self, provides: Vec<RelationOrSoname>) {
        self.0.provides = vec_convert!(provides);
    }

    #[getter]
    fn get_conflicts(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.conflicts.clone())
    }

    #[setter]
    fn set_conflicts(&mut self, conflicts: Vec<PackageRelation>) {
        self.0.conflicts = vec_convert!(conflicts);
    }

    #[getter]
    fn get_replaces(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.replaces.clone())
    }

    #[setter]
    fn set_replaces(&mut self, replaces: Vec<PackageRelation>) {
        self.0.replaces = vec_convert!(replaces);
    }

    #[getter]
    fn get_check_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.check_dependencies.clone())
    }

    #[setter]
    fn set_check_dependencies(&mut self, check_dependencies: Vec<PackageRelation>) {
        self.0.check_dependencies = vec_convert!(check_dependencies);
    }

    #[getter]
    fn get_make_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.make_dependencies.clone())
    }

    #[setter]
    fn set_make_dependencies(&mut self, make_dependencies: Vec<PackageRelation>) {
        self.0.make_dependencies = vec_convert!(make_dependencies);
    }

    #[getter]
    fn get_sources(&self) -> Vec<Source> {
        vec_convert!(self.0.sources.clone())
    }

    #[setter]
    fn set_sources(&mut self, sources: Vec<Source>) {
        self.0.sources = vec_convert!(sources);
    }

    #[getter]
    fn get_no_extracts(&self) -> Vec<String> {
        self.0.no_extracts.clone()
    }

    #[setter]
    fn set_no_extracts(&mut self, no_extracts: Vec<String>) {
        self.0.no_extracts = no_extracts;
    }

    #[getter]
    fn get_b2_checksums(&self) -> Vec<SkippableBlake2b512Checksum> {
        vec_convert!(self.0.b2_checksums.clone())
    }

    #[setter]
    fn set_b2_checksums(&mut self, b2_checksums: Vec<SkippableBlake2b512Checksum>) {
        self.0.b2_checksums = vec_convert!(b2_checksums);
    }

    #[getter]
    fn get_md5_checksums(&self) -> Vec<SkippableMd5Checksum> {
        vec_convert!(self.0.md5_checksums.clone())
    }

    #[setter]
    fn set_md5_checksums(&mut self, md5_checksums: Vec<SkippableMd5Checksum>) {
        self.0.md5_checksums = vec_convert!(md5_checksums);
    }

    #[getter]
    fn get_sha1_checksums(&self) -> Vec<SkippableSha1Checksum> {
        vec_convert!(self.0.sha1_checksums.clone())
    }

    #[setter]
    fn set_sha1_checksums(&mut self, sha1_checksums: Vec<SkippableSha1Checksum>) {
        self.0.sha1_checksums = vec_convert!(sha1_checksums);
    }

    #[getter]
    fn get_sha224_checksums(&self) -> Vec<SkippableSha224Checksum> {
        vec_convert!(self.0.sha224_checksums.clone())
    }

    #[setter]
    fn set_sha224_checksums(&mut self, sha224_checksums: Vec<SkippableSha224Checksum>) {
        self.0.sha224_checksums = vec_convert!(sha224_checksums);
    }

    #[getter]
    fn get_sha256_checksums(&self) -> Vec<SkippableSha256Checksum> {
        vec_convert!(self.0.sha256_checksums.clone())
    }

    #[setter]
    fn set_sha256_checksums(&mut self, sha256_checksums: Vec<SkippableSha256Checksum>) {
        self.0.sha256_checksums = vec_convert!(sha256_checksums);
    }

    #[getter]
    fn get_sha384_checksums(&self) -> Vec<SkippableSha384Checksum> {
        vec_convert!(self.0.sha384_checksums.clone())
    }

    #[setter]
    fn set_sha384_checksums(&mut self, sha384_checksums: Vec<SkippableSha384Checksum>) {
        self.0.sha384_checksums = vec_convert!(sha384_checksums);
    }

    #[getter]
    fn get_sha512_checksums(&self) -> Vec<SkippableSha512Checksum> {
        vec_convert!(self.0.sha512_checksums.clone())
    }

    #[setter]
    fn set_sha512_checksums(&mut self, sha512_checksums: Vec<SkippableSha512Checksum>) {
        self.0.sha512_checksums = vec_convert!(sha512_checksums);
    }

    #[getter]
    fn get_crc_checksums(&self) -> Vec<SkippableCrc32CksumChecksum> {
        vec_convert!(self.0.crc_checksums.clone())
    }

    #[setter]
    fn set_crc_checksums(&mut self, crc_checksums: Vec<SkippableCrc32CksumChecksum>) {
        self.0.crc_checksums = vec_convert!(crc_checksums);
    }

    fn __str__(&self) -> String {
        self.0.name.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "PackageBase(name='{}', version={})",
            self.0.name,
            self.get_version().__repr__()
        )
    }
}

impl_from!(
    PackageBase,
    alpm_srcinfo::source_info::v1::package_base::PackageBase
);

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageBaseArchitecture(
    alpm_srcinfo::source_info::v1::package_base::PackageBaseArchitecture,
);

#[pymethods]
impl PackageBaseArchitecture {
    #[new]
    fn new() -> Self {
        alpm_srcinfo::source_info::v1::package_base::PackageBaseArchitecture::default().into()
    }

    fn merge_package_properties(&mut self, properties: PackageArchitecture) {
        self.0.merge_package_properties(properties.into())
    }

    #[getter]
    fn get_dependencies(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.dependencies.clone())
    }

    #[setter]
    fn set_dependencies(&mut self, dependencies: Vec<RelationOrSoname>) {
        self.0.dependencies = vec_convert!(dependencies);
    }

    #[getter]
    fn get_optional_dependencies(&self) -> Vec<OptionalDependency> {
        vec_convert!(self.0.optional_dependencies.clone())
    }

    #[setter]
    fn set_optional_dependencies(&mut self, optional_dependencies: Vec<OptionalDependency>) {
        self.0.optional_dependencies = vec_convert!(optional_dependencies);
    }

    #[getter]
    fn get_provides(&self) -> Vec<RelationOrSoname> {
        vec_convert!(self.0.provides.clone())
    }

    #[setter]
    fn set_provides(&mut self, provides: Vec<RelationOrSoname>) {
        self.0.provides = vec_convert!(provides);
    }

    #[getter]
    fn get_conflicts(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.conflicts.clone())
    }

    #[setter]
    fn set_conflicts(&mut self, conflicts: Vec<PackageRelation>) {
        self.0.conflicts = vec_convert!(conflicts);
    }

    #[getter]
    fn get_replaces(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.replaces.clone())
    }

    #[setter]
    fn set_replaces(&mut self, replaces: Vec<PackageRelation>) {
        self.0.replaces = vec_convert!(replaces);
    }

    #[getter]
    fn get_check_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.check_dependencies.clone())
    }

    #[setter]
    fn set_check_dependencies(&mut self, check_dependencies: Vec<PackageRelation>) {
        self.0.check_dependencies = vec_convert!(check_dependencies);
    }

    #[getter]
    fn get_make_dependencies(&self) -> Vec<PackageRelation> {
        vec_convert!(self.0.make_dependencies.clone())
    }

    #[setter]
    fn set_make_dependencies(&mut self, make_dependencies: Vec<PackageRelation>) {
        self.0.make_dependencies = vec_convert!(make_dependencies);
    }

    #[getter]
    fn get_sources(&self) -> Vec<Source> {
        vec_convert!(self.0.sources.clone())
    }

    #[setter]
    fn set_sources(&mut self, sources: Vec<Source>) {
        self.0.sources = vec_convert!(sources);
    }

    #[getter]
    fn get_b2_checksums(&self) -> Vec<SkippableBlake2b512Checksum> {
        vec_convert!(self.0.b2_checksums.clone())
    }

    #[setter]
    fn set_b2_checksums(&mut self, b2_checksums: Vec<SkippableBlake2b512Checksum>) {
        self.0.b2_checksums = vec_convert!(b2_checksums);
    }

    #[getter]
    fn get_md5_checksums(&self) -> Vec<SkippableMd5Checksum> {
        vec_convert!(self.0.md5_checksums.clone())
    }

    #[setter]
    fn set_md5_checksums(&mut self, md5_checksums: Vec<SkippableMd5Checksum>) {
        self.0.md5_checksums = vec_convert!(md5_checksums);
    }

    #[getter]
    fn get_sha1_checksums(&self) -> Vec<SkippableSha1Checksum> {
        vec_convert!(self.0.sha1_checksums.clone())
    }

    #[setter]
    fn set_sha1_checksums(&mut self, sha1_checksums: Vec<SkippableSha1Checksum>) {
        self.0.sha1_checksums = vec_convert!(sha1_checksums);
    }

    #[getter]
    fn get_sha224_checksums(&self) -> Vec<SkippableSha224Checksum> {
        vec_convert!(self.0.sha224_checksums.clone())
    }

    #[setter]
    fn set_sha224_checksums(&mut self, sha224_checksums: Vec<SkippableSha224Checksum>) {
        self.0.sha224_checksums = vec_convert!(sha224_checksums);
    }

    #[getter]
    fn get_sha256_checksums(&self) -> Vec<SkippableSha256Checksum> {
        vec_convert!(self.0.sha256_checksums.clone())
    }

    #[setter]
    fn set_sha256_checksums(&mut self, sha256_checksums: Vec<SkippableSha256Checksum>) {
        self.0.sha256_checksums = vec_convert!(sha256_checksums);
    }

    #[getter]
    fn get_sha384_checksums(&self) -> Vec<SkippableSha384Checksum> {
        vec_convert!(self.0.sha384_checksums.clone())
    }

    #[setter]
    fn set_sha384_checksums(&mut self, sha384_checksums: Vec<SkippableSha384Checksum>) {
        self.0.sha384_checksums = vec_convert!(sha384_checksums);
    }

    #[getter]
    fn get_sha512_checksums(&self) -> Vec<SkippableSha512Checksum> {
        vec_convert!(self.0.sha512_checksums.clone())
    }

    #[setter]
    fn set_sha512_checksums(&mut self, sha512_checksums: Vec<SkippableSha512Checksum>) {
        self.0.sha512_checksums = vec_convert!(sha512_checksums);
    }

    #[getter]
    fn get_crc_checksums(&self) -> Vec<SkippableCrc32CksumChecksum> {
        vec_convert!(self.0.crc_checksums.clone())
    }

    #[setter]
    fn set_crc_checksums(&mut self, crc_checksums: Vec<SkippableCrc32CksumChecksum>) {
        self.0.crc_checksums = vec_convert!(crc_checksums);
    }
}

impl_from!(
    PackageBaseArchitecture,
    alpm_srcinfo::source_info::v1::package_base::PackageBaseArchitecture
);

#[pymodule(gil_used = false, name = "package_base", submodule)]
pub mod py_package_base {
    #[pymodule_export]
    use super::PackageBase;
    #[pymodule_export]
    use super::PackageBaseArchitecture;
}
