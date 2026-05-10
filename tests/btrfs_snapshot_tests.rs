use metapak::backends::snapshots::btrfs::BtrfsProvider;
use metapak::traits::SnapshotProvider;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_btrfs_list_parsing() {
    let tmp_dir = tempdir().unwrap();
    let snapshots_dir = tmp_dir.path().join("snapshots");
    fs::create_dir_all(&snapshots_dir).unwrap();

    let provider = BtrfsProvider::new("/".to_string(), snapshots_dir.to_str().unwrap().to_string());

    // 1. Basic arch-tui label
    let arch_id = "arch-tui-base-20260503-1430";
    fs::create_dir_all(snapshots_dir.join(arch_id)).unwrap();

    // 2. Basic metapak label
    let metapak_id = "metapak-base-20260503-1430";
    fs::create_dir_all(snapshots_dir.join(metapak_id)).unwrap();

    // 3. Complex metapak label with dashes
    let complex_metapak_id = "metapak-my-long-label-20260503-1430";
    fs::create_dir_all(snapshots_dir.join(complex_metapak_id)).unwrap();

    // 4. Invalid format
    let invalid_id = "metapak-invalid";
    fs::create_dir_all(snapshots_dir.join(invalid_id)).unwrap();

    let snapshots = provider.list().await.unwrap();

    assert_eq!(snapshots.len(), 3, "Should have parsed 3 snapshots");

    // Check arch-tui label
    let arch = snapshots
        .iter()
        .find(|s| s.id == arch_id)
        .expect("arch-tui snapshot not found");
    assert_eq!(arch.label, "base");

    // Check metapak label
    let metapak = snapshots
        .iter()
        .find(|s| s.id == metapak_id)
        .expect("metapak snapshot not found");
    assert_eq!(metapak.label, "base");

    // Check complex metapak label
    let complex = snapshots
        .iter()
        .find(|s| s.id == complex_metapak_id)
        .expect("Complex metapak snapshot not found");
    assert_eq!(
        complex.label, "my-long-label",
        "Complex label with dashes should be correctly parsed"
    );
}
