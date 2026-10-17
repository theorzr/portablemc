"""Common utilities for the tests."""

from __future__ import annotations
from pathlib import Path
import json

from portablemc import base, mojang, fabric, forge


class MainDir:
    """A temporary main directory with a static JVM, so that no JVM is searched or
    downloaded."""

    def __init__(self, path: Path) -> None:
        # Resolved because the installer canonicalizes paths, the temporary directory
        # may be behind a symlink (such as on macOS).
        self.path = path.resolve()
        # The installer requires these to exist, even if no library or asset is used.
        (self.path / "libraries").mkdir()
        (self.path / "assets").mkdir()
        self.jvm_file = self.write_file("jvm/java")

    def write_version(self, name: str, metadata: dict) -> Path:
        """Write the metadata of a version and return its file."""
        file = self.path / "versions" / name / f"{name}.json"
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_text(json.dumps(metadata))
        return file

    def write_file(self, path: str) -> Path:
        """Create an empty file relative to the main directory and return it."""
        file = self.path / path
        file.parent.mkdir(parents=True, exist_ok=True)
        file.touch()
        return file

    def setup(self, installer: base.Installer) -> None:
        installer.set_main_dir(self.path)
        installer.jvm_policy = self.jvm_file

    def installer(self, version: str) -> base.Installer:
        installer = base.Installer(version)
        self.setup(installer)
        return installer

    def mojang_installer(self, version: str) -> mojang.Installer:
        """Return a Mojang installer that never fetches versions, only local ones."""
        installer = mojang.Installer(version)
        self.setup(installer)
        installer.add_fetch_exclude(mojang.FetchExclude.All())
        return installer


class RecordHandler(fabric.Handler, forge.Handler):
    """A handler for all installers that records the name and arguments of every called
    method, and that cancels any resources download so that no test can hit the network
    by mistake."""

    def __init__(self) -> None:
        self.events: list[tuple[str, tuple]] = []

    def __getattribute__(self, name: str):
        attr = object.__getattribute__(self, name)
        if name.startswith("_") or name in ("names", "args") or not callable(attr):
            return attr
        def record(*args):
            object.__getattribute__(self, "events").append((name, args))
            return attr(*args)
        return record

    def download_resources(self) -> bool:
        return True

    def names(self) -> list[str]:
        return [name for name, _ in self.events]

    def args(self, name: str) -> tuple:
        """Return the arguments of the first call of the given method."""
        return next(args for event_name, args in self.events if event_name == name)
