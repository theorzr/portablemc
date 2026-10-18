//! Install and launch the latest stable version of a mod loader, while printing the
//! installation progress with a custom handler.
//!
//! ```sh
//! cargo run --example loader -- <fabric|quilt|legacyfabric|babric|forge|neoforge> [game version]
//! ```

use std::error::Error;
use std::io::Write;
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
        match event {
            base::Event::LoadedHierarchy { hierarchy } => {
                let names = hierarchy.iter().map(|v| v.name()).collect::<Vec<_>>();
                println!("Loaded versions: {}", names.join(" -> "));
            }
            base::Event::LoadedJvm { file, version, .. } => {
                println!("Loaded JVM {} at {}", version.unwrap_or("?"), file.display());
            }
            base::Event::DownloadProgress { count, total_count, size, total_size } => {
                print!("\rDownloading {count}/{total_count} ({:.1}/{:.1} MB)",
                    size as f32 / 1_000_000.0, total_size as f32 / 1_000_000.0);
                if count == total_count {
                    println!();
                }
                let _ = std::io::stdout().flush();
            }
            _ => {}
        }
    }
}

impl moj::Handler for Printer {
    fn on_event(&mut self, event: moj::Event) {
        match event {
            moj::Event::Base(event) => base::Handler::on_event(self, event),
            moj::Event::FetchVersion { version } => println!("Fetching version {version}"),
            _ => {}
        }
    }
}

impl fabric::Handler for Printer {
    fn on_event(&mut self, event: fabric::Event) {
        match event {
            fabric::Event::Mojang(event) => moj::Handler::on_event(self, event),
            fabric::Event::FetchVersion { game_version, loader_version } => {
                println!("Fetching loader {loader_version} for {game_version}");
            }
            _ => {}
        }
    }
}

impl forge::Handler for Printer {
    fn on_event(&mut self, event: forge::Event) {
        match event {
            forge::Event::Mojang(event) => moj::Handler::on_event(self, event),
            forge::Event::Installing { reason, .. } => println!("Installing loader ({reason:?})"),
            forge::Event::RunInstallerProcessor { name, .. } => println!("Running processor {name}"),
            forge::Event::Installed => println!("Loader installed"),
            _ => {}
        }
    }
}
