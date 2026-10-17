"""Private entry point of the embedded PortableMC command line interface."""

import sys


def main() -> None:
    try:
        from ._portablemc import _cli_main
    except ImportError:
        # The native module can be built without the CLI, when it is packaged separately.
        sys.exit("portablemc: the command line interface is not included in this build")
    # Always naming the program 'portablemc', whether it's run as a script or a module.
    sys.exit(_cli_main(["portablemc", *sys.argv[1:]]))
