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
    uuid: Option<String>,
    partuuid: Option<String>,
    fstype: Option<String>,
    label: Option<String>,
    partlabel: Option<String>,
    wwn: Option<String>,
    serial: Option<String>,
    model: Option<String>,
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
            uuid: self.uuid,
            partuuid: self.partuuid,
            fstype: self.fstype,
            label: self.label,
            partlabel: self.partlabel,
            wwn: self.wwn,
            serial: self.serial,
            model: self.model,
        }
    }
}

/// identifiers are compared only where lsblk has a value. lsblk gets them
/// from libudev/libblkid, we get them from the udev db file plus sysfs, so
/// where lsblk says null we may still know something -- virtio serial on a
/// box without udev, say. where lsblk knows, we must agree
fn blank_ids_where_lsblk_is_null(ours: &mut BlockDevice, theirs: &BlockDevice) {
    macro_rules! align {
        ($($f:ident),*) => { $( if theirs.$f.is_none() { ours.$f = None; } )* };
    }
    align!(uuid, partuuid, fstype, label, partlabel, wwn, serial, model);
    // lsblk 2.39 reads model from sysfs ("Virtual Disk"), 2.40+ from udev's
    // ID_MODEL ("Virtual_Disk"). we report the human form; compare modulo that
    if let (Some(o), Some(t)) = (&mut ours.model, &theirs.model) {
        if o.replace('_', " ") == t.replace('_', " ") {
            *o = t.clone();
        }
    }
    for (o, t) in ours.children.iter_mut().zip(&theirs.children) {
        blank_ids_where_lsblk_is_null(o, t);
    }
}

#[test]
#[ignore = "requires a real /sys and the lsblk command"]
fn walk_matches_lsblk() {
    let out = std::process::Command::new("lsblk")
        .args([
            "--json",
            "--bytes",
            "-o",
            "NAME,MAJ:MIN,RM,SIZE,RO,TYPE,MOUNTPOINTS,UUID,PARTUUID,FSTYPE,LABEL,PARTLABEL,WWN,SERIAL,MODEL",
        ])
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

    let mut via_walk = get_devices().expect("walk /sys");
    assert_eq!(via_walk.len(), via_lsblk.len(), "different root count");
    for (o, t) in via_walk.devices.iter_mut().zip(&via_lsblk.devices) {
        blank_ids_where_lsblk_is_null(o, t);
    }
    assert_eq!(via_walk, via_lsblk, "walk disagrees with lsblk");
}
