use crate::models::{Package, PackageSource};
use crate::errors::{Result, AppError};
use reqwest::Client;
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
    #[serde(rename = "URL")]
    url: Option<String>,
    #[serde(rename = "Maintainer")]
    maintainer: Option<String>,
    #[serde(rename = "DependsOn")]
    depends_on: Option<Vec<String>>,
    #[serde(rename = "MakeDepends")]
    make_depends: Option<Vec<String>>,
    #[serde(rename = "OptDepends")]
    opt_depends: Option<Vec<String>>,
    #[serde(rename = "Conflicts")]
    conflicts: Option<Vec<String>>,
    #[serde(rename = "Licenses")]
    licenses: Option<Vec<String>>,
    #[serde(rename = "Keywords")]
    keywords: Option<Vec<String>>,
    #[serde(rename = "Provides")]
    provides: Option<Vec<String>>,
}

pub async fn search(query: &str) -> Result<Vec<Package>> {
    let client = Client::new();
    let url = format!("https://aur.archlinux.org/rpc/v5/search/{}", query);

    let resp = client.get(&url)
        .header("User-Agent", "arch-tui")
        .send()
        .await
        .map_err(|e| AppError::Aur(format!("Failed to send AUR request: {}", e)))?;

    let aur_response: AurResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Aur(format!("Failed to parse AUR response: {}", e)))?;

    let packages = aur_response.results.into_iter().map(|aur_pkg| {
        let mut all_deps = Vec::new();
        if let Some(depends) = aur_pkg.depends_on {
            all_deps.extend(depends);
        }
        if let Some(make_depends) = aur_pkg.make_depends {
            all_deps.extend(make_depends);
        }

        Package {
            name: aur_pkg.name,
            version: aur_pkg.version,
            description: aur_pkg.description.unwrap_or_default(),
            source: PackageSource::Aur,
            is_installed: false, // We will update this later by checking against pacman -Qm or similar
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: aur_pkg.licenses.unwrap_or_default(),
            maintainers: aur_pkg.maintainer.map(|m| vec![m]).unwrap_or_default(),
            keywords: aur_pkg.keywords.unwrap_or_default(),
            url: aur_pkg.url,
            depends_on: all_deps,
            required_by: vec![],
            opt_depends: aur_pkg.opt_depends.unwrap_or_default(),
            conflicts: aur_pkg.conflicts.unwrap_or_default(),
            replaces: vec![],
            provides: aur_pkg.provides.unwrap_or_default(),
        }
    }).collect();

    Ok(packages)
}
