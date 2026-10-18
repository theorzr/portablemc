"""Authenticate with a Microsoft account and launch the latest release with it. The
account is stored in a database file in the working directory, so that subsequent
launches with the same username don't need to authenticate again.

    python msa.py [username]
"""

import sys

from portablemc import mojang, msa


# The Azure application ID of PortableMC, you should register your own application
# for your launcher.
AZURE_APP_ID = "708e91b5-99f8-4a1d-80ec-e746cbb24771"


def main():

    db = msa.Database("portablemc_msa.json")

    account = None
    if len(sys.argv) > 1:
        account = db.load_from_username(sys.argv[1])

    if account is not None:
        # Check that the account is still valid, refreshing its token if needed.
        try:
            account.request_profile()
        except msa.AuthOutdatedTokenError:
            account.request_refresh()
    else:
        flow = msa.Auth(AZURE_APP_ID).request_device_code()
        print(flow.message)
        account = flow.wait()

    print(f"Authenticated as {account.username} ({account.uuid})")
    db.store(account)

    installer = mojang.Installer()
    installer.set_auth_msa(account)
    installer.install().command()().wait()


if __name__ == "__main__":
    main()
