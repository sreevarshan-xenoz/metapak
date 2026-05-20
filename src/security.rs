//! Security audit module for package vulnerability scanning.
//!
//! This module provides CVE scanning functionality using the OSV API
//! to check packages against known security vulnerabilities.

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Vulnerability {
    pub id: String,
    pub summary: String,
    pub severity: Severity,
    pub published: Option<String>,
    pub modified: Option<String>,
    pub details: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
#[allow(dead_code)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    #[default]
    None,
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
    fn test_vulnerability_struct() {
        let vuln = Vulnerability {
            id: "CVE-2021-1234".to_string(),
            summary: "Test".to_string(),
            severity: Severity::Critical,
            published: None,
            modified: None,
            details: None,
        };
        assert_eq!(vuln.id, "CVE-2021-1234");
        assert_eq!(vuln.severity, Severity::Critical);
    }
}
