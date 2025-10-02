"""Tests for URL type."""

import pytest
from alpm import ALPMError, alpm_types


@pytest.mark.parametrize(
    "url_str",
    [
        "http://example.com",
        "http://example.com/",
        "https://example.com",
        "ftp://example.com",
        "https://example.com/some/path",
        "https://example.com/path?param=value",
        "https://example.com/path#section",
    ],
)
def test_url_valid(url_str: str) -> None:
    """Test creating a valid HTTP URL."""
    url = alpm_types.Url(url_str)
    assert str(url).rstrip("/") == url_str.rstrip("/")


def test_url_equality() -> None:
    """Test URL equality."""
    url_str = "https://example.com"
    url1 = alpm_types.Url(url_str)
    url2 = alpm_types.Url(url_str)
    assert url1 == url2


def test_url_inequality() -> None:
    """Test URL inequality."""
    url1 = alpm_types.Url("https://example.com")
    url2 = alpm_types.Url("https://different.com")
    assert url1 != url2


def test_url_frozen() -> None:
    """Test that URL is frozen (immutable)."""
    url = alpm_types.Url("https://example.com")
    with pytest.raises(AttributeError):
        url.new_attr = "test"  # type: ignore[attr-defined]


@pytest.mark.parametrize("invalid_url", ["", "not_a_url", "http://", "lorem ipsum"])
def test_url_invalid(invalid_url: str) -> None:
    """Test URL error handling for invalid URLs."""
    with pytest.raises(ALPMError):
        alpm_types.Url(invalid_url)
