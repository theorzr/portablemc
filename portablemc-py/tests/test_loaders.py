"""Tests of the Fabric and Forge installers."""

import pytest

from portablemc import base, mojang, fabric, forge


def test_fabric_installer():
    installer = fabric.Installer(fabric.Loader.Quilt)
    assert installer.loader == fabric.Loader.Quilt
    assert installer.game_version == fabric.GameVersion.Stable
    assert installer.loader_version == fabric.LoaderVersion.Stable
    assert isinstance(installer, mojang.Installer)
    installer.game_version = "1.20.1"
    installer.loader_version = fabric.LoaderVersion.Unstable
    assert installer.game_version == "1.20.1"
    assert installer.loader_version == fabric.LoaderVersion.Unstable


@pytest.mark.parametrize("name", [
    "LatestVersionNotFoundError", "GameVersionNotFoundError", "LoaderVersionNotFoundError",
])
def test_fabric_errors_hierarchy(name: str):
    assert issubclass(getattr(fabric, name), base.Error)


def test_forge_installer():
    installer = forge.Installer(forge.Loader.Forge, forge.Version.Stable("1.20.1"))
    assert installer.loader == forge.Loader.Forge
    assert isinstance(installer.version, forge.Version.Stable)
    assert isinstance(installer.version, forge.Version)
    assert isinstance(installer, mojang.Installer)
    installer.version = forge.Version.Name("21.0.1")
    assert installer.version == forge.Version.Name("21.0.1")


@pytest.mark.parametrize("name", [
    "LatestVersionNotFoundError", "InstallerNotFoundError", "InstallerProcessorFailedError",
])
def test_forge_errors_hierarchy(name: str):
    assert issubclass(getattr(forge, name), base.Error)


@pytest.mark.network
def test_fabric_api():
    api = fabric.Api(fabric.Loader.Fabric)
    game_versions = api.request_game_versions()
    assert any(v.stable for v in game_versions)
    loader_versions = api.request_loader_versions(game_versions[0].name)
    assert len(loader_versions) > 0


@pytest.mark.network
def test_forge_repo():
    repo = forge.Repo.request(forge.Loader.Forge)
    assert repo.loader == forge.Loader.Forge
    version = repo.find_latest("1.20.1")
    assert version is not None
    assert version.game_version == "1.20.1"
    assert repo.find_by_name(version.name).name == version.name
