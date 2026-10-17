from pathlib import Path

import pytest

from helpers import MainDir


@pytest.fixture
def main_dir(tmp_path: Path) -> MainDir:
    return MainDir(tmp_path)
