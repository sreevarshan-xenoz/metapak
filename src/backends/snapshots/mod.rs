//! Snapshot provider subsystem for system rollback safety.
//!
//! Supports btrfs and Timeshift snapshot providers, enabling
//! pre-operation snapshots and rollback on failure.

pub mod btrfs;
pub mod timeshift;
