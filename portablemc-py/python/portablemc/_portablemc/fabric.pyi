from typing_extensions import Self
from enum import Enum, auto

from . import mojang, base


class Loader(Enum):
    Fabric = auto()
    Quilt = auto()
    LegacyFabric = auto()
    Babric = auto()


class GameVersion(Enum):
    Stable = auto()
    Unstable = auto()


class LoaderVersion(Enum):
    Stable = auto()
    Unstable = auto()


class Installer(mojang.Installer):

    def __new__(cls, loader: Loader, game_version: str | GameVersion = GameVersion.Stable, loader_version: str | LoaderVersion = LoaderVersion.Stable) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader: ...
    @loader.setter
    def loader(self, loader: Loader): ...

    @property
    def game_version(self) -> str | GameVersion: ...
    @game_version.setter
    def game_version(self, game_version: str | GameVersion): ...

    @property
    def loader_version(self) -> str | LoaderVersion: ...
    @loader_version.setter
    def loader_version(self, loader_version: str | LoaderVersion): ...

    def install(self, handler: base.Handler | None = None) -> base.Game: ...


class Handler(mojang.Handler):

    def fetch_loader_version(self, game_version: str, loader_version: str) -> None: ...
    def fetched_loader_version(self, game_version: str, loader_version: str) -> None: ...


class Api:
    """A Fabric-compatible API, used to list the game and loader versions it supports."""

    def __new__(cls, loader: Loader) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader: ...

    def request_game_versions(self) -> list[ApiGameVersion]:
        """Request the supported game versions, the most recent first."""

    def request_loader_versions(self, game_version: str | None = None) -> list[ApiLoaderVersion]:
        """Request the loader versions, optionally for a specific game version, the
        most recent first."""


class ApiGameVersion:

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class ApiLoaderVersion:

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class LatestVersionNotFoundError(base.Error):
    game_version: str | None
    """The game version if the loader version was not found, none for game version."""
    stable: bool

class GameVersionNotFoundError(base.Error):
    game_version: str

class LoaderVersionNotFoundError(base.Error):
    game_version: str
    loader_version: str
