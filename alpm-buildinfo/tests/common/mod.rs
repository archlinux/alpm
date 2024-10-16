pub const VALID_BUILDINFO_DATA: &str = r#"builddate = 1
builddir = /build
buildenv = envfoo
buildenv = envbar
format = 1
installed = bar-1.2.3-1-any
installed = beh-2.2.3-4-any
options = some_option
options = !other_option
packager = Foobar McFooface <foobar@mcfooface.org>
pkgarch = any
pkgbase = foo
pkgbuild_sha256sum = b5bb9d8014a0f9b1d61e21e796d78dccdf1352f23cd32812f4850b878ae4944c
pkgname = foo
pkgver = 1:1.0.0-1
"#;

pub struct BuildInfoV1Input {
    pub builddate: (Option<String>, bool),
    pub builddir: (Option<String>, bool),
    pub buildenv: (Option<Vec<String>>, bool),
    pub installed: (Option<Vec<String>>, bool),
    pub options: (Option<Vec<String>>, bool),
    pub packager: (Option<String>, bool),
    pub pkgarch: (Option<String>, bool),
    pub pkgbase: (Option<String>, bool),
    pub pkgbuild_sha256sum: (Option<String>, bool),
    pub pkgname: (Option<String>, bool),
    pub pkgver: (Option<String>, bool),
    pub should_be_valid: bool,
}

impl Default for BuildInfoV1Input {
    fn default() -> Self {
        BuildInfoV1Input {
            builddate: (None, false),
            builddir: (None, false),
            buildenv: (None, false),
            installed: (None, false),
            options: (None, false),
            packager: (None, false),
            pkgarch: (None, false),
            pkgbase: (None, false),
            pkgbuild_sha256sum: (None, false),
            pkgname: (None, false),
            pkgver: (None, false),
            should_be_valid: false,
        }
    }
}
