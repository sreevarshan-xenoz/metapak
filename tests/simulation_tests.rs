use metapak::simulation::SimulationEngine;

#[test]
fn test_parse_pacman_output_with_sizes() {
    let output = r#"
Packages (3) package1-1.0-1  package2-2.0-1  package3-3.0-1

Total Download Size:    10.50 MiB
Total Installed Size:   50.00 MiB
Net Upgrade Size:       40.00 MiB
"#;
    let result = SimulationEngine::parse_pacman_output(output);
    assert_eq!(result.total_download_bytes, (10.5 * 1024.0 * 1024.0) as u64);
    assert_eq!(result.disk_change_bytes, (40.0 * 1024.0 * 1024.0) as i64);
}

#[test]
fn test_parse_pacman_output_with_conflicts() {
    let output = r#"
error: unresolvable package conflicts detected
error: failed to prepare transaction (conflicting dependencies)
:: package1 and package2 are in conflict
"#;
    let result = SimulationEngine::parse_pacman_output(output);
    assert!(!result.conflicts.is_empty(), "Should detect conflicts");
    assert!(result
        .conflicts
        .iter()
        .any(|c| c.contains("package1 and package2 are in conflict")));
}

#[test]
fn test_parse_pacman_output_with_existing_files() {
    let output = r#"
error: failed to commit transaction (conflicting files)
package1: /usr/bin/file exists in filesystem
Errors occurred, no packages were upgraded.
"#;
    let result = SimulationEngine::parse_pacman_output(output);
    assert!(!result.conflicts.is_empty());
    assert!(result
        .conflicts
        .iter()
        .any(|c| c.contains("/usr/bin/file exists in filesystem")));
}
