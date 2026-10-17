"""List the latest Mojang releases, and the versions supported by Fabric and Forge.

    python search.py
"""

from itertools import islice

from portablemc import base, mojang, fabric, forge


def main():

    manifest = mojang.Manifest.request()
    print(f"Latest release: {manifest.latest_release_name}")
    print(f"Latest snapshot: {manifest.latest_snapshot_name}")

    print("Last 5 releases:")
    releases = (v for v in manifest if v.channel == base.VersionChannel.Release)
    for version in islice(releases, 5):
        print(f"  {version.name} ({version.release_time.date()})")

    api = fabric.Api(fabric.Loader.Fabric)
    latest_game = next(v for v in api.request_game_versions() if v.stable)
    latest_loader = next(v for v in api.request_loader_versions(latest_game.name) if v.stable)
    print(f"Latest Fabric: {latest_loader.name} for {latest_game.name}")

    repo = forge.Repo.request(forge.Loader.Forge)
    version = repo.find_latest(manifest.latest_release_name)
    if version is not None:
        print(f"Latest Forge: {version.name}")
    else:
        print(f"No stable Forge for {manifest.latest_release_name}")


if __name__ == "__main__":
    main()
