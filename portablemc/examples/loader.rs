//! Install and launch the latest stable version of a mod loader, while printing the
//! installation progress with a custom handler.
//!
//! ```sh
//! cargo run --example loader -- <fabric|quilt|legacyfabric|babric|forge|neoforge> [game version]
//! ```

use std::error::Error;
use std::env;

use portablemc::{base, moj, fabric, forge};


fn main() -> Result<(), Box<dyn Error>> {

    let mut args = env::args().skip(1);
    let loader = args.next().ok_or("missing loader argument")?;
    let game_version = args.next();

    let game = match loader.as_str() {
        "fabric" | "quilt" | "legacyfabric" | "babric" => {

            let loader = match loader.as_str() {
                "fabric" => fabric::Loader::Fabric,
                "quilt" => fabric::Loader::Quilt,
                "legacyfabric" => fabric::Loader::LegacyFabric,
                _ => fabric::Loader::Babric,
            };

            let game_version = match game_version {
                Some(name) => fabric::GameVersion::Name(name),
                None => fabric::GameVersion::Stable,
            };

            let mut installer = fabric::Installer::new(loader, game_version, fabric::LoaderVersion::Stable);
            installer.install(&mut Printer)?

        }
        "forge" | "neoforge" => {

            let loader = match loader.as_str() {
                "forge" => forge::Loader::Forge,
                _ => forge::Loader::NeoForge,
            };

            // Forge versions are tied to a game version, so we need to know the latest
            // Mojang release if none is given.
            let game_version = match game_version {
                Some(name) => name,
                None => moj::Manifest::request(())?.latest_release_name().to_string(),
            };

            let mut installer = forge::Installer::new(loader, forge::Version::Stable(game_version));
            installer.install(&mut Printer)?

        }
        "ornithe" => {

            let mut installer = moj::Installer::new(match game_version {
                Some(name) => moj::Version::Name(format!("ornithe-{name}")),
                None => moj::Version::Release,
            });

            installer.set_manifest_url("https://ornithemc.net/mc-versions/gen2/version_manifest.json");
            installer.set_manifest_prefix("ornithe-");

            installer.install(&mut Printer)?

        }
        _ => return Err(format!("unknown loader: {loader}").into()),
    };

    game.spawn_and_wait()?;
    Ok(())

}

/// A handler that prints a few relevant events, it implements the handler of every
/// installer, each one forwarding the events of the installer it extends.
struct Printer;

impl base::Handler for Printer {
    fn on_event(&mut self, event: base::Event) {
        println!("{event:?}");
    }
}

impl moj::Handler for Printer {
    fn on_event(&mut self, event: moj::Event) {
        println!("{event:?}");
    }
}

impl fabric::Handler for Printer {
    fn on_event(&mut self, event: fabric::Event) {
        println!("{event:?}");
    }
}

impl forge::Handler for Printer {
    fn on_event(&mut self, event: forge::Event) {
        println!("{event:?}");
    }
}
