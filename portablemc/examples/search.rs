//! List the latest Mojang releases, and the versions supported by Fabric and Forge.
//!
//! ```sh
//! cargo run --example search
//! ```

use std::error::Error;

use portablemc::base::VersionChannel;
use portablemc::{moj, fabric, forge};


fn main() -> Result<(), Box<dyn Error>> {

    let manifest = moj::Manifest::request(())?;
    println!("Latest release: {}", manifest.latest_release_name());
    println!("Latest snapshot: {}", manifest.latest_snapshot_name());

    println!("Last 5 releases:");
    for version in manifest.iter().filter(|v| v.channel() == VersionChannel::Release).take(5) {
        println!("  {} ({})", version.name(), version.release_time().date_naive());
    }

    let api = fabric::Api::new(fabric::Loader::Fabric);
    let game_versions = api.request_game_versions()?;
    let latest_game = game_versions.find_latest(true).ok_or("no fabric game version")?;
    let loader_versions = api.request_loader_versions(Some(latest_game.name()))?;
    let latest_loader = loader_versions.find_latest(true).ok_or("no fabric loader version")?;
    println!("Latest Fabric: {} for {}", latest_loader.name(), latest_game.name());

    let repo = forge::Repo::request(forge::Loader::Forge)?;
    match repo.find_latest(manifest.latest_release_name(), true) {
        Some(version) => println!("Latest Forge: {}", version.name()),
        None => println!("No stable Forge for {}", manifest.latest_release_name()),
    }

    Ok(())

}
