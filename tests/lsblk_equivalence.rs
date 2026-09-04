//! the walk must produce exactly what lsblk --json --bytes says on the same
//! machine. any drift is a bug in the walker, not noise. ignored by default
//! since it needs a real /sys and an lsblk binary -- ci runs it on linux
//!
//! `serde_json` is a dev dep only for this -- the crate itself parses nothing

use blockdev::{BlockDevice, BlockDevices, DeviceType, MajMin, get_devices};
use serde::Deserialize;

/// mirror of lsblk's json, just enough to compare
#[derive(Deserialize)]
struct LsblkOut {
    blockdevices: Vec<LsblkDev>,
}

#[derive(Deserialize)]
struct LsblkDev {
    name: String,
    #[serde(rename = "maj:min")]
    maj_min: String,
    rm: bool,
    size: u64,
    ro: bool,
    #[serde(rename = "type")]
    ty: String,
    #[serde(default)]
    mountpoints: Vec<Option<String>>,
    #[serde(default)]
    children: Vec<LsblkDev>,
}

impl LsblkDev {
    fn into_blockdev(self) -> BlockDevice {
        BlockDevice {
            name: self.name,
            maj_min: self.maj_min.parse::<MajMin>().unwrap(),
            rm: self.rm,
            size: self.size,
            ro: self.ro,
            device_type: self.ty.parse::<DeviceType>().unwrap(),
            mountpoints: self.mountpoints.into_iter().flatten().collect(),
            children: self
                .children
                .into_iter()
                .map(LsblkDev::into_blockdev)
                .collect(),
        }
    }
}

#[test]
#[ignore = "requires a real /sys and the lsblk command"]
fn walk_matches_lsblk() {
    let out = std::process::Command::new("lsblk")
        .args(["--json", "--bytes"])
        .output()
        .expect("run lsblk");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let parsed: LsblkOut = serde_json::from_slice(&out.stdout).expect("lsblk json");
    let via_lsblk = BlockDevices {
        devices: parsed
            .blockdevices
            .into_iter()
            .map(LsblkDev::into_blockdev)
            .collect(),
    };

    let via_walk = get_devices().expect("walk /sys");
    assert_eq!(via_walk, via_lsblk, "walk disagrees with lsblk");
}
