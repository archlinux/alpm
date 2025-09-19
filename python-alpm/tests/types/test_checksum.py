"""Tests for checksum alpm_types."""

import pytest
from alpm import ALPMError, alpm_types
from alpm.type_aliases import Checksum, SkippableChecksum


@pytest.mark.parametrize(
    "checksum_type, valid_hash",
    [
        (
            alpm_types.Blake2b512Checksum,
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
        ),
        (alpm_types.Md5Checksum, "d41d8cd98f00b204e9800998ecf8427e"),
        (alpm_types.Sha1Checksum, "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        (
            alpm_types.Sha224Checksum,
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
        ),
        (
            alpm_types.Sha256Checksum,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            alpm_types.Sha384Checksum,
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
        ),
        (
            alpm_types.Sha512Checksum,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        ),
    ],
)
def test_checksum_valid(checksum_type: type[Checksum], valid_hash: str) -> None:
    """Test creating a valid checksum."""
    checksum = checksum_type(valid_hash)
    assert str(checksum) == valid_hash


@pytest.mark.parametrize(
    "checksum_type, valid_hash",
    [
        (
            alpm_types.SkippableBlake2b512Checksum,
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce",
        ),
        (alpm_types.SkippableMd5Checksum, "d41d8cd98f00b204e9800998ecf8427e"),
        (alpm_types.SkippableSha1Checksum, "da39a3ee5e6b4b0d3255bfef95601890afd80709"),
        (
            alpm_types.SkippableSha224Checksum,
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f",
        ),
        (
            alpm_types.SkippableSha256Checksum,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        ),
        (
            alpm_types.SkippableSha384Checksum,
            "38b060a751ac96384cd9327eb1b1e36a21fdb71114be07434c0cc7bf63f6e1da274edebfe76f65fbd51ad2f14898b95b",
        ),
        (
            alpm_types.SkippableSha512Checksum,
            "cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e",
        ),
    ],
)
def test_skippable_checksum_valid(
    checksum_type: type[SkippableChecksum], valid_hash: str
) -> None:
    """Test creating a valid skippable checksum."""
    checksum = checksum_type(valid_hash)
    assert checksum.is_skipped is False
    assert str(checksum) == valid_hash


@pytest.mark.parametrize(
    "checksum_type",
    [
        alpm_types.SkippableBlake2b512Checksum,
        alpm_types.SkippableMd5Checksum,
        alpm_types.SkippableSha1Checksum,
        alpm_types.SkippableSha224Checksum,
        alpm_types.SkippableSha256Checksum,
        alpm_types.SkippableSha384Checksum,
        alpm_types.SkippableSha512Checksum,
    ],
)
def test_skippable_checksum_skip(checksum_type: type[SkippableChecksum]) -> None:
    """Test creating a valid skipped skippable checksum."""
    checksum = checksum_type("SKIP")
    assert checksum.is_skipped is True
    assert str(checksum) == "SKIP"


@pytest.mark.parametrize(
    "checksum_type",
    [
        alpm_types.Blake2b512Checksum,
        alpm_types.Md5Checksum,
        alpm_types.Sha1Checksum,
        alpm_types.Sha224Checksum,
        alpm_types.Sha256Checksum,
        alpm_types.Sha384Checksum,
        alpm_types.Sha512Checksum,
    ],
)
def test_checksum_invalid(checksum_type: type[Checksum]) -> None:
    """Test creating an invalid checksum raises error."""
    with pytest.raises(ALPMError):
        checksum_type("invalid_hash")


def test_checksum_equality() -> None:
    """Test checksum equality comparison."""
    hash_value = "d41d8cd98f00b204e9800998ecf8427e"
    checksum1 = alpm_types.Md5Checksum(hash_value)
    checksum2 = alpm_types.Md5Checksum(hash_value)
    assert checksum1 == checksum2


def test_checksum_inequality() -> None:
    """Test checksum inequality comparison."""
    checksum1 = alpm_types.Md5Checksum("d41d8cd98f00b204e9800998ecf8427e")
    checksum2 = alpm_types.Md5Checksum("5d41402abc4b2a76b9719d911017c592")
    assert checksum1 != checksum2


def test_checksum_ordering() -> None:
    """Test checksum ordering comparison."""
    checksum1 = alpm_types.Md5Checksum("5d41402abc4b2a76b9719d911017c592")
    checksum2 = alpm_types.Md5Checksum("d41d8cd98f00b204e9800998ecf8427e")
    assert checksum1 < checksum2


def test_checksum_frozen() -> None:
    """Test that checksums are frozen (immutable)."""
    checksum = alpm_types.Md5Checksum("d41d8cd98f00b204e9800998ecf8427e")
    with pytest.raises(AttributeError):
        checksum.new_attr = "test"  # type: ignore[attr-defined]
