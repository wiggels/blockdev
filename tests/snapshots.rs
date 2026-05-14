//! Snapshot tests for `blockdev` over real-shaped `lsblk` fixtures.
//!
//! These cover each persona's headline use case:
//! - cloud: NVMe-only EC2 layout with `--bytes` numeric sizes
//! - backup: full hierarchy with RAID1, `[SWAP]`, and `/boot`
//! - embedded/legacy: legacy single-`"mountpoint"` (pre-2.37) schema
//! - k8s storage: LUKS + LVM stack

use blockdev::{BlockDevice, BlockDevices, parse_lsblk};

const RAID_ROOT: &str = include_str!("fixtures/raid_root.json");
const LEGACY_MOUNTPOINT: &str = include_str!("fixtures/legacy_mountpoint.json");
const LVM_CRYPT: &str = include_str!("fixtures/lvm_crypt.json");
const CLOUD_EC2: &str = include_str!("fixtures/cloud_ec2.json");

/// Compact summary of a parsed tree that is stable across cosmetic refactors of
/// the snapshot format (unlike `assert_debug_snapshot` of the full struct,
/// which churns on every field rename).
fn summary(devices: &BlockDevices) -> Vec<String> {
    let mut out = Vec::new();
    for top in devices {
        for dev in top.descendants() {
            let mounts: Vec<&str> = dev.active_mountpoints_iter().collect();
            out.push(format!(
                "{name} [{ty}] {majmin} size={size} rm={rm} ro={ro} mounts={mounts:?} system={sys}",
                name = dev.name,
                ty = dev.device_type,
                majmin = dev.maj_min,
                size = dev.size,
                rm = dev.rm,
                ro = dev.ro,
                sys = top_owner_is_system(top, dev),
            ));
        }
    }
    out
}

fn top_owner_is_system(top: &BlockDevice, dev: &BlockDevice) -> bool {
    // A device is "system-owned" if its top-level ancestor contains `/`.
    let _ = dev;
    top.is_system()
}

#[test]
fn snapshot_raid_root() {
    let parsed = parse_lsblk(RAID_ROOT).expect("parse");
    insta::assert_json_snapshot!("raid_root_summary", summary(&parsed));
    let system: Vec<&str> = parsed.system_iter().map(|d| d.name.as_str()).collect();
    let non_system: Vec<&str> = parsed.non_system_iter().map(|d| d.name.as_str()).collect();
    insta::assert_json_snapshot!("raid_root_partition", (system, non_system));
}

#[test]
fn snapshot_legacy_mountpoint() {
    let parsed = parse_lsblk(LEGACY_MOUNTPOINT).expect("parse");
    insta::assert_json_snapshot!("legacy_mountpoint_summary", summary(&parsed));
}

#[test]
fn snapshot_lvm_crypt() {
    let parsed = parse_lsblk(LVM_CRYPT).expect("parse");
    insta::assert_json_snapshot!("lvm_crypt_summary", summary(&parsed));
    // The full crypt+lvm stack should be marked system because vg0-root is at "/".
    let names: Vec<&str> = parsed
        .iter_all()
        .filter(|d| !d.active_mountpoints().is_empty())
        .map(|d| d.name.as_str())
        .collect();
    insta::assert_json_snapshot!("lvm_crypt_mounted_names", names);
}

#[test]
fn snapshot_cloud_ec2() {
    let parsed = parse_lsblk(CLOUD_EC2).expect("parse");
    insta::assert_json_snapshot!("cloud_ec2_summary", summary(&parsed));
    let candidates: Vec<&str> = parsed
        .non_system_iter()
        .filter(|d| d.is_disk())
        .map(|d| d.name.as_str())
        .collect();
    insta::assert_json_snapshot!("cloud_ec2_non_system_disks", candidates);
}

#[test]
fn snapshot_serialization_roundtrip_cloud() {
    let parsed = parse_lsblk(CLOUD_EC2).expect("parse");
    let json = serde_json::to_string_pretty(&parsed).expect("serialize");
    insta::assert_snapshot!("cloud_ec2_roundtrip_json", json);
    let reparsed: BlockDevices = serde_json::from_str(&json).expect("re-parse");
    assert_eq!(parsed, reparsed);
}
