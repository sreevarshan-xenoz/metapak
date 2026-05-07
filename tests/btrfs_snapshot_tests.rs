use arch_tui::backends::snapshots::btrfs::BtrfsProvider;
use arch_tui::traits::SnapshotProvider;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_btrfs_list_parsing() {
    let tmp_dir = tempdir().unwrap();
    let snapshots_dir = tmp_dir.path().join("snapshots");
    fs::create_dir_all(&snapshots_dir).unwrap();

    let provider = BtrfsProvider::new("/".to_string(), snapshots_dir.to_str().unwrap().to_string());

    // 1. Basic label
    let basic_id = "arch-tui-base-20260503-1430";
    fs::create_dir_all(snapshots_dir.join(basic_id)).unwrap();

    // 2. Complex label with dashes
    let complex_id = "arch-tui-my-long-label-20260503-1430";
    fs::create_dir_all(snapshots_dir.join(complex_id)).unwrap();

    // 3. Invalid format (too few parts)
    let invalid_id = "arch-tui-invalid";
    fs::create_dir_all(snapshots_dir.join(invalid_id)).unwrap();

    let snapshots = provider.list().await.unwrap();

    assert_eq!(snapshots.len(), 2, "Should have parsed 2 snapshots");

    // Check basic label
    let basic = snapshots
        .iter()
        .find(|s| s.id == basic_id)
        .expect("Basic snapshot not found");
    assert_eq!(basic.label, "base");

    // Check complex label - THIS IS EXPECTED TO FAIL CURRENTLY
    let complex = snapshots
        .iter()
        .find(|s| s.id == complex_id)
        .expect("Complex snapshot not found");
    assert_eq!(
        complex.label, "my-long-label",
        "Complex label with dashes should be correctly parsed"
    );
}
