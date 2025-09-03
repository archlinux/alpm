# TODO: <https://gitlab.archlinux.org/archlinux/alpm/alpm/-/issues/204>
from alpm.alpm_srcinfo.source_info.v1 import merged as _mod

__all__ = _mod.__all__
for name in getattr(_mod, "__all__"):
    globals()[name] = getattr(_mod, name)
