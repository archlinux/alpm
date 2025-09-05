"""Python bindings for the Arch Linux Package Management (ALPM) project."""

from ._native import alpm_srcinfo, alpm_types, ALPMError
from . import type_aliases

__all__ = ["alpm_types", "alpm_srcinfo", "type_aliases", "ALPMError"]
