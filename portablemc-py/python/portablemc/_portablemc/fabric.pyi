from typing_extensions import Self
from enum import Enum, auto

from . import moj, base


class Loader(Enum):
    """The Fabric-like mod loaders."""

    Fabric = auto()
    """The official Fabric loader."""
    Quilt = auto()
    """The Quilt loader, a fork of Fabric."""
    LegacyFabric = auto()
    """The LegacyFabric project, backporting Fabric to older versions, up to 1.14
    snapshots."""
    Babric = auto()
    """The Babric project, supporting Fabric for beta 1.7.3."""
    OrnitheFabric = auto()
    """The Ornithe project, supporting Fabric on old versions with its own (gen2)
    intermediary mappings."""
    OrnitheQuilt = auto()
    """Same as `OrnitheFabric` but for the Quilt loader."""


class GameVersion(Enum):
    """An alias to the latest game version supported by the loader."""

    Stable = auto()
    """The latest stable game version, usually the latest release."""
    Unstable = auto()
    """The latest stable or unstable game version, usually the latest snapshot."""


class LoaderVersion(Enum):
    """An alias to the latest loader version for the game version."""

    Stable = auto()
    """The latest stable loader version."""
    Unstable = auto()
    """The latest stable or unstable loader version."""


class Installer(moj.Installer):
    """An installer for Fabric-like mod loaders. The `version`, `manifest_url` and
    `manifest_prefix` properties are overwritten when installing."""

    def __new__(cls, loader: Loader, game_version: str | GameVersion = GameVersion.Stable, loader_version: str | LoaderVersion = LoaderVersion.Stable) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader:
        """The loader to install."""
    @loader.setter
    def loader(self, loader: Loader): ...

    @property
    def game_version(self) -> str | GameVersion:
        """The game version to install the loader for."""
    @game_version.setter
    def game_version(self, game_version: str | GameVersion): ...

    @property
    def loader_version(self) -> str | LoaderVersion:
        """The loader version to install."""
    @loader_version.setter
    def loader_version(self, loader_version: str | LoaderVersion): ...

    def install(self, handler: base.Handler | None = None) -> base.Game: ...


class Handler(moj.Handler):
    """Handler for Fabric installation events, see `moj.Handler`."""

    def fetch_loader_version(self, game_version: str, loader_version: str) -> None:
        """The loader version metadata will be fetched."""
    def fetched_loader_version(self, game_version: str, loader_version: str) -> None:
        """The loader version metadata has been fetched."""


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
    """A game version supported by a Fabric-compatible API."""

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class ApiLoaderVersion:
    """A loader version of a Fabric-compatible API."""

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class LatestVersionNotFoundError(base.Error):
    """No game or loader version matches the requested stable or unstable alias."""
    game_version: str | None
    """The game version if the loader version was not found, none for game version."""
    stable: bool

class GameVersionNotFoundError(base.Error):
    """The game version is not supported by the loader."""
    game_version: str

class LoaderVersionNotFoundError(base.Error):
    """The loader version is not supported by the loader for the game version."""
    game_version: str
    loader_version: str
