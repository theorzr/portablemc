"""Private entry point of the embedded PortableMC command line interface."""

import sys

from ._portablemc import _cli_main  # type: ignore


def main() -> None:
    # Always naming the program 'portablemc', whether it's run as a script or a module.
    sys.exit(_cli_main(["portablemc", *sys.argv[1:]]))
