use crate::models::{Package, PackageSource};
use anyhow::Result;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct AurResponse {
    results: Vec<AurPackage>,
}

#[derive(Deserialize, Debug)]
struct AurPackage {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Version")]
    version: String,
    #[serde(rename = "Description")]
    description: Option<String>,
}

pub fn search(query: &str) -> Result<Vec<Package>> {
    let client = Client::new();
    let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", query);
    
    let resp = client.get(&url)
        .header("User-Agent", "arch-tui")
        .send()?
        .json::<AurResponse>()?;

    let packages = resp.results.into_iter().map(|aur_pkg| {
        Package {
            name: aur_pkg.name,
            version: aur_pkg.version,
            description: aur_pkg.description.unwrap_or_default(),
            source: PackageSource::Aur,
            is_installed: false, // We will update this later by checking against pacman -Qm or similar
        }
    }).collect();

    Ok(packages)
}
