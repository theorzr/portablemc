from typing import Iterator
from typing_extensions import Self
from pathlib import Path
from enum import Enum, auto

from . import mojang, base


class Loader(Enum):
    Forge = auto()
    NeoForge = auto()


class Version:
    class Stable(Version):
        """The latest stable loader version for the given game version."""
        def __new__(cls, game_version: str) -> Self: ...
    class Unstable(Version):
        """The latest stable or unstable loader version for the given game version."""
        def __new__(cls, game_version: str) -> Self: ...
    class Name(Version):
        """The full loader version name."""
        def __new__(cls, name: str) -> Self: ...


class Installer(mojang.Installer):

    def __new__(cls, loader: Loader, version: Version) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader: ...
    @loader.setter
    def loader(self, loader: Loader): ...

    @mojang.Installer.version.getter
    def version(self) -> Version: ...
    @version.setter
    def version(self, version: Version): ...

    def install(self, handler: base.Handler | None = None) -> base.Game: ...


class Handler(mojang.Handler):

    def installing(self, tmp_dir: Path, reason: InstallReason) -> None: ...
    def fetch_installer(self, version: str) -> None: ...
    def fetched_installer(self, version: str) -> None: ...
    def installing_game(self) -> None: ...
    def fetch_installer_libraries(self) -> None: ...
    def fetched_installer_libraries(self) -> None: ...
    def run_installer_processor(self, name: str, task: str | None) -> None: ...
    def installed(self) -> None: ...


class InstallReason(Enum):
    MissingVersionMetadata = auto()
    MissingCoreLibrary = auto()
    MissingClientExtra = auto()
    MissingClientSrg = auto()
    MissingPatchedClient = auto()
    MissingUniversalClient = auto()


class Repo:
    """The versions repository of Forge or NeoForge."""

    @staticmethod
    def request(loader: Loader) -> Repo: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader: ...

    def __iter__(self) -> Iterator[RepoVersion]:
        """Iterate over all loader versions, the order is not consistent between Forge
        and NeoForge."""

    def find_by_name(self, name: str) -> RepoVersion | None: ...
    def find_latest(self, game_version: str, stable: bool = True) -> RepoVersion | None: ...


class RepoVersion:

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def game_version(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class LatestVersionNotFoundError(base.Error):
    game_version: str
    stable: bool

class InstallerNotFoundError(base.Error):
    version: str

class MavenMetadataMalformedError(base.Error): ...

class InstallerProfileNotFoundError(base.Error): ...

class InstallerProfileIncoherentError(base.Error): ...

class InstallerVersionMetadataNotFoundError(base.Error): ...

class InstallerFileNotFoundError(base.Error):
    entry: str

class InstallerProcessorNotFoundError(base.Error):
    name: str

class InstallerProcessorMainClassNotFoundError(base.Error):
    name: str

class InstallerProcessorDependencyNotFoundError(base.Error):
    name: str
    dependency: str

class InstallerProcessorFailedError(base.Error):
    name: str
    status: int | None
    stdout: bytes
    stderr: bytes

class InstallerProcessorCorruptedError(base.Error):
    name: str
    file: Path
    expected_sha1: bytes
