"""Tests of the Microsoft authentication, only what doesn't require an account."""

from pathlib import Path
from uuid import UUID

import pytest

from portablemc import msa


def test_auth():
    auth = msa.Auth("app")
    assert auth.app_id == "app"
    assert auth.language_code is None
    auth.language_code = "fr"
    assert auth.language_code == "fr"


@pytest.mark.parametrize("name", [
    "AuthDeclinedError", "AuthTimedOutError", "AuthOutdatedTokenError",
    "AuthDoesNotOwnGameError", "AuthInvalidStatusError",
])
def test_errors_hierarchy(name: str):
    assert issubclass(getattr(msa, name), msa.AuthError)


def test_database_missing(tmp_path: Path):
    db = msa.Database(tmp_path / "db.json")
    assert db.file == tmp_path / "db.json"
    assert list(db.load_iter()) == []
    assert db.load_from_username("Steve") is None
    assert db.load_from_uuid(UUID(int=0)) is None
    assert db.remove_from_username("Steve") is None


def test_database_corrupted(tmp_path: Path):
    file = tmp_path / "db.json"
    file.write_text("{not json")
    with pytest.raises(msa.DatabaseCorruptedError):
        list(msa.Database(file).load_iter())
    assert issubclass(msa.DatabaseCorruptedError, msa.DatabaseError)
