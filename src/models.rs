#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: PackageSource,
    pub is_installed: bool,
    /// Size of the installed package in KB
    pub installed_size: Option<u64>,
    /// Size of the download in KB
    pub download_size: Option<u64>,
    /// Package groups
    pub groups: Vec<String>,
    /// Package licenses
    pub licenses: Vec<String>,
    /// Package maintainers (for AUR packages)
    pub maintainers: Vec<String>,
    /// Package keywords
    pub keywords: Vec<String>,
    /// URL of the project
    pub url: Option<String>,
    /// Dependencies of the package
    pub depends_on: Vec<String>,
    /// Packages that depend on this package
    pub required_by: Vec<String>,
    /// Optional dependencies
    pub opt_depends: Vec<String>,
    /// Conflicts with
    pub conflicts: Vec<String>,
    /// Replaces other packages
    pub replaces: Vec<String>,
    /// Provides virtual packages
    pub provides: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Pacman,
    Aur,
}

impl Package {
    pub fn format_installed_size(&self) -> String {
        match self.installed_size {
            Some(size) if size > 0 => Self::format_size_kb(size),
            _ => "-".to_string(),
        }
    }

    pub fn format_download_size(&self) -> String {
        match self.download_size {
            Some(size) if size > 0 => Self::format_size_kb(size),
            _ => "-".to_string(),
        }
    }

    fn format_size_kb(kb: u64) -> String {
        if kb >= 1024 * 1024 {
            format!("{:.1}M", kb as f64 / (1024.0 * 1024.0))
        } else if kb >= 1024 {
            format!("{:.1}K", kb as f64 / 1024.0)
        } else {
            format!("{}K", kb)
        }
    }
}
