"""Tests for OpenPGP-related alpm_types."""

import pytest
from alpm import ALPMError, alpm_types


def test_openpgp_key_id_valid() -> None:
    """Test creating a valid OpenPGP key ID."""
    key_id = alpm_types.OpenPGPKeyId("ABCD1234ABCD5678")
    assert str(key_id) == "ABCD1234ABCD5678"


def test_openpgp_key_id_invalid() -> None:
    """Test creating an invalid OpenPGP key ID raises error."""
    with pytest.raises(ALPMError):
        alpm_types.OpenPGPKeyId("invalid")


def test_openpgp_fingerprint_valid() -> None:
    """Test creating a valid OpenPGP v4 fingerprint."""
    fingerprint = "1234567890ABCDEF1234567890ABCDEF12345678"
    fp = alpm_types.OpenPGPv4Fingerprint(fingerprint)
    assert str(fp) == fingerprint


def test_openpgp_fingerprint_invalid() -> None:
    """Test creating an invalid OpenPGP fingerprint raises error."""
    with pytest.raises(ALPMError):
        alpm_types.OpenPGPv4Fingerprint("invalid")


def test_openpgp_identifier_from_str_key_id() -> None:
    """Test parsing OpenPGP identifier as key ID."""
    result = alpm_types.openpgp_identifier_from_str("ABCD1234ABCD5678")
    assert isinstance(result, alpm_types.OpenPGPKeyId)


def test_openpgp_identifier_from_str_fingerprint() -> None:
    """Test parsing OpenPGP identifier as fingerprint."""
    fingerprint = "1234567890ABCDEF1234567890ABCDEF12345678"
    result = alpm_types.openpgp_identifier_from_str(fingerprint)
    assert isinstance(result, alpm_types.OpenPGPv4Fingerprint)


def test_openpgp_identifier_from_str_invalid() -> None:
    """Test parsing invalid OpenPGP identifier raises error."""
    with pytest.raises(ALPMError):
        alpm_types.openpgp_identifier_from_str("invalid_identifier")


def test_openpgp_key_id_equality() -> None:
    """Test OpenPGP key ID equality."""
    key_id1 = alpm_types.OpenPGPKeyId("ABCD1234ABCD5678")
    key_id2 = alpm_types.OpenPGPKeyId("ABCD1234ABCD5678")
    assert key_id1 == key_id2


def test_openpgp_fingerprint_equality() -> None:
    """Test OpenPGP fingerprint equality."""
    fingerprint_str = "1234567890ABCDEF1234567890ABCDEF12345678"
    fp1 = alpm_types.OpenPGPv4Fingerprint(fingerprint_str)
    fp2 = alpm_types.OpenPGPv4Fingerprint(fingerprint_str)
    assert fp1 == fp2
