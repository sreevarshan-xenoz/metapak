//! Security audit module for package vulnerability scanning.
//!
//! This module provides CVE scanning functionality using the OSV API
//! to check packages against known security vulnerabilities.

use crate::errors::Result;
use crate::models::Package;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub summary: String,
    pub severity: Severity,
    pub published: Option<String>,
    pub modified: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    None,
}

impl Default for Severity {
    fn default() -> Self {
        Severity::None
    }
}

impl From<&str> for Severity {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "CRITICAL" | "CRITICALITY_CRITICAL" => Severity::Critical,
            "HIGH" | "CRITICALITY_HIGH" => Severity::High,
            "MEDIUM" | "CRITICALITY_MEDIUM" => Severity::Medium,
            "LOW" | "CRITICALITY_LOW" => Severity::Low,
            _ => Severity::None,
        }
    }
}

#[derive(Debug, Serialize)]
struct OsvQuery {
    package: OsvPackage,
    version: String,
}

#[derive(Debug, Serialize)]
struct OsvPackage {
    name: String,
    ecosystem: String,
}

#[derive(Debug, Deserialize)]
struct OsvResponse {
    vulnerabilities: Option<Vec<OsvVulnerability>>,
}

#[derive(Debug, Deserialize)]
struct OsvVulnerability {
    id: String,
    summary: Option<String>,
    severity: Option<OsvSeverity>,
    published: Option<String>,
    modified: Option<String>,
    details: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OsvSeverity {
    score: Option<String>,
    #[serde(rename = "type")]
    severity_type: Option<String>,
}

pub struct SecurityAuditService {
    client: Client,
    cache: Arc<RwLock<HashMap<String, Vec<Vulnerability>>>>,
    ecosystem_map: HashMap<String, String>,
}

impl SecurityAuditService {
    pub fn new() -> Self {
        let ecosystem_map = [
            ("Pacman", "Arch Linux"),
            ("Aur", "AUR"),
            ("Apt", "Debian"),
            ("Dnf", "PyPI"),
            ("Zypper", "PyPI"),
            ("Brew", "PyPI"),
            ("Winget", "NuGet"),
            ("Chocolatey", "PyPI"),
            ("Flatpak", "PyPI"),
            ("Snap", "PyPI"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();

        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            ecosystem_map,
        }
    }

    pub async fn check_package(&self, package: &Package) -> Result<Vec<Vulnerability>> {
        let cache_key = format!("{}:{}", package.name, package.version);

        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                return Ok(cached.clone());
            }
        }

        let ecosystem = self
            .ecosystem_map
            .get(&package.source.to_string())
            .cloned()
            .unwrap_or_else(|| "PyPI".to_string());

        let query = OsvQuery {
            package: OsvPackage {
                name: package.name.clone(),
                ecosystem: ecosystem.clone(),
            },
            version: package.version.clone(),
        };

        let url = "https://api.osv.dev/v1/query";
        let response = self.client.post(url).json(&query).send().await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(osv_response) = resp.json::<OsvResponse>().await {
                    let vulnerabilities: Vec<Vulnerability> = osv_response
                        .vulnerabilities
                        .unwrap_or_default()
                        .iter()
                        .map(|v| Vulnerability {
                            id: v.id.clone(),
                            summary: v.summary.clone().unwrap_or_default(),
                            severity: v
                                .severity
                                .as_ref()
                                .and_then(|s| s.severity_type.as_deref())
                                .map(Severity::from)
                                .unwrap_or_default(),
                            published: v.published.clone(),
                            modified: v.modified.clone(),
                            details: v.details.clone(),
                        })
                        .collect();

                    {
                        let mut cache = self.cache.write().await;
                        cache.insert(cache_key, vulnerabilities.clone());
                    }

                    Ok(vulnerabilities)
                } else {
                    Ok(vec![])
                }
            }
            _ => Ok(vec![]),
        }
    }

    pub async fn check_packages(
        &self,
        packages: &[Package],
    ) -> HashMap<String, Vec<Vulnerability>> {
        let mut results = HashMap::new();

        for package in packages {
            if let Ok(vulns) = self.check_package(package).await {
                if !vulns.is_empty() {
                    results.insert(package.name.clone(), vulns);
                }
            }
        }

        results
    }

    pub async fn audit_installed(&self, packages: &[Package]) -> SecurityAuditReport {
        let vulnerable_packages = self.check_packages(packages).await;

        let critical = vulnerable_packages
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.severity == Severity::Critical)
            .count();
        let high = vulnerable_packages
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.severity == Severity::High)
            .count();
        let medium = vulnerable_packages
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.severity == Severity::Medium)
            .count();
        let low = vulnerable_packages
            .values()
            .flat_map(|v| v.iter())
            .filter(|v| v.severity == Severity::Low)
            .count();

        SecurityAuditReport {
            total_packages_checked: packages.len(),
            vulnerable_packages: vulnerable_packages.len(),
            critical_count: critical,
            high_count: high,
            medium_count: medium,
            low_count: low,
            vulnerabilities: vulnerable_packages,
        }
    }

    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}

impl Default for SecurityAuditService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SecurityAuditReport {
    pub total_packages_checked: usize,
    pub vulnerable_packages: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub vulnerabilities: HashMap<String, Vec<Vulnerability>>,
}

impl SecurityAuditReport {
    pub fn has_vulnerabilities(&self) -> bool {
        self.vulnerable_packages > 0
    }

    pub fn total_vulnerabilities(&self) -> usize {
        self.critical_count + self.high_count + self.medium_count + self.low_count
    }

    pub fn risk_level(&self) -> &'static str {
        if self.critical_count > 0 || self.high_count > 5 {
            "CRITICAL"
        } else if self.high_count > 0 || self.medium_count > 5 {
            "HIGH"
        } else if self.medium_count > 0 || self.low_count > 5 {
            "MEDIUM"
        } else if self.low_count > 0 {
            "LOW"
        } else {
            "NONE"
        }
    }

    pub fn format_summary(&self) -> String {
        if !self.has_vulnerabilities() {
            return format!(
                "No vulnerabilities found in {} packages",
                self.total_packages_checked
            );
        }

        format!(
            "Found {} vulnerable packages (C:{} H:{} M:{} L:{})",
            self.vulnerable_packages,
            self.critical_count,
            self.high_count,
            self.medium_count,
            self.low_count
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_from_str() {
        assert_eq!(Severity::from("CRITICAL"), Severity::Critical);
        assert_eq!(Severity::from("HIGH"), Severity::High);
        assert_eq!(Severity::from("MEDIUM"), Severity::Medium);
        assert_eq!(Severity::from("LOW"), Severity::Low);
        assert_eq!(Severity::from("UNKNOWN"), Severity::None);
    }

    #[test]
    fn test_audit_report_risk_level() {
        let report = SecurityAuditReport {
            total_packages_checked: 100,
            vulnerable_packages: 0,
            critical_count: 0,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            vulnerabilities: HashMap::new(),
        };
        assert_eq!(report.risk_level(), "NONE");

        let mut vulns = HashMap::new();
        vulns.insert(
            "test".to_string(),
            vec![Vulnerability {
                id: "CVE-2021-1234".to_string(),
                summary: "Test".to_string(),
                severity: Severity::Critical,
                published: None,
                modified: None,
                details: None,
            }],
        );

        let report = SecurityAuditReport {
            total_packages_checked: 100,
            vulnerable_packages: 1,
            critical_count: 1,
            high_count: 0,
            medium_count: 0,
            low_count: 0,
            vulnerabilities: vulns,
        };
        assert_eq!(report.risk_level(), "CRITICAL");
    }
}
