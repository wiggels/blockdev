//! snapshot tests over real shaped machine layouts, built as fake sysroots
//!
//! one per persona:
//! - cloud: nvme only ec2 box
//! - backup: full hierarchy w/ raid1, swap, /boot
//! - k8s storage: luks + lvm stack
//! - workstation: usb stick, dvd, empty loops

mod common;

use blockdev::{BlockDevice, BlockDevices, get_devices_at};
use common::FakeRoot;

/// compact summary that is stable across cosmetic struct changes
fn summary(devices: &BlockDevices) -> Vec<String> {
    let mut out = Vec::new();
    for top in devices {
        for dev in top.descendants() {
            out.push(format!(
                "{name} [{ty}] {majmin} size={size} rm={rm} ro={ro} mounts={mounts:?} system={sys}",
                name = dev.name,
                ty = dev.device_type,
                majmin = dev.maj_min,
                size = dev.size,
                rm = dev.rm,
                ro = dev.ro,
                mounts = dev.mountpoints,
                sys = top.is_system(),
            ));
        }
    }
    out
}

fn raid_root() -> FakeRoot {
    let r = FakeRoot::new("raid_root");
    r.device("sda", "8:0", 937_703_088)
        .part("sda", "sda1", "8:1", 1_048_576)
        .part("sda", "sda2", "8:2", 16_777_216)
        .part("sda", "sda3", "8:3", 919_877_296)
        .device("sdb", "8:16", 937_703_088)
        .part("sdb", "sdb1", "8:17", 1_048_576)
        .part("sdb", "sdb2", "8:18", 16_777_216)
        .part("sdb", "sdb3", "8:19", 919_877_296)
        .md("md0", "9:0", 1_046_528, "raid1")
        .holds("sda/sda1", "md0")
        .holds("sdb/sdb1", "md0")
        .md("md1", "9:1", 919_745_216, "raid1")
        .holds("sda/sda3", "md1")
        .holds("sdb/sdb3", "md1")
        .device("nvme0n1", "259:0", 3_750_748_848)
        .mounts(
            "25 1 9:1 / / rw,relatime - ext4 /dev/md1 rw\n\
             26 25 9:0 / /boot rw,relatime - ext4 /dev/md0 rw\n\
             27 25 259:0 / /data rw,relatime - xfs /dev/nvme0n1 rw\n",
            "/dev/sda2 partition 8388604 0 -2\n/dev/sdb2 partition 8388604 0 -3\n",
        );
    r
}

#[test]
fn snapshot_raid_root() {
    let r = raid_root();
    let parsed = get_devices_at(r.path()).unwrap();
    insta::assert_json_snapshot!("raid_root_summary", summary(&parsed));
    let system: Vec<&str> = parsed.system_iter().map(|d| d.name.as_str()).collect();
    let non_system: Vec<&str> = parsed.non_system_iter().map(|d| d.name.as_str()).collect();
    insta::assert_json_snapshot!("raid_root_partition", (system, non_system));
}

#[test]
fn snapshot_lvm_crypt() {
    let r = FakeRoot::new("lvm_crypt");
    r.device("nvme0n1", "259:0", 2_000_409_264)
        .part("nvme0n1", "nvme0n1p1", "259:1", 1_048_576)
        .part("nvme0n1", "nvme0n1p2", "259:2", 1_999_357_952)
        .dm(
            "dm-0",
            "253:0",
            1_999_325_184,
            "cryptroot",
            "CRYPT-LUKS2-0c1e-cryptroot",
        )
        .holds("nvme0n1/nvme0n1p2", "dm-0")
        .dm("dm-1", "253:1", 1_900_000_000, "vg0-root", "LVM-abc123")
        .dm("dm-2", "253:2", 99_000_000, "vg0-swap", "LVM-def456")
        .holds("dm-0", "dm-1")
        .holds("dm-0", "dm-2")
        .mounts(
            "30 1 253:1 / / rw - ext4 /dev/mapper/vg0-root rw\n\
             31 30 259:1 / /boot/efi rw - vfat /dev/nvme0n1p1 rw\n",
            "/dev/dm-2 partition 49500000 0 -2\n",
        );
    let parsed = get_devices_at(r.path()).unwrap();
    insta::assert_json_snapshot!("lvm_crypt_summary", summary(&parsed));
    let mounted: Vec<&str> = parsed
        .iter_all()
        .filter(|d| d.is_mounted())
        .map(|d| d.name.as_str())
        .collect();
    insta::assert_json_snapshot!("lvm_crypt_mounted_names", mounted);
}

#[test]
fn snapshot_cloud_ec2() {
    let r = FakeRoot::new("cloud_ec2");
    r.device("nvme0n1", "259:0", 16_777_216)
        .part("nvme0n1", "nvme0n1p1", "259:1", 16_744_415)
        .part("nvme0n1", "nvme0n1p14", "259:2", 8_192)
        .part("nvme0n1", "nvme0n1p15", "259:3", 217_088)
        .device("nvme1n1", "259:4", 209_715_200)
        .device("nvme2n1", "259:5", 209_715_200)
        .mounts(
            "20 1 259:1 / / rw - ext4 /dev/nvme0n1p1 rw\n\
             21 20 259:3 / /boot/efi rw - vfat /dev/nvme0n1p15 rw\n",
            "",
        );
    let parsed = get_devices_at(r.path()).unwrap();
    insta::assert_json_snapshot!("cloud_ec2_summary", summary(&parsed));
    let candidates: Vec<&str> = parsed
        .non_system_iter()
        .filter(|d| d.is_disk())
        .map(|d| d.name.as_str())
        .collect();
    insta::assert_json_snapshot!("cloud_ec2_non_system_disks", candidates);
}

#[test]
fn snapshot_workstation() {
    let r = FakeRoot::new("workstation");
    r.device("sda", "8:0", 1_953_525_168)
        .part("sda", "sda1", "8:1", 1_953_523_120)
        .device("sdb", "8:16", 60_555_264)
        .file("sys/class/block/sdb/removable", "1")
        .part("sdb", "sdb1", "8:17", 60_553_216)
        .device("sr0", "11:0", 8_388_608)
        .file("sys/class/block/sr0/device/type", "5\n")
        .file("sys/class/block/sr0/ro", "1")
        .device("loop0", "7:0", 0)
        .device("loop1", "7:1", 133_120)
        .file(
            "sys/class/block/loop1/loop/backing_file",
            "/var/lib/snapd/x.snap",
        )
        .device("ram0", "1:0", 16_384)
        .mounts(
            "20 1 8:1 / / rw - ext4 /dev/sda1 rw\n\
             21 20 8:17 / /media/hunter/USB\\040STICK rw - vfat /dev/sdb1 rw\n\
             22 20 7:1 / /snap/x/1 ro - squashfs /dev/loop1 ro\n",
            "",
        );
    let parsed = get_devices_at(r.path()).unwrap();
    insta::assert_json_snapshot!("workstation_summary", summary(&parsed));
}

#[test]
fn many_disks_walk_is_sorted_and_complete() {
    let r = FakeRoot::new("many");
    r.many_disks(64);
    let parsed = get_devices_at(r.path()).unwrap();
    assert_eq!(parsed.len(), 64);
    let ordered = parsed.iter().map(|d| d.maj_min).collect::<Vec<_>>();
    let mut sorted = ordered.clone();
    sorted.sort();
    assert_eq!(ordered, sorted);
    assert!(parsed.iter().all(BlockDevice::has_children));
    assert_eq!(parsed.system().len(), 1);
    assert_eq!(parsed.non_system().len(), 63);
}

#[cfg(feature = "serde")]
#[test]
fn snapshot_serde_roundtrip() {
    let r = raid_root();
    let parsed = get_devices_at(r.path()).unwrap();
    let json = serde_json::to_string_pretty(&parsed).unwrap();
    insta::assert_snapshot!("raid_root_json", json);
    let back: BlockDevices = serde_json::from_str(&json).unwrap();
    assert_eq!(back, parsed);
}
