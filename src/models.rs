#[derive(Debug, Clone, Default)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: PackageSource,
    pub is_installed: bool,
    pub is_outdated: bool,
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
    /// AUR: Number of votes
    pub votes: Option<i32>,
    /// AUR: Popularity score
    pub popularity: Option<f32>,
    /// AUR: First submitted timestamp
    pub first_submitted: Option<i64>,
    /// AUR: Last updated timestamp
    pub last_updated: Option<i64>,
    /// AUR: Package base ID
    pub package_base_id: Option<String>,
    /// AUR: Num votes (alias for votes, different API naming)
    pub num_votes: Option<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Pacman,
    Aur,
}

impl Default for PackageSource {
    fn default() -> Self {
        Self::Pacman
    }
}

impl Package {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            source: PackageSource::default(),
            is_installed: false,
            is_outdated: false,
            installed_size: None,
            download_size: None,
            groups: vec![],
            licenses: vec![],
            maintainers: vec![],
            keywords: vec![],
            url: None,
            depends_on: vec![],
            required_by: vec![],
            opt_depends: vec![],
            conflicts: vec![],
            replaces: vec![],
            provides: vec![],
            votes: None,
            popularity: None,
            first_submitted: None,
            last_updated: None,
            package_base_id: None,
            num_votes: None,
        }
    }

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

    pub fn format_votes(&self) -> String {
        let votes = self.num_votes.unwrap_or_else(|| self.votes.unwrap_or(-1));
        if votes > 0 {
            votes.to_string()
        } else {
            "-".to_string()
        }
    }

    pub fn format_popularity(&self) -> String {
        match self.popularity {
            Some(p) => format!("{:.1}", p),
            _ => "-".to_string(),
        }
    }

    pub fn format_last_updated(&self) -> String {
        match self.last_updated {
            Some(ts) => {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_secs() as i64)
                    .unwrap_or(0);
                let diff = now - ts;
                if diff < 86400 {
                    format!("{}h ago", diff / 3600)
                } else if diff < 2592000 {
                    format!("{}d ago", diff / 86400)
                } else if diff < 31536000 {
                    format!("{}mo ago", diff / 2592000)
                } else {
                    format!("{}y ago", diff / 31536000)
                }
            }
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

    pub fn format_size(size: u64) -> String {
        Self::format_size_kb(size)
    }

    pub fn get_size(&self) -> u64 {
        self.download_size.unwrap_or(self.installed_size.unwrap_or(0))
    }
}

#[derive(Debug, Clone, Default)]
pub struct OutdatedPackage {
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub download_size: u64,
    pub repository: String,
    pub is_security_update: bool,
    pub cve_info: Option<String>,
    pub new_dependencies: Vec<String>,
    pub removed_dependencies: Vec<String>,
    pub new_opt_depends: Vec<String>,
    pub removed_opt_depends: Vec<String>,
    pub conflicts: Vec<String>,
    pub replaces: Vec<String>,
    pub is_aur: bool,
    pub needs_rebuild: bool,
    pub changelog: Option<String>,
    pub is_selected: bool,
}

impl OutdatedPackage {
    pub fn new(name: String, current_version: String, new_version: String) -> Self {
        Self {
            name,
            current_version,
            new_version,
            download_size: 0,
            repository: String::new(),
            is_security_update: false,
            cve_info: None,
            new_dependencies: Vec::new(),
            removed_dependencies: Vec::new(),
            new_opt_depends: Vec::new(),
            removed_opt_depends: Vec::new(),
            conflicts: Vec::new(),
            replaces: Vec::new(),
            is_aur: false,
            needs_rebuild: false,
            changelog: None,
            is_selected: false,
        }
    }

    pub fn formatted_size(&self) -> String {
        Package::format_size_kb(self.download_size)
    }

    pub fn version_change(&self) -> String {
        format!("{} → {}", self.current_version, self.new_version)
    }
}
