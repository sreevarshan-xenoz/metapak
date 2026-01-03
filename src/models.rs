#[derive(Debug, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: String,
    pub source: PackageSource,
    pub is_installed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PackageSource {
    Pacman,
    Aur,
}
