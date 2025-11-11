use std::collections::BTreeMap;

use alpm_srcinfo::source_info::v1::package as alpm_srcinfo_package;
use pyo3::{exceptions::PyTypeError, prelude::*};

use crate::{
    macros::{btree_convert, impl_from, vec_convert},
    types::{
        env::MakepkgOption,
        license::License,
        path::RelativeFilePath,
        relation::{OptionalDependency, PackageRelation, RelationOrSoname},
        system::{Architectures, SystemArchitecture},
        url::Url,
    },
};

/// Helper macro to convert a generic [`package::Override<T>`] into non-generic
/// [`Option<Override>`]. This is used in getters to convert the internal representation into the
/// Python-exposed one. Since we can't pass generics to Python, we are deducing the type `T` based
/// on the struct field.
///
/// - `$override` is the source override value to convert.
/// - `$matcher` must always be `package::Override::Yes { value }` to match the inner value.
/// - `$body` defines the conversion from a generic `value` to [`Overridable`] enum variant.
///
/// Conversions of [`package::Override::No`] and [`package::Override::Clear`] are handled
/// internally.
///
/// # Examples
///
/// ```
/// use alpm_types::Description;
/// use alpm_srcinfo::source_info::v1::package::Override as AlpmOverride;
///
/// let override = AlpmOverride::Yes { value: Description::new("Some description") };
///
/// let pyoverride = into_pyoverride!(override,
///     // Since we know the override type is `Description`, which
///     // is internally a `String`. We can convert it to `Overridable::String`.
///     AlpmOverride::Yes { value } => Overridable::String(value.to_string())
/// );
/// ```
///
/// [`package::Override<T>`]: alpm_srcinfo_package::Override
/// [`package::Override::No`]: alpm_srcinfo_package::Override::No
/// [`package::Override::Clear`]: alpm_srcinfo_package::Override::Clear
macro_rules! into_pyoverride {
    ($override:expr, $matcher:pat_param => $body:expr) => {
        match $override {
            alpm_srcinfo_package::Override::No => None,
            alpm_srcinfo_package::Override::Clear => Some(Override(None)),
            $matcher => {
                let overridable = $body;
                Some(Override(Some(overridable)))
            }
        }
    };
}

/// Helper macro to implement setters for fields of type [`package::Override<T>`].
/// Handles conversion from non-generic [`Option<Override>`] to a generic [`package::Override<T>`].
/// This is used in setters to convert the Python-exposed representation into the internal one.
///
/// Sets a struct field `$field` to `$override`, after converting it to appropriate type.
/// Available conversions are implemented using [`impl_tryfrom_override!`] macro.
///
/// # Note
///
/// This macro only implements some of the setter body instead of the whole setter method,
/// as `#[pymethods]` forbids macros directly inside impl block bodies.
///
/// # Examples
///
/// ```ignore
/// #[setter]
/// fn set_description(&mut self, description: Option<Override>) -> PyResult<()> {
///     // Converts `description` of type `Option<Override>` to `package::Override<T>`,
///     // and sets `self.0.description` to the converted value.
///     impl_override_setter!(self.0.description, description);
///     Ok(())
/// }
/// ```
///
/// [`package::Override<T>`]: alpm_srcinfo_package::Override
/// ```internal
macro_rules! impl_override_setter {
    ($field:expr, $override:expr) => {
        match $override {
            None => $field = alpm_srcinfo_package::Override::No,
            Some(value) => {
                $field = alpm_srcinfo_package::Override::try_from(value)?;
            }
        }
    };
}

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct Package(alpm_srcinfo_package::Package);

#[pymethods]
impl Package {
    #[new]
    fn new(name: &str) -> Result<Self, crate::types::Error> {
        let inner = alpm_srcinfo_package::Package::from(alpm_types::Name::new(name)?);
        Ok(inner.into())
    }

    #[getter]
    fn get_name(&self) -> String {
        self.0.name.to_string()
    }

    #[setter]
    fn set_name(&mut self, name: &str) -> Result<(), crate::types::Error> {
        self.0.name = alpm_types::Name::new(name)?;
        Ok(())
    }

    #[getter]
    fn get_description(&self) -> Option<Override> {
        into_pyoverride!(self.0.description.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::String(value.to_string())
        )
    }

    #[setter]
    fn set_description(&mut self, description: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.description, description);
        Ok(())
    }

    #[getter]
    fn get_url(&self) -> Option<Override> {
        into_pyoverride!(self.0.url.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::Url(value.into())
        )
    }

    #[setter]
    fn set_url(&mut self, url: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.url, url);
        Ok(())
    }

    #[getter]
    fn get_changelog(&self) -> Option<Override> {
        into_pyoverride!(self.0.changelog.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::RelativeFilePath(value.into())
        )
    }

    #[setter]
    fn set_changelog(&mut self, changelog: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.changelog, changelog);
        Ok(())
    }

    #[getter]
    fn get_licenses(&self) -> Option<Override> {
        into_pyoverride!(self.0.licenses.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::Licenses(vec_convert!(value))
        )
    }

    #[setter]
    fn set_licenses(&mut self, licenses: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.licenses, licenses);
        Ok(())
    }

    #[getter]
    fn get_install(&self) -> Option<Override> {
        into_pyoverride!(self.0.install.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::RelativeFilePath(value.into())
        )
    }

    #[setter]
    fn set_install(&mut self, install: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.install, install);
        Ok(())
    }

    #[getter]
    fn get_groups(&self) -> Option<Override> {
        into_pyoverride!(self.0.groups.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::Strings(value)
        )
    }

    #[setter]
    fn set_groups(&mut self, groups: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.groups, groups);
        Ok(())
    }

    #[getter]
    fn get_options(&self) -> Option<Override> {
        into_pyoverride!(self.0.options.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::MakepkgOptions(vec_convert!(value))
        )
    }

    #[setter]
    fn set_options(&mut self, options: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.options, options);
        Ok(())
    }

    #[getter]
    fn get_backups(&self) -> Option<Override> {
        into_pyoverride!(self.0.backups.clone(),
            alpm_srcinfo_package::Override::Yes { value } => Overridable::RelativeFilePaths(vec_convert!(value))
        )
    }

    #[setter]
    fn set_backups(&mut self, backups: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.backups, backups);
        Ok(())
    }

    // No `Override` type here, as this field can't be cleared.
    #[getter]
    fn get_architectures(&self) -> Option<Architectures> {
        self.0.architectures.clone().map(From::from)
    }

    // No `Override` type here, as this field can't be cleared.
    #[setter]
    fn set_architectures(&mut self, architectures: Option<Architectures>) {
        self.0.architectures = architectures.map(From::from);
    }

    #[getter]
    fn get_architecture_properties(&self) -> BTreeMap<SystemArchitecture, PackageArchitecture> {
        btree_convert!(self.0.architecture_properties.clone())
    }

    #[setter]
    fn set_architecture_properties(
        &mut self,
        architecture_properties: BTreeMap<SystemArchitecture, PackageArchitecture>,
    ) {
        self.0.architecture_properties = btree_convert!(architecture_properties);
    }

    #[getter]
    fn get_dependencies(&self) -> Option<Override> {
        into_pyoverride!(self.0.dependencies.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::RelationsOrSonames(vec_convert!(value))
        )
    }

    #[setter]
    fn set_dependencies(&mut self, dependencies: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.dependencies, dependencies);
        Ok(())
    }

    #[getter]
    fn optional_dependencies(&self) -> Option<Override> {
        into_pyoverride!(self.0.optional_dependencies.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::OptionalDependencies(vec_convert!(value))
        )
    }

    #[setter]
    fn set_optional_dependencies(
        &mut self,
        optional_dependencies: Option<Override>,
    ) -> PyResult<()> {
        impl_override_setter!(self.0.optional_dependencies, optional_dependencies);
        Ok(())
    }

    #[getter]
    fn provides(&self) -> Option<Override> {
        into_pyoverride!(self.0.provides.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::RelationsOrSonames(vec_convert!(value))
        )
    }

    #[setter]
    fn set_provides(&mut self, provides: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.provides, provides);
        Ok(())
    }

    #[getter]
    fn conflicts(&self) -> Option<Override> {
        into_pyoverride!(self.0.conflicts.clone(),
            alpm_srcinfo_package::Override::Yes { value } => {
                let relations: Vec<PackageRelation> = vec_convert!(value);
                Overridable::RelationsOrSonames(vec_convert!(relations))
            }
        )
    }

    #[setter]
    fn set_conflicts(&mut self, conflicts: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.conflicts, conflicts);
        Ok(())
    }

    #[getter]
    fn replaces(&self) -> Option<Override> {
        into_pyoverride!(self.0.replaces.clone(),
            alpm_srcinfo_package::Override::Yes { value } => {
                let relations: Vec<PackageRelation> = vec_convert!(value);
                Overridable::RelationsOrSonames(vec_convert!(relations))
            }
        )
    }

    #[setter]
    fn set_replaces(&mut self, replaces: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.replaces, replaces);
        Ok(())
    }

    fn __str__(&self) -> String {
        self.0.name.to_string()
    }

    fn __repr__(&self) -> String {
        format!("Package(name='{}')", self.0.name)
    }
}

impl_from!(Package, alpm_srcinfo_package::Package);

#[pyclass(eq)]
#[derive(Clone, Debug, PartialEq)]
pub struct PackageArchitecture(alpm_srcinfo_package::PackageArchitecture);

// Note: We can't use macros directly inside `#[pymethods]` body,
// that's why setter macros are wrapped in methods.
#[pymethods]
impl PackageArchitecture {
    #[new]
    fn new() -> Self {
        alpm_srcinfo_package::PackageArchitecture::default().into()
    }

    #[getter]
    fn get_dependencies(&self) -> Option<Override> {
        into_pyoverride!(self.0.dependencies.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::RelationsOrSonames(vec_convert!(value))
        )
    }

    #[setter]
    fn set_dependencies(&mut self, dependencies: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.dependencies, dependencies);
        Ok(())
    }

    #[getter]
    fn optional_dependencies(&self) -> Option<Override> {
        into_pyoverride!(self.0.optional_dependencies.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::OptionalDependencies(vec_convert!(value))
        )
    }

    #[setter]
    fn set_optional_dependencies(
        &mut self,
        optional_dependencies: Option<Override>,
    ) -> PyResult<()> {
        impl_override_setter!(self.0.optional_dependencies, optional_dependencies);
        Ok(())
    }

    #[getter]
    fn provides(&self) -> Option<Override> {
        into_pyoverride!(self.0.provides.clone(),
            alpm_srcinfo_package::Override::Yes { value } =>
            Overridable::RelationsOrSonames(vec_convert!(value))
        )
    }

    #[setter]
    fn set_provides(&mut self, provides: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.provides, provides);
        Ok(())
    }

    #[getter]
    fn conflicts(&self) -> Option<Override> {
        into_pyoverride!(self.0.conflicts.clone(),
            alpm_srcinfo_package::Override::Yes { value } => {
                let relations: Vec<PackageRelation> = vec_convert!(value);
                Overridable::RelationsOrSonames(vec_convert!(relations))
            }
        )
    }

    #[setter]
    fn set_conflicts(&mut self, conflicts: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.conflicts, conflicts);
        Ok(())
    }

    #[getter]
    fn replaces(&self) -> Option<Override> {
        into_pyoverride!(self.0.replaces.clone(),
            alpm_srcinfo_package::Override::Yes { value } => {
                let relations: Vec<PackageRelation> = vec_convert!(value);
                Overridable::RelationsOrSonames(vec_convert!(relations))
            }
        )
    }

    #[setter]
    fn set_replaces(&mut self, replaces: Option<Override>) -> PyResult<()> {
        impl_override_setter!(self.0.replaces, replaces);
        Ok(())
    }
}

impl_from!(
    PackageArchitecture,
    alpm_srcinfo_package::PackageArchitecture
);

// This enum is used to represent all possible types that can be overridden.
// This is necessary because Rust generics are evaluated at compile time.
#[derive(Clone, Debug, FromPyObject, IntoPyObject, PartialEq)]
pub enum Overridable {
    String(String),
    Strings(Vec<String>),
    Url(Url),
    MakepkgOptions(Vec<MakepkgOption>),
    RelativeFilePath(RelativeFilePath),
    RelativeFilePaths(Vec<RelativeFilePath>),
    Licenses(Vec<License>),
    RelationsOrSonames(Vec<RelationOrSoname>),
    OptionalDependencies(Vec<OptionalDependency>),
}

// Python           | Rust
//------------------|---------------------
// None             | Override::No
// Override(None)   | Override::Clear
// Override(T)      | Override::Yes { value: T }
#[pyclass(frozen)]
#[derive(Clone, Debug, PartialEq)]
pub struct Override(Option<Overridable>);

#[pymethods]
impl Override {
    #[new]
    fn new(value: Option<Overridable>) -> Self {
        Self(value)
    }

    #[getter]
    fn value(&self) -> Option<Overridable> {
        self.0.clone()
    }

    fn __repr__<'a>(&self, py: Python<'a>) -> PyResult<String> {
        match &self.0 {
            Some(overridable) => Ok(format!(
                "Override(value={})",
                overridable.clone().into_pyobject(py)?.repr()?
            )),
            None => Ok("Override(value=None)".to_string()),
        }
    }
}

impl_from!(Override, Option<Overridable>);

/// Implements [`TryFrom<Override>`] for [`package::Override<T>`] for selected `T`.
/// This is fallible because we can't know the type passed from Python at compile time.
/// We are expecting type `T` based on the [`Package`]/[`PackageArchitecture`] field we are setting.
/// If the type doesn't match, we raise a Python `TypeError` and display the expected type.
///
/// Implementing these conversions lets us easily implement setters with `impl_override_setter!`
/// macro.
///
/// - `$type` is the target type `T` of `alpm_srcinfo_package::Override<T>`.
/// - `$name` is the name of expected Python type, used in error messages.
/// - `$matcher` is a pattern to match `Overridable` enum variant.
/// - `$body` defines the conversion from matched `Overridable` variant to target type `T`.
///
/// # Note
///
/// `T` must be an [`Overridable`] variant (or a newtype of [`Overridable`] variant).
/// We have to track all possible `T`s and restrict it on Python side,
/// as Rust generics are evaluated at compile time.
///
/// # Examples
///
/// ```
/// impl_tryfrom_override!(
///     Vec<alpm_types::MakepkgOption>,
///     "list[MakepkgOption]",
///     // We can use `vec_convert!` macro to convert vec of newtypes `Vec<MakepkgOption>`
///     // to Vec<alpm_types::MakepkgOption>.
///     Overridable::MakepkgOptions(options) => vec_convert!(options)
/// );
/// ```
///
/// [`Package`]: alpm_srcinfo_package::Package
/// [`PackageArchitecture`]: alpm_srcinfo_package::PackageArchitecture
/// [`package::Override<T>`]: alpm_srcinfo_package::Override
macro_rules! impl_tryfrom_override {
    ($type:ty, $name:literal, $matcher:pat_param => $body:expr) => {
        impl TryFrom<Override> for alpm_srcinfo_package::Override<$type> {
            type Error = PyErr;
            fn try_from(value: Override) -> Result<Self, Self::Error> {
                match value.0 {
                    Some($matcher) => Ok(alpm_srcinfo_package::Override::Yes {
                        value: $body.into(),
                    }),
                    None => Ok(alpm_srcinfo_package::Override::Clear),
                    _ => Err(PyTypeError::new_err(format!(
                        "expected type: 'Override[{}] | None'",
                        $name
                    ))),
                }
            }
        }
    };
}

impl_tryfrom_override!(String, "str", Overridable::String(s) => s.as_str());
impl_tryfrom_override!(Vec<String>, "list[str]", Overridable::Strings(s) => s);
impl_tryfrom_override!(alpm_types::PackageDescription, "str", Overridable::String(s) => s.as_str());
impl_tryfrom_override!(alpm_types::Url, "Url", Overridable::Url(u) => u);
impl_tryfrom_override!(alpm_types::RelativeFilePath, "RelativeFilePath", Overridable::RelativeFilePath(p) => p);
impl_tryfrom_override!(Vec<alpm_types::RelativeFilePath>, "list[RelativeFilePath]", Overridable::RelativeFilePaths(p) => vec_convert!(p));
impl_tryfrom_override!(Vec<alpm_types::License>, "list[License]",
    Overridable::Licenses(l) => vec_convert!(l)
);
impl_tryfrom_override!(Vec<alpm_types::RelationOrSoname>, "list[RelationOrSoname]", Overridable::RelationsOrSonames(r) => vec_convert!(r));
impl_tryfrom_override!(Vec<alpm_types::OptionalDependency>, "list[OptionalDependency]", Overridable::OptionalDependencies(o) => vec_convert!(o));
impl_tryfrom_override!(Vec<alpm_types::MakepkgOption>, "list[MakepkgOption]", Overridable::MakepkgOptions(o) => vec_convert!(o));

// This is a special case, because we can't differentiate between
// `PackageRelation` and `RelationOrSoname` when passing Override between Python and Rust.
// Adding a dedicated `Overridable::PackageRelations` variant would cause conversions
// to wrong variants in case of ambiguity.
impl TryFrom<Override> for alpm_srcinfo_package::Override<Vec<alpm_types::PackageRelation>> {
    type Error = PyErr;
    fn try_from(value: Override) -> Result<Self, Self::Error> {
        match value.0 {
            Some(Overridable::RelationsOrSonames(relations_or_sonames)) => {
                let package_relations: Vec<PackageRelation> = relations_or_sonames
                    .into_iter()
                    .map(TryFrom::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(alpm_srcinfo_package::Override::Yes {
                    value: vec_convert!(package_relations),
                })
            }
            None => Ok(alpm_srcinfo_package::Override::Clear),
            _ => Err(PyTypeError::new_err(
                "expected type: 'Override[list[PackageRelation]] | None'",
            )),
        }
    }
}

#[pymodule(gil_used = false, name = "package", submodule)]
pub mod py_package {
    #[pymodule_export]
    use super::Override;
    #[pymodule_export]
    use super::Package;
    #[pymodule_export]
    use super::PackageArchitecture;
}
