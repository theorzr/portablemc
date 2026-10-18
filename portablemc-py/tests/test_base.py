"""Tests of the base installer, using local versions metadata only."""

from pathlib import Path

import pytest

from portablemc import base

from helpers import MainDir, RecordHandler


# A version with no libraries, which can be fully installed without any download if its
# client is present.
MINIMAL = {
    "id": "minimal",
    "mainClass": "net.minecraft.client.main.Main",
    "arguments": {
        "jvm": ["-cp", "${classpath}"],
        "game": ["--username", "${auth_player_name}"],
    },
}

LIBRARIES = {
    "id": "libraries",
    "mainClass": "net.minecraft.client.main.Main",
    "libraries": [
        {"name": "mock:lib0:1.0.0", "url": "https://mock.invalid/"},
        {
            "name": "mock:lib1:1.0.0",
            "downloads": {
                "artifact": {
                    "path": "lib1-non-standard.jar",
                    "url": "https://mock.invalid/lib1.jar",
                    "size": 3,
                    "sha1": "0123456789abcdef0123456789abcdef01234567",
                },
            },
        },
        {"name": "mock:lib2:1.0.0", "url": "https://mock.invalid/", "rules": []},
    ],
}


@pytest.fixture
def minimal(main_dir: MainDir) -> base.Installer:
    """An installer for the minimal version, ready to be installed."""
    main_dir.write_version("minimal", MINIMAL)
    main_dir.write_file("versions/minimal/minimal.jar")
    return main_dir.installer("minimal")


@pytest.fixture
def libraries(main_dir: MainDir) -> base.Installer:
    main_dir.write_version("libraries", LIBRARIES)
    main_dir.write_file("versions/libraries/libraries.jar")
    return main_dir.installer("libraries")


def test_dirs():
    installer = base.Installer("foo")
    assert installer.version == "foo"
    installer.set_main_dir("main")
    assert installer.versions_dir == Path("main/versions")
    assert installer.libraries_dir == Path("main/libraries")
    assert installer.assets_dir == Path("main/assets")
    assert installer.mc_dir == Path("main")
    installer.mc_dir = Path("other")
    assert installer.mc_dir == Path("other")
    assert installer.versions_dir == Path("main/versions")


@pytest.mark.parametrize("policy", [
    base.JvmPolicy.System,
    base.JvmPolicy.Mojang,
    base.JvmPolicy.SystemThenMojang,
    base.JvmPolicy.MojangThenSystem,
])
def test_jvm_policy(policy: base.JvmPolicy):
    installer = base.Installer("foo")
    installer.jvm_policy = policy
    assert installer.jvm_policy == policy


def test_properties():
    installer = base.Installer("foo")
    installer.version = "bar"
    assert installer.version == "bar"
    installer.jvm_policy = "/usr/bin/java"
    assert installer.jvm_policy == Path("/usr/bin/java")
    installer.strict_assets_check = True
    assert installer.strict_assets_check
    installer.launcher_name = "test"
    assert installer.launcher_name == "test"
    assert "bar" in repr(installer)


def test_wrong_types():
    installer = base.Installer("foo")
    with pytest.raises(TypeError):
        installer.version = 1  # type: ignore
    with pytest.raises(TypeError):
        installer.jvm_policy = 1  # type: ignore


def test_library():
    download = base.LibraryDownload("https://mock.invalid/a.jar", 3, bytes(20))
    assert download.url == "https://mock.invalid/a.jar"
    assert download.size == 3
    assert download.sha1 == bytes(20)
    lib = base.LoadedLibrary("a:b:1.0", download=download)
    assert lib.name == "a:b:1.0"
    assert lib.path is None
    assert not lib.natives
    assert lib.download.url == download.url
    lib.path = "custom.jar"
    assert lib.path == Path("custom.jar")


@pytest.mark.parametrize("name", [
    "HierarchyLoopError", "VersionNotFoundError", "ClientNotFoundError",
    "DownloadResourcesCancelledError", "DownloadError", "InternalError",
])
def test_errors_hierarchy(name: str):
    assert issubclass(getattr(base, name), base.Error)


def test_default_main_dir():
    main_dir = base.default_main_dir()
    assert main_dir is None or isinstance(main_dir, Path)


def test_version_not_found(main_dir: MainDir):
    handler = RecordHandler()
    with pytest.raises(base.VersionNotFoundError) as exc:
        main_dir.installer("missing").install(handler)
    assert exc.value.version == "missing"
    assert handler.names()[-1] == "need_version"


def test_need_version_retry(main_dir: MainDir):

    class Handler(RecordHandler):
        def need_version(self, version: str, file: Path) -> bool:
            main_dir.write_version(version, {**MINIMAL, "id": version})
            return True

    main_dir.write_file("versions/late/late.jar")
    handler = Handler()
    main_dir.installer("late").install(handler)
    names = handler.names()
    assert names[names.index("need_version") + 1] == "loaded_version"


def test_hierarchy_loop(main_dir: MainDir):
    main_dir.write_version("recurse", {"id": "recurse", "inheritsFrom": "recurse"})
    with pytest.raises(base.HierarchyLoopError) as exc:
        main_dir.installer("recurse").install(RecordHandler())
    assert exc.value.version == "recurse"


def test_hierarchy(main_dir: MainDir):
    main_dir.write_version("parent", MINIMAL)
    main_dir.write_version("child", {"id": "child", "inheritsFrom": "parent"})
    main_dir.write_file("versions/child/child.jar")
    handler = RecordHandler()
    main_dir.installer("child").install(handler)
    hierarchy, = handler.args("loaded_hierarchy")
    assert [v.name for v in hierarchy] == ["child", "parent"]
    assert hierarchy[0].dir == main_dir.path / "versions" / "child"


def test_client_not_found(main_dir: MainDir):
    main_dir.write_version("noclient", {"id": "noclient"})
    with pytest.raises(base.ClientNotFoundError):
        main_dir.installer("noclient").install(RecordHandler())


def test_main_class_not_found(main_dir: MainDir):
    main_dir.write_version("nomain", {"id": "nomain"})
    main_dir.write_file("versions/nomain/nomain.jar")
    with pytest.raises(base.MainClassNotFoundError):
        main_dir.installer("nomain").install(RecordHandler())


def test_game(main_dir: MainDir, minimal: base.Installer):
    handler = RecordHandler()
    game = minimal.install(handler)

    assert handler.names() == [
        "filter_features", "loaded_features",
        "load_hierarchy", "load_version", "loaded_version", "loaded_hierarchy",
        "load_client", "loaded_client",
        "load_libraries", "filter_libraries", "loaded_libraries",
        "filter_libraries_files", "loaded_libraries_files",
        "no_logger", "no_assets",
        "load_jvm", "loaded_jvm",
        "extracted_binaries",
    ]

    assert game.jvm_file == main_dir.jvm_file
    assert game.mc_dir == main_dir.path
    assert game.main_class == "net.minecraft.client.main.Main"
    assert game.jvm_args[0] == "-cp"
    assert str(main_dir.path / "versions/minimal/minimal.jar") in game.jvm_args[1]
    assert game.game_args[0] == "--username"

    args = game.args()
    assert args[0] == main_dir.jvm_file
    assert args[1:] == [*game.jvm_args, game.main_class, *game.game_args]

    assert game.command().keywords["cwd"] == main_dir.path


def test_no_handler(minimal: base.Installer):
    minimal.install()


def test_partial_handler(minimal: base.Installer):
    """Any object can be used as handler, missing methods are an error."""
    with pytest.raises(AttributeError):
        minimal.install(object())  # type: ignore


def test_libraries(main_dir: MainDir, libraries: base.Installer):
    handler = RecordHandler()
    with pytest.raises(base.DownloadResourcesCancelledError):
        libraries.install(handler)

    libs, = handler.args("loaded_libraries")
    assert [lib.name for lib in libs] == ["mock:lib0:1.0.0", "mock:lib1:1.0.0"]
    assert libs[0].download.url == "https://mock.invalid/mock/lib0/1.0.0/lib0-1.0.0.jar"
    assert libs[1].path == Path("lib1-non-standard.jar")
    assert libs[1].download.size == 3
    assert libs[1].download.sha1 == bytes.fromhex("0123456789abcdef0123456789abcdef01234567")

    class_files, natives_files = handler.args("loaded_libraries_files")
    assert class_files == [
        main_dir.path / "libraries/mock/lib0/1.0.0/lib0-1.0.0.jar",
        main_dir.path / "libraries/lib1-non-standard.jar",
    ]
    assert natives_files == []


def test_filter_libraries(main_dir: MainDir, libraries: base.Installer):

    main_dir.write_file("libraries/custom.jar")
    extra_file = main_dir.write_file("extra.jar")

    class Handler(RecordHandler):
        def filter_libraries(self, libraries: list[base.LoadedLibrary]) -> None:
            libraries.clear()
            libraries.append(base.LoadedLibrary("custom:lib:1.0", "custom.jar"))
        def filter_libraries_files(self, class_files: list[Path], natives_files: list[Path]) -> None:
            class_files.append(extra_file)

    handler = Handler()
    game = libraries.install(handler)

    libs, = handler.args("loaded_libraries")
    assert [lib.name for lib in libs] == ["custom:lib:1.0"]
    class_files, _ = handler.args("loaded_libraries_files")
    assert class_files == [main_dir.path / "libraries/custom.jar", extra_file]
    assert "download_resources" not in handler.names()
    assert isinstance(game, base.Game)


def test_filter_libraries_wrong_type(libraries: base.Installer):

    class Handler(RecordHandler):
        def filter_libraries(self, libraries: list) -> None:
            libraries.append("not a library")

    with pytest.raises(TypeError):
        libraries.install(Handler())


def test_filter_features(minimal: base.Installer):

    class Handler(RecordHandler):
        def filter_features(self, features: set[str]) -> None:
            features.add("is_demo_user")

    handler = Handler()
    minimal.install(handler)
    assert handler.args("loaded_features") == ({"is_demo_user"},)


def test_handler_exception(minimal: base.Installer):
    """The first exception raised by the handler aborts the installation and no other
    method is called after it."""

    class Handler(RecordHandler):
        def load_client(self) -> None:
            raise ValueError("abort")

    handler = Handler()
    with pytest.raises(ValueError, match="abort"):
        minimal.install(handler)
    assert handler.names()[-1] == "load_client"
