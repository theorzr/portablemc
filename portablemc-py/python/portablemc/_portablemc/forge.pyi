from typing import Iterator
from typing_extensions import Self
from pathlib import Path
from enum import Enum, auto

from . import moj, base


class Loader(Enum):
    """The Forge-like mod loaders."""

    Forge = auto()
    NeoForge = auto()


class Version:
    """The loader version to install."""

    class Stable(Version):
        """The latest stable loader version for the given game version."""
        def __new__(cls, game_version: str) -> Self: ...
    class Unstable(Version):
        """The latest stable or unstable loader version for the given game version."""
        def __new__(cls, game_version: str) -> Self: ...
    class Name(Version):
        """The full loader version name."""
        def __new__(cls, name: str) -> Self: ...


class Installer(moj.Installer):
    """An installer for Forge and NeoForge mod loaders. The `version` property is
    overwritten, and the `manifest_url` and `manifest_prefix` properties are removed
    when installing."""

    def __new__(cls, loader: Loader, version: Version) -> Self: ...

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader:
        """The loader to install."""
    @loader.setter
    def loader(self, loader: Loader): ...

    @property  # type: ignore[override]
    def version(self) -> Version:  # pyright: ignore[reportIncompatibleMethodOverride]
        """The loader version to install."""
    @version.setter
    def version(self, version: Version): ...  # pyright: ignore[reportIncompatibleMethodOverride]

    def install(self, handler: base.Handler | None = None) -> base.Game: ...


class Handler(moj.Handler):
    """Handler for Forge installation events, see `moj.Handler`."""

    def installing(self, tmp_dir: Path, reason: InstallReason) -> None:
        """The loader is not (properly) installed and will be installed, using the given
        temporary directory."""
    def fetch_installer(self, version: str) -> None:
        """The loader installer will be fetched."""
    def fetched_installer(self, version: str) -> None:
        """The loader installer has been fetched."""
    def installing_game(self) -> None:
        """The game will be installed because the loader installer needs it."""
    def fetch_installer_libraries(self) -> None:
        """The loader installer libraries will be downloaded or extracted."""
    def fetched_installer_libraries(self) -> None:
        """The loader installer libraries have been downloaded or extracted."""
    def run_installer_processor(self, name: str, task: str | None) -> None:
        """An installer processor will be run."""
    def installed(self) -> None:
        """The loader has been installed, its installation will be checked again."""


class InstallReason(Enum):
    """The reason for (re)installing the loader."""

    MissingVersionMetadata = auto()
    """The version metadata is missing, the loader was probably not installed."""
    MissingCoreLibrary = auto()
    """The core library, which should be extracted from the installer, is missing."""
    MissingClientExtra = auto()
    """The client extra artifact is missing."""
    MissingClientSrg = auto()
    """The client srg artifact is missing."""
    MissingPatchedClient = auto()
    """The patched client is missing."""
    MissingUniversalClient = auto()
    """The universal client is missing."""


class Repo:
    """The versions repository of Forge or NeoForge."""

    @staticmethod
    def request(loader: Loader) -> Repo:
        """Request the online versions repository of the given loader."""

    def __repr__(self) -> str: ...

    @property
    def loader(self) -> Loader: ...

    def __iter__(self) -> Iterator[RepoVersion]:
        """Iterate over all loader versions, the order is not consistent between Forge
        and NeoForge."""

    def find_by_name(self, name: str) -> RepoVersion | None:
        """Return the loader version with this exact name, if present."""
    def find_latest(self, game_version: str, stable: bool = True) -> RepoVersion | None:
        """Return the latest stable or unstable loader version for the game version, if
        any. The latest stable version is also the latest unstable one if no unstable
        version is more recent."""


class RepoVersion:
    """A loader version of the repository."""

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str:
        """The full loader version name, containing both game and loader versions."""
    @property
    def game_version(self) -> str: ...
    @property
    def stable(self) -> bool: ...


class LatestVersionNotFoundError(base.Error):
    """No loader version matches the requested stable or unstable alias for the game
    version."""
    game_version: str
    stable: bool

class InstallerNotFoundError(base.Error):
    """No installer is found for the loader version."""
    version: str

class MavenMetadataMalformedError(base.Error):
    """The repository 'maven-metadata.xml' file is malformed."""

class InstallerProfileNotFoundError(base.Error):
    """The installer 'install_profile.json' file is not found."""

class InstallerProfileIncoherentError(base.Error):
    """The installer 'install_profile.json' file has versions incoherent with the
    expected loader and game versions."""

class InstallerVersionMetadataNotFoundError(base.Error):
    """The installer 'version.json' file, containing the version metadata, is not
    found."""

class InstallerFileNotFoundError(base.Error):
    """A file to extract from the installer is not found."""
    entry: str

class InstallerProcessorNotFoundError(base.Error):
    """An installer processor library is not found."""
    name: str

class InstallerProcessorMainClassNotFoundError(base.Error):
    """The main class of an installer processor is not found."""
    name: str

class InstallerProcessorDependencyNotFoundError(base.Error):
    """A dependency of an installer processor is not found."""
    name: str
    dependency: str

class InstallerProcessorFailedError(base.Error):
    """An installer processor has failed, with its exit status and output."""
    name: str
    status: int | None
    stdout: bytes
    stderr: bytes

class InstallerProcessorCorruptedError(base.Error):
    """A file produced by an installer processor has an unexpected SHA-1."""
    name: str
    file: Path
    expected_sha1: bytes
