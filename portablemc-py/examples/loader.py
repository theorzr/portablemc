"""Install and launch the latest stable version of a mod loader, while printing the
installation progress with a custom handler.

    python loader.py <fabric|quilt|legacyfabric|babric|forge|neoforge> [game version]
"""

from __future__ import annotations
from pathlib import Path
import sys

from portablemc import base, mojang, fabric, forge


FABRIC_LOADERS = {
    "fabric": fabric.Loader.Fabric,
    "quilt": fabric.Loader.Quilt,
    "legacyfabric": fabric.Loader.LegacyFabric,
    "babric": fabric.Loader.Babric,
}

FORGE_LOADERS = {
    "forge": forge.Loader.Forge,
    "neoforge": forge.Loader.NeoForge,
}


class Printer(mojang.Handler):
    """Print a few relevant events common to all installers."""

    def loaded_hierarchy(self, hierarchy: list[base.LoadedVersion]) -> None:
        print("Loaded versions:", " -> ".join(v.name for v in hierarchy))

    def loaded_jvm(self, file: Path, version: str | None, compatible: bool) -> None:
        print(f"Loaded JVM {version or '?'} at {file}")

    def fetch_version(self, version: str) -> None:
        print(f"Fetching version {version}")

    def download_progress(self, count: int, total_count: int, size: int, total_size: int) -> None:
        print(f"\rDownloading {count}/{total_count} ({size / 1e6:.1f}/{total_size / 1e6:.1f} MB)",
            end="\n" if count == total_count else "", flush=True)


class FabricPrinter(Printer, fabric.Handler):

    def fetch_loader_version(self, game_version: str, loader_version: str) -> None:
        print(f"Fetching loader {loader_version} for {game_version}")


class ForgePrinter(Printer, forge.Handler):

    def installing(self, tmp_dir: Path, reason: forge.InstallReason) -> None:
        print(f"Installing loader ({reason})")

    def run_installer_processor(self, name: str, task: str | None) -> None:
        print(f"Running processor {name}")

    def installed(self) -> None:
        print("Loader installed")


def main():

    if len(sys.argv) < 2:
        sys.exit("missing loader argument")

    loader = sys.argv[1]
    game_version = sys.argv[2] if len(sys.argv) > 2 else None

    if loader in FABRIC_LOADERS:
        installer = fabric.Installer(FABRIC_LOADERS[loader], game_version or fabric.GameVersion.Stable)
        game = installer.install(FabricPrinter())
    elif loader in FORGE_LOADERS:
        # Forge versions are tied to a game version, so we need to know the latest
        # Mojang release if none is given.
        if game_version is None:
            game_version = mojang.Manifest.request().latest_release_name
        installer = forge.Installer(FORGE_LOADERS[loader], forge.Version.Stable(game_version))
        game = installer.install(ForgePrinter())
    else:
        sys.exit(f"unknown loader: {loader}")

    game.command()().wait()


if __name__ == "__main__":
    main()
