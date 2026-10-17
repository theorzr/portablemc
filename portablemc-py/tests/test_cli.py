"""Tests of the embedded command line interface."""

import subprocess
import sys


def run(*args: str) -> subprocess.CompletedProcess:
    return subprocess.run([sys.executable, "-m", "portablemc", *args], capture_output=True, text=True)


def test_help():
    res = run("--help")
    assert res.returncode == 0
    assert "Usage: portablemc" in res.stdout


def test_invalid_argument():
    assert run("--does-not-exist").returncode == 2
