from typing import Iterator
from typing_extensions import Self
from os import PathLike
from datetime import datetime
import pathlib
from uuid import UUID
from enum import Enum, auto

from . import base, msa


class Installer(base.Installer):
    """An installer for Mojang-provided versions, it supports standard arguments such as
    demo mode, window resolution and quick play, and provides fixes for known issues of
    old versions. Versions missing from the hierarchy are fetched from the versions
    manifest, if present in it."""

    def __new__(cls, version: str | Version = Version.Release) -> Self:
        """Create a new installer for the given version, with all fixes enabled except
        LWJGL, and missing versions fetching enabled."""

    def __repr__(self) -> str: ...

    @property  # type: ignore[override]
    def version(self) -> str | Version:  # pyright: ignore[reportIncompatibleMethodOverride]
        """The version to install, a specific version that is not provided by Mojang
        must be already installed in the versions directory."""
    @version.setter
    def version(self, version: str | Version): ...

    @property
    def fetch_excludes(self) -> list[FetchExclude]:
        """The filters excluding versions from being fetched, empty by default."""
    @fetch_excludes.setter
    def fetch_excludes(self, excludes: list[FetchExclude]): ...

    def add_fetch_exclude(self, exclude: FetchExclude) -> None:
        """Append the given filter to the fetch excludes."""
    def clear_fetch_exclude(self) -> None:
        """Clear all fetch excludes."""

    @property
    def demo(self) -> bool:
        """Enable the demo mode of the game."""
    @demo.setter
    def demo(self, demo: bool): ...

    @property
    def quick_play(self) -> QuickPlay | None:
        """The Quick Play mode when launching the game, from 1.20 (23w14a), none by
        default."""
    @quick_play.setter
    def quick_play(self, quick_play: QuickPlay | None): ...

    @property
    def resolution(self) -> tuple[int, int] | None:
        """The initial (width, height) of the game's window, none by default."""
    @resolution.setter
    def resolution(self, resolution: tuple[int, int] | None): ...

    @property
    def disable_multiplayer(self) -> bool:
        """Disable the multiplayer when launching the game."""
    @disable_multiplayer.setter
    def disable_multiplayer(self, disable_multiplayer: bool): ...

    @property
    def disable_chat(self) -> bool:
        """Disable the chat when launching the game."""
    @disable_chat.setter
    def disable_chat(self, disable_chat: bool): ...

    @property
    def auth_uuid(self) -> UUID:
        """The configured authentication UUID, nil if not configured, in which case
        `set_auth_offline_hostname` is used when installing."""
    @property
    def auth_username(self) -> str:
        """The configured authentication username, empty if not configured, in which
        case `set_auth_offline_hostname` is used when installing."""

    def set_auth_offline(self, uuid: UUID, username: str) -> None:
        """Use an offline session with the given UUID and username, the username is
        truncated to 16 bytes."""
    def set_auth_offline_uuid(self, uuid: UUID) -> None:
        """Use an offline session with the given UUID, the username is derived from the
        first 8 characters of the UUID."""
    def set_auth_offline_username(self, username: str) -> None:
        """Use an offline session with the given username (truncated to 16 characters),
        the UUID is derived from it like Mojang's authlib does, so it is the same UUID
        as when joining an offline-mode server."""
    def set_auth_offline_username_legacy(self, username: str) -> None:
        """Use an offline session with the given username (truncated to 16 characters),
        the UUID is derived from it with a PortableMC-specific derivation."""
    def set_auth_offline_hostname(self) -> None:
        """Use an offline session with a UUID derived from this machine's hostname, the
        username is then derived from the UUID, this is the default authentication."""
    def set_auth_msa(self, account: msa.Account) -> None:
        """Use online authentication with the given Microsoft account."""

    @property
    def client_id(self) -> str:
        """The client id used for the game's telemetry, empty by default."""
    @client_id.setter
    def client_id(self, client_id: str): ...

    @property
    def fix_legacy_quick_play(self) -> bool:
        """Use legacy arguments for multiplayer Quick Play on versions before 1.20
        (23w14a), enabled by default."""
    @fix_legacy_quick_play.setter
    def fix_legacy_quick_play(self, fix: bool): ...

    @property
    def fix_legacy_proxy(self) -> bool:
        """Use a proxy for legacy online resources, such as skins, on alpha, beta and
        release up to 1.5, enabled by default."""
    @fix_legacy_proxy.setter
    def fix_legacy_proxy(self, fix: bool): ...

    @property
    def fix_legacy_merge_sort(self) -> bool:
        """Use the legacy merge sort on alpha and beta versions to avoid crashes,
        enabled by default."""
    @fix_legacy_merge_sort.setter
    def fix_legacy_merge_sort(self, fix: bool): ...

    @property
    def fix_legacy_resolution(self) -> bool:
        """Add arguments to force the initial window resolution on versions that don't
        support it, enabled by default."""
    @fix_legacy_resolution.setter
    def fix_legacy_resolution(self, fix: bool): ...

    @property
    def fix_broken_authlib(self) -> bool:
        """Replace authlib 2.1.28 of versions 1.16.4 and 1.16.5, which disables the
        multiplayer button, enabled by default."""
    @fix_broken_authlib.setter
    def fix_broken_authlib(self, fix: bool): ...

    @property
    def fix_lwjgl(self) -> str | None:
        """Change the LWJGL version, from 3.2.3 and with ARM support when available,
        it's not guaranteed to work with every game version, none by default."""
    @fix_lwjgl.setter
    def fix_lwjgl(self, lwjgl_version: str | None): ...

    @property
    def manifest_url(self) -> str | None:
        """The URL of the version manifest used to find missing versions and check if
        versions need to be updated. By default, this targets the public Mojang
        manifest, but it can be used to specify a mirror or other manifests."""
    @manifest_url.setter
    def manifest_url(self, manifest_url: str | None): ...

    @property
    def manifest_prefix(self) -> str | None:
        """The prefix to set for all the versions of the configured manifest, only
        versions starting with this prefix are fetched from the manifest. For example
        with prefix `foo-`, version `foo-1.7.10` is fetched as `1.7.10` from the
        manifest but stored under `<versions_dir>/foo-1.7.10/`. The prefix is also
        added to the latest release and snapshot, but a named version must include it."""
    @manifest_prefix.setter
    def manifest_prefix(self, manifest_prefix: str | None): ...

    def install(self, handler: base.Handler | None = None) -> base.Game: ...


class Handler(base.Handler):
    """Handler for Mojang installation events, see `base.Handler`."""

    def invalidated_version(self, version: str) -> None:
        """The version metadata is invalid (size, SHA-1...) and has been removed to be
        fetched again."""
    def fetch_version(self, version: str) -> None:
        """The version metadata is missing and will be fetched."""
    def fetched_version(self, version: str) -> None:
        """The version metadata has been fetched."""
    def fixed_legacy_quick_play(self) -> None:
        """Legacy Quick Play arguments have been added."""
    def fixed_legacy_proxy(self, host: str, port: int) -> None:
        """The legacy proxy has been set."""
    def fixed_legacy_merge_sort(self) -> None:
        """The legacy merge sort has been enabled."""
    def fixed_legacy_resolution(self) -> None:
        """Legacy resolution arguments have been added."""
    def fixed_broken_authlib(self) -> None:
        """The broken authlib has been replaced."""
    def warn_unsupported_quick_play(self) -> None:
        """Quick Play is requested but not supported by this version, or the fix is
        disabled."""
    def warn_unsupported_resolution(self) -> None:
        """A resolution is requested but not supported by this version and the fix is
        disabled."""


class Version(Enum):
    """An alias to the latest version of a channel."""

    Release = auto()
    """The latest release."""
    Snapshot = auto()
    """The latest snapshot."""


class QuickPlay:
    """The Quick Play mode when launching the game, usually unsupported before 1.20
    (23w14a), except multiplayer with the legacy fix."""

    class Path(QuickPlay):
        """Follow the Quick Play instructions in the given path, relative to the
        working directory."""
        path: pathlib.Path
        def __new__(cls, path: str | PathLike[str]) -> Self: ...
    class Singleplayer(QuickPlay):
        """Directly join the world with the given name."""
        name: str
        def __new__(cls, name: str) -> Self: ...
    class Multiplayer(QuickPlay):
        """Directly join the given server."""
        host: str
        port: int
        def __new__(cls, host: str, port: int) -> Self: ...
    class Realms(QuickPlay):
        """Directly join the realm with the given id."""
        id: str
        def __new__(cls, id: str) -> Self: ...


class FetchExclude:
    """A filter excluding versions from being fetched from the manifest."""

    class All(FetchExclude):
        """Exclude all versions from being fetched, it overrides all other excludes."""
        def __new__(cls) -> Self: ...
    class Exact(FetchExclude):
        """Exclude the version with this exact name."""
        name: str
        def __new__(cls, name: str) -> Self: ...
    class Regex(FetchExclude):
        """Exclude versions with a name matching this regex."""
        pattern: str
        def __new__(cls, pattern: str) -> Self: ...


class Manifest:
    """The Mojang versions manifest, it can be indexed and iterated over its versions."""

    @staticmethod
    def request(handler: base.Handler | None = None, url: str | None = None) -> Manifest:
        """Request the Mojang versions manifest, the handler's `download_progress`
        method is called if the manifest needs to be downloaded. The default Mojang
        manifest is requested if no URL is given."""

    def __repr__(self) -> str: ...
    def __len__(self) -> int: ...
    def __getitem__(self, index: int) -> ManifestVersion: ...
    def __iter__(self) -> Iterator[ManifestVersion]: ...

    @property
    def latest_release_name(self) -> str:
        """The latest release version name."""
    @property
    def latest_snapshot_name(self) -> str:
        """The latest snapshot version name."""

    def latest_name(self, key: str) -> str | None:
        """Return the latest version name for the given "latest" key, such as
        `release` or `snapshot`, or any other key present in the manifest."""

    def find_by_name(self, name: str) -> ManifestVersion | None:
        """Return the version with the given name, if present."""


class ManifestVersion:
    """A version of the Mojang versions manifest."""

    def __repr__(self) -> str: ...

    @property
    def name(self) -> str: ...
    @property
    def channel(self) -> base.VersionChannel: ...
    @property
    def time(self) -> datetime:
        """The last update time of this version."""
    @property
    def release_time(self) -> datetime: ...
    @property
    def url(self) -> str:
        """The download URL of the version metadata."""
    @property
    def size(self) -> int | None:
        """The expected size of the version metadata, if any."""
    @property
    def sha1(self) -> bytes | None:
        """The expected SHA-1 of the version metadata, if any."""


class LwjglFixNotFoundError(base.Error):
    """The LWJGL fix version is not supported, because it's too old (< 3.2.3) or not
    available for this system."""
    version: str
