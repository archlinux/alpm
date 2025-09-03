# TODO: <https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/204>
from alpm.alpm_srcinfo import error as _mod

__all__ = _mod.__all__
for name in getattr(_mod, "__all__"):
    globals()[name] = getattr(_mod, name)
