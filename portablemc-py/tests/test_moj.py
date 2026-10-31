"""Tests of the Mojang installer."""

from pathlib import Path
from uuid import UUID

import pytest

from portablemc import base, moj

from helpers import MainDir, RecordHandler


MINIMAL = {
    "id": "minimal",
    "mainClass": "net.minecraft.client.main.Main",
    "arguments": {
        "game": [
            "--username", "${auth_player_name}",
            "--uuid", "${auth_uuid}",
            {
                "rules": [{"action": "allow", "features": {"is_demo_user": True}}],
                "value": "--demo",
            },
        ],
    },
}


def test_version():
    assert moj.Installer().version == moj.Version.Release
    assert moj.Installer(moj.Version.Snapshot).version == moj.Version.Snapshot
    installer = moj.Installer("1.20")
    assert installer.version == "1.20"
    assert isinstance(installer, base.Installer)


def test_auth_offline():
    installer = moj.Installer()
    installer.set_auth_offline(UUID(int=1), "Alex")
    assert installer.auth_uuid == UUID(int=1)
    assert installer.auth_username == "Alex"
    # The UUID is derived from the username, so it is stable.
    installer.set_auth_offline_username("Steve")
    uuid = installer.auth_uuid
    installer.set_auth_offline_username("Steve")
    assert installer.auth_uuid == uuid
    assert installer.auth_username == "Steve"


def test_properties():
    installer = moj.Installer()
    assert installer.resolution is None
    installer.resolution = (800, 600)
    assert installer.resolution == (800, 600)
    installer.demo = True
    assert installer.demo
    installer.fix_lwjgl = "3.3.3"
    assert installer.fix_lwjgl == "3.3.3"
    installer.fix_lwjgl = None
    assert installer.fix_lwjgl is None


def test_quick_play():
    installer = moj.Installer()
    assert installer.quick_play is None
    installer.quick_play = moj.QuickPlay.Multiplayer("localhost", 25565)
    quick_play = installer.quick_play
    assert isinstance(quick_play, moj.QuickPlay.Multiplayer)
    assert isinstance(quick_play, moj.QuickPlay)
    assert (quick_play.host, quick_play.port) == ("localhost", 25565)
    installer.quick_play = moj.QuickPlay.Path("world")
    assert installer.quick_play.path == Path("world")


def test_fetch_excludes():
    installer = moj.Installer()
    assert installer.fetch_excludes == []
    installer.fetch_excludes = [moj.FetchExclude.Exact("a")]
    installer.add_fetch_exclude(moj.FetchExclude.Regex("b.*"))
    exact, regex = installer.fetch_excludes
    assert exact.name == "a"
    assert regex.pattern == "b.*"
    installer.clear_fetch_exclude()
    assert installer.fetch_excludes == []


def test_local_version(main_dir: MainDir):
    main_dir.write_version("minimal", MINIMAL)
    main_dir.write_file("versions/minimal/minimal.jar")
    installer = main_dir.moj_installer("minimal")
    installer.set_auth_offline(UUID(int=1), "Alex")
    installer.demo = True
    game = installer.install(RecordHandler())
    assert game.game_args == [
        "--username", "Alex",
        "--uuid", "00000000000000000000000000000001",
        "--demo",
    ]


def test_fetch_excluded(main_dir: MainDir):
    with pytest.raises(base.VersionNotFoundError):
        main_dir.moj_installer("missing").install(RecordHandler())


@pytest.mark.network
def test_manifest():
    manifest = moj.Manifest.request()
    assert len(manifest) > 0
    release = manifest.find_by_name(manifest.latest_release_name)
    assert release is not None
    assert release.channel == base.VersionChannel.Release
    assert manifest.find_by_name("does not exist") is None
    assert manifest[0].name == next(iter(manifest)).name


@pytest.mark.network
def test_install_release(main_dir: MainDir):
    installer = moj.Installer()
    main_dir.setup(installer)
    handler = RecordHandler()
    with pytest.raises(base.DownloadResourcesCancelledError):
        installer.install(handler)
    assert "fetch_version" in handler.names()
