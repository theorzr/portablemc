//! Authenticate with a Microsoft account and launch the latest release with it. The
//! account is stored in a database file in the working directory, so that subsequent
//! launches with the same username don't need to authenticate again.
//!
//! ```sh
//! cargo run --example msa -- [username]
//! ```

use std::error::Error;
use std::env;

use portablemc::{moj, msa};


/// The Azure application ID of PortableMC, you should register your own application
/// for your launcher.
const AZURE_APP_ID: &str = "708e91b5-99f8-4a1d-80ec-e746cbb24771";


fn main() -> Result<(), Box<dyn Error>> {

    let db = msa::Database::new("portablemc_msa.json");

    let stored_account = match env::args().nth(1) {
        Some(username) => db.load_from_username(&username)?,
        None => None,
    };

    let account = match stored_account {
        Some(mut account) => {
            // Check that the account is still valid, refreshing its token if needed.
            match account.request_profile() {
                Ok(()) => {}
                Err(msa::AuthError::OutdatedToken) => account.request_refresh()?,
                Err(e) => return Err(e.into()),
            }
            account
        }
        None => {
            let flow = msa::Auth::new(AZURE_APP_ID).request_device_code()?;
            println!("{}", flow.message());
            flow.wait()?
        }
    };

    println!("Authenticated as {} ({})", account.username(), account.uuid());
    db.store(account.clone())?;

    let mut installer = moj::Installer::new_with_release();
    installer.set_auth_msa(&account);
    installer.install(())?.spawn_and_wait()?;

    Ok(())

}
