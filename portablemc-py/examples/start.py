"""Install and launch a Mojang version in offline mode, in the default main directory.

    python start.py [version] [username]
"""

import sys

from portablemc import mojang


def main():

    installer = mojang.Installer()
    if len(sys.argv) > 1:
        installer.version = sys.argv[1]
    if len(sys.argv) > 2:
        installer.set_auth_offline_username(sys.argv[2])

    game = installer.install()
    process = game.command()()
    print(f"Game exited with {process.wait()}")


if __name__ == "__main__":
    main()
