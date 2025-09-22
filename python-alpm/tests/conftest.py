import pytest


@pytest.fixture
def valid_srcinfo_content() -> str:
    """Fixture providing valid SRCINFO content."""
    return """
pkgbase = example
	pkgdesc = A example with all pkgbase properties set.
	pkgver = 0.1.0
	pkgrel = 1
	epoch = 1
	url = https://archlinux.org
	install = install.sh.stub
	changelog = changelog.stub
	arch = x86_64
	arch = aarch64
	groups = group
	groups = group_2
	license = MIT
	depends = default_dep
	optdepends = default_optdep
	provides = default_provides
	conflicts = default_conflict
	replaces = default_replaces
	options = !lto
	backup = etc/pacman.conf
	provides_x86_64 = arch_default_provides
	conflicts_x86_64 = arch_default_conflict
	depends_x86_64 = arch_default_dep
	replaces_x86_64 = arch_default_replaces
	optdepends_x86_64 = arch_default_optdep

pkgname = example
	pkgdesc = overridden
	url = https://overridden.com/
	install = overridden.stub
	changelog = overridden.stub
	groups = overridden
	license = Apache-2.0
	depends = overridden
	optdepends = overridden
	provides = overridden
	conflicts = overridden
	replaces = overridden
	options = emptydirs
	backup = overridden
	provides_x86_64 = arch_overridden
	conflicts_x86_64 = arch_overridden
	depends_x86_64 = arch_overridden
	replaces_x86_64 = arch_overridden
	optdepends_x86_64 = arch_overridden

"""


@pytest.fixture
def valid_pkgbuild_content() -> str:
    """Fixture providing valid PKGBUILD content."""
    return """
#!/bin/bash
# Disable unused variable warnings:
# shellcheck disable=2034
pkgname=(example)
pkgver=0.1.0
pkgrel=1
epoch=1
arch=(x86_64 aarch64)

pkgdesc="A example with all pkgbase properties set."
url="https://archlinux.org/"
license=(MIT)
changelog=changelog
install=install.sh
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
"""
