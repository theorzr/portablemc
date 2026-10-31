from typing import Iterator
from typing_extensions import Self
from uuid import UUID
from os import PathLike
from pathlib import Path


class Auth:
    """Microsoft account authenticator."""

    def __new__(cls, app_id: str) -> Self:
        """Create a new authenticator with the given application (client) id."""

    def __repr__(self) -> str: ...

    @property
    def app_id(self) -> str: ...

    @property
    def language_code(self) -> str | None:
        """The ISO 639 language code to use for localized messages."""
    @language_code.setter
    def language_code(self, code: str): ...

    def request_device_code(self) -> DeviceCodeFlow:
        """Request a device code and return the flow containing the user code and the
        verification URI, then wait it to get the authenticated account."""


class DeviceCodeFlow:
    """A device code authentication flow."""

    def __repr__(self) -> str: ...

    @property
    def app_id(self) -> str: ...
    @property
    def user_code(self) -> str:
        """The code the user should enter at the verification URI."""
    @property
    def verification_uri(self) -> str: ...
    @property
    def message(self) -> str:
        """A message to display to the user, with the user code and verification URI."""

    def wait(self) -> Account:
        """Wait for the user to authenticate, the GIL is released while waiting and it can
        be interrupted by signals such as Ctrl-C."""


class Account:
    """An authenticated Minecraft account."""

    def __repr__(self) -> str: ...

    @property
    def app_id(self) -> str:
        """The id of the application the account was authorized for."""
    @property
    def access_token(self) -> str:
        """The access token given to the game."""
    @property
    def uuid(self) -> UUID: ...
    @property
    def username(self) -> str: ...
    @property
    def xuid(self) -> str:
        """The Xbox user id."""

    def request_profile(self) -> None:
        """Request the account's profile, updating its username if it changed. Not
        needed for newly authenticated or refreshed accounts, if it fails the account
        may need to be refreshed."""

    def request_refresh(self) -> None:
        """Refresh the account using its refresh token, updating its username, UUID and
        access token."""


class Database:
    """A file-backed database storing accounts atomically."""

    def __new__(cls, file: str | PathLike[str]) -> Self:
        """Create a database at the given file, the file and its parent directory may
        not exist, nothing is loaded until requested."""

    def __repr__(self) -> str: ...

    @property
    def file(self) -> Path: ...

    def load_iter(self) -> Iterator[Account]:
        """Load all accounts of the database and iterate over them."""

    def load_from_uuid(self, uuid: UUID) -> Account | None:
        """Load the account with the given UUID, if present."""
    def load_from_username(self, username: str) -> Account | None:
        """Load the first account with the given username, if present, a username is
        not guaranteed to be unique."""

    def remove_from_uuid(self, uuid: UUID) -> Account | None:
        """Remove the account with the given UUID and return it, if present."""
    def remove_from_username(self, username: str) -> Account | None:
        """Remove the first account with the given username and return it, if present."""

    def store(self, account: Account) -> None:
        """Store the given account, replacing any account with the same UUID."""


class AuthError(Exception):
    """Base class for all authentication errors."""

class AuthDeclinedError(AuthError):
    """The user declined the authorization."""

class AuthTimedOutError(AuthError):
    """The authentication flow timed out."""

class AuthOutdatedTokenError(AuthError):
    """The token is outdated when requesting the profile, it can be refreshed."""

class AuthDoesNotOwnGameError(AuthError):
    """The account does not own the game."""

class AuthInvalidStatusError(AuthError):
    """An unexpected HTTP status has been received."""
    status: int


class DatabaseError(Exception):
    """Base class for all account database errors, the cause is set to an `OSError`
    for I/O errors."""

class DatabaseCorruptedError(DatabaseError):
    """The database is corrupted, the file can be moved to a backup location before
    retrying."""

class DatabaseWriteFailedError(DatabaseError):
    """The database could not be written."""
