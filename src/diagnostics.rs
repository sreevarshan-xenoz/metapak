use std::process::Command;
use std::fs;

#[derive(Debug, Clone)]
pub struct DiagnosticItem {
    pub label: String,
    pub status: String,
}

pub fn run_diagnostics() -> Vec<DiagnosticItem> {
    let mut items = Vec::new();

    items.push(DiagnosticItem {
        label: "pacman binary".to_string(),
        status: if command_exists("pacman") {
            "OK".to_string()
        } else {
            "MISSING".to_string()
        },
    });

    let aur_helper = if command_exists("paru") {
        "paru"
    } else if command_exists("yay") {
        "yay"
    } else {
        "none"
    };
    items.push(DiagnosticItem {
        label: "AUR helper".to_string(),
        status: aur_helper.to_string(),
    });

    let lock_exists = std::path::Path::new("/var/lib/pacman/db.lck").exists();
    items.push(DiagnosticItem {
        label: "pacman db lock".to_string(),
        status: if lock_exists {
            "LOCKED".to_string()
        } else {
            "clear".to_string()
        },
    });

    items.push(DiagnosticItem {
        label: "disk space /".to_string(),
        status: disk_usage_root().unwrap_or_else(|| "unknown".to_string()),
    });

    items
}

pub fn get_system_info() -> Vec<DiagnosticItem> {
    let mut items = Vec::new();

    // OS Info
    items.push(DiagnosticItem {
        label: "OS".to_string(),
        status: get_os_info().unwrap_or_else(|| "unknown".to_string()),
    });

    // Kernel
    items.push(DiagnosticItem {
        label: "Kernel".to_string(),
        status: get_kernel_version().unwrap_or_else(|| "unknown".to_string()),
    });

    // Hostname
    items.push(DiagnosticItem {
        label: "Hostname".to_string(),
        status: get_hostname().unwrap_or_else(|| "unknown".to_string()),
    });

    // Uptime
    items.push(DiagnosticItem {
        label: "Uptime".to_string(),
        status: get_uptime().unwrap_or_else(|| "unknown".to_string()),
    });

    // CPU
    items.push(DiagnosticItem {
        label: "CPU".to_string(),
        status: get_cpu_info().unwrap_or_else(|| "unknown".to_string()),
    });

    // CPU Cores
    items.push(DiagnosticItem {
        label: "CPU Cores".to_string(),
        status: get_cpu_cores().to_string(),
    });

    // Memory
    items.push(DiagnosticItem {
        label: "Memory".to_string(),
        status: get_memory_info().unwrap_or_else(|| "unknown".to_string()),
    });

    // Total packages
    items.push(DiagnosticItem {
        label: "Installed packages".to_string(),
        status: get_total_packages().unwrap_or_else(|_| "unknown".to_string()),
    });

    // Screen resolution (if available)
    if let Some(res) = get_screen_resolution() {
        items.push(DiagnosticItem {
            label: "Screen".to_string(),
            status: res,
        });
    }

    // DE/WM
    items.push(DiagnosticItem {
        label: "Desktop".to_string(),
        status: get_desktop_environment().unwrap_or_else(|| "none".to_string()),
    });

    items
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn disk_usage_root() -> Option<String> {
    let output = Command::new("df").arg("-h").arg("/").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    if cols.len() < 5 {
        return None;
    }
    Some(format!("{} used", cols[4]))
}

fn get_os_info() -> Option<String> {
    let content = fs::read_to_string("/etc/os-release").ok()?;
    for line in content.lines() {
        if line.starts_with("PRETTY_NAME=") {
            return Some(line.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string());
        }
    }
    Some("Arch Linux".to_string())
}

fn get_kernel_version() -> Option<String> {
    let output = Command::new("uname").arg("-r").output().ok()?;
    if output.status.success() {
        return Some(String::from_utf8(output.stdout).ok()?.trim().to_string());
    }
    None
}

fn get_hostname() -> Option<String> {
    let output = Command::new("hostname").output().ok()?;
    if output.status.success() {
        return Some(String::from_utf8(output.stdout).ok()?.trim().to_string());
    }
    None
}

fn get_uptime() -> Option<String> {
    let output = Command::new("uptime").arg("-p").output().ok()?;
    if output.status.success() {
        return Some(String::from_utf8(output.stdout).ok()?.trim().to_string());
    }
    // Fallback to seconds
    let content = fs::read_to_string("/proc/uptime").ok()?;
    let parts: Vec<&str> = content.split_whitespace().collect();
    if let Some(secs) = parts.first() {
        let total_secs: u64 = secs.parse().ok()?;
        let days = total_secs / 86400;
        let hours = (total_secs % 86400) / 3600;
        let minutes = (total_secs % 3600) / 60;
        return Some(format!("{}d {}h {}m", days, hours, minutes));
    }
    None
}

fn get_cpu_info() -> Option<String> {
    let output = Command::new("cat").arg("/proc/cpuinfo").output().ok()?;
    let content = String::from_utf8(output.stdout).ok()?;
    for line in content.lines() {
        if line.starts_with("model name") {
            let info = line.split(':').nth(1)?.trim().to_string();
            // Truncate long CPU names
            if info.len() > 40 {
                return Some(format!("{}...", &info[..40]));
            }
            return Some(info);
        }
    }
    None
}

fn get_cpu_cores() -> usize {
    std::thread::available_parallelism().map(|p| p.get()).unwrap_or(1)
}

fn get_memory_info() -> Option<String> {
    let content = fs::read_to_string("/proc/meminfo").ok()?;
    let mut total = 0u64;
    let mut available = 0u64;

    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = parse_meminfo_value(line)?;
        } else if line.starts_with("MemAvailable:") {
            available = parse_meminfo_value(line)?;
        }
    }

    if total > 0 {
        let used = total - available;
        let used_gb = used as f64 / 1024.0 / 1024.0;
        let total_gb = total as f64 / 1024.0 / 1024.0;
        return Some(format!("{:.1}GB / {:.1}GB used", used_gb, total_gb));
    }
    None
}

fn parse_meminfo_value(line: &str) -> Option<u64> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    parts.get(1)?.parse().ok()
}

fn get_total_packages() -> Result<String, std::io::Error> {
    let output = Command::new("pacman").arg("-Qq").output()?;
    let count = String::from_utf8_lossy(&output.stdout)
        .lines()
        .count();
    Ok(count.to_string())
}

fn get_screen_resolution() -> Option<String> {
    let output = Command::new("xrandr").arg("--current").output().ok()?;
    let content = String::from_utf8(output.stdout).ok()?;
    for line in content.lines() {
        if line.contains("*") {
            let res = line.split_whitespace().next()?;
            return Some(res.to_string());
        }
    }
    None
}

fn get_desktop_environment() -> Option<String> {
    // Check various environment variables
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        if !de.is_empty() {
            return Some(de);
        }
    }
    if let Ok(de) = std::env::var("DESKTOP_SESSION") {
        if !de.is_empty() {
            return Some(de);
        }
    }
    None
}

#[derive(Debug, Clone)]
pub struct OrphanPackage {
    pub name: String,
    pub reason: String,
}

pub fn find_orphan_packages() -> Vec<OrphanPackage> {
    let mut orphans = Vec::new();

    // Get all explicitly installed packages
    let explicit_output = Command::new("pacman")
        .args(["-Qet", "--color", "never"])
        .output();

    if let Ok(output) = explicit_output {
        if output.status.success() {
            let packages = String::from_utf8_lossy(&output.stdout);
            for line in packages.lines() {
                let pkg_name = line.split_whitespace().next().unwrap_or("");
                if !pkg_name.is_empty() {
                    // Check if this package is a dependency of another package
                    if !is_required_by_other_package(pkg_name) {
                        orphans.push(OrphanPackage {
                            name: pkg_name.to_string(),
                            reason: "Not required by any installed package".to_string(),
                        });
                    }
                }
            }
        }
    }

    orphans
}

fn is_required_by_other_package(pkg_name: &str) -> bool {
    // Check if any package depends on this one
    let output = Command::new("pacman")
        .args(["-Q", pkg_name])
        .output();

    if let Ok(output) = output {
        if output.status.success() {
            // pacman -Q shows the package info including dependencies
            let info = String::from_utf8_lossy(&output.stdout);
            // If it says "optional dependencies" it might still be needed
            // But we keep it simple - if explicitly installed, check reverse deps
        }
    }

    // Check reverse dependencies using pacman -Sii
    let output = Command::new("pacman")
        .args(["-Sii", pkg_name])
        .output();

    if let Ok(output) = output {
        let info = String::from_utf8_lossy(&output.stdout);
        for line in info.lines() {
            if line.contains("Required By") {
                let deps = line.split(':').nth(1).unwrap_or("").trim();
                return !deps.is_empty() && deps != "None";
            }
        }
    }

    false
}

#[derive(Debug, Clone)]
pub struct PackageSize {
    pub name: String,
    pub size_kb: u64,
    pub size_formatted: String,
}

pub fn get_package_sizes() -> Vec<PackageSize> {
    let mut packages = Vec::new();

    let output = Command::new("pacman")
        .args(["-Qi", "--color", "never"])
        .output();

    if let Ok(output) = output {
        let content = String::from_utf8_lossy(&output.stdout);
        let mut current_pkg = String::new();

        for line in content.lines() {
            if line.starts_with("Name            :") {
                if let Some(name) = line.split(':').nth(1) {
                    current_pkg = name.trim().to_string();
                }
            } else if line.starts_with("Installed Size  :") {
                if let Some(size_str) = line.split(':').nth(1) {
                    let size_str = size_str.trim();
                    let size_val: f64 = size_str
                        .split_whitespace()
                        .next()
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0.0);
                    let unit = size_str.split_whitespace().nth(1).unwrap_or("");

                    let size_kb = match unit {
                        "KiB" => size_val as u64,
                        "MiB" => (size_val * 1024.0) as u64,
                        "GiB" => (size_val * 1024.0 * 1024.0) as u64,
                        _ => size_val as u64,
                    };

                    if !current_pkg.is_empty() {
                        packages.push(PackageSize {
                            name: current_pkg.clone(),
                            size_kb,
                            size_formatted: size_str.to_string(),
                        });
                    }
                }
            }
        }
    }

    packages.sort_by(|a, b| b.size_kb.cmp(&a.size_kb));
    packages
}
