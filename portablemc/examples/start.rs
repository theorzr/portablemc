//! Install and launch a Mojang version in offline mode, in the default main directory.
//!
//! ```sh
//! cargo run --example start -- [version] [username]
//! ```

use std::error::Error;
use std::env;

use portablemc::moj;


fn main() -> Result<(), Box<dyn Error>> {

    let mut args = env::args().skip(1);

    let mut installer = match args.next() {
        Some(version) => moj::Installer::new(version),
        None => moj::Installer::new_with_release(),
    };

    if let Some(username) = args.next() {
        installer.set_auth_offline_username(username);
    }

    // The unit handler ignores all installation events.
    let game = installer.install(())?;
    let status = game.spawn_and_wait()?;
    println!("Game exited with {status}");

    Ok(())

}
