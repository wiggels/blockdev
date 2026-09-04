//! the walk -- build a [`BlockDevices`] tree straight from `/sys` and `/proc`
//!
//! this is what lsblk does under the hood -- see `lsblk-cmd/lsblk.c` and
//! `lsblk-cmd/mnt.c` in util-linux. semantics mirror its defaults, i.e.
//! plain `lsblk --json --bytes`:
//!
//! * roots are `/sys/block` entries that are not ram disks -- major 1 -- and
//!   have no `slaves`, i.e. are not in the middle of a stack
//! * zero size loop devices with no backing file are hidden
//! * partitions are subdirs named `<disk>N` or `<disk>pN`
//! * a device's `holders` -- md, dm, whatever sits on top -- become children
//! * type follows lsblk's `get_type`: part, dm uuid prefix, loop, md level,
//!   scsi device type, else disk
//! * mountpoints come from `/proc/self/mountinfo` matched by maj:min or by
//!   source path, newest first, then `/proc/swaps`
//! * everything sorted by maj:min like lsblk has done since linux 4.8
//!
//! not covered -- and not needed for the current `BlockDevice` shape:
//! multi device fs groups -- btrfs raid, zfs pools -- where only one member
//! shows in mountinfo, and the `BLKROGET` ioctl fallback when `ro` is absent

use std::fs;
use std::path::{Path, PathBuf};

use crate::{BlockDevError, BlockDevice, BlockDevices, DeviceType, MajMin};

/// major numbers w/ special handling -- same constants lsblk uses
const RAMDISK_MAJOR: u32 = 1;
const LOOP_MAJOR: u32 = 7;

/// hard cap on holder recursion. real stacks are 3-4 deep -- disk > part >
/// crypt > lvm. sysfs cannot cycle in practice but a bad fixture could
const MAX_DEPTH: usize = 32;

/// entry point -- `root` holds `sys/` and `proc/`
pub(crate) fn walk(root: &Path) -> Result<BlockDevices, BlockDevError> {
    // sys/block first -- on a non linux box or a masked container that is
    // the error worth seeing, not a missing mountinfo
    let block = root.join("sys/block");
    let entries = fs::read_dir(&block).map_err(|source| BlockDevError::Io {
        path: block.clone(),
        source,
    })?;

    let walker = Walker {
        class_block: root.join("sys/class/block"),
        mounts: MountTable::load(root)?,
    };

    let mut devices = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|source| BlockDevError::Io {
            path: block.clone(),
            source,
        })?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if let Some(dev) = walker.root_device(name) {
            devices.push(dev);
        }
    }
    devices.sort_by_key(|d| d.maj_min);
    Ok(BlockDevices { devices })
}

struct Walker {
    class_block: PathBuf,
    mounts: MountTable,
}

impl Walker {
    /// top level entry -- applies the root only filters then builds the subtree
    fn root_device(&self, name: &str) -> Option<BlockDevice> {
        let dir = self.class_block.join(name);
        // no dev file -> not a block device dir, or it vanished mid walk
        let maj_min = read_majmin(&dir)?;
        if maj_min.major == RAMDISK_MAJOR {
            return None;
        }
        // in-middle devices show up under whatever they sit on, not as roots
        if dir_has_entries(&dir.join("slaves")) {
            return None;
        }
        let size = read_size(&dir);
        if size == 0 && maj_min.major == LOOP_MAJOR && !dir.join("loop/backing_file").exists() {
            return None;
        }
        Some(self.build(name, &dir, maj_min, size, Root, 0))
    }

    /// build one device plus its partitions and holders. `parent` is the whole
    /// disk when this is a partition -- rm falls back to it like lsblk
    fn build(
        &self,
        kname: &str,
        dir: &Path,
        maj_min: MajMin,
        size: u64,
        via: Via<'_>,
        depth: usize,
    ) -> BlockDevice {
        // /sys/block never lists partitions so roots skip the stat. a holder
        // can be a partition -- kpartx -- so those still check for `start`
        let is_part = match via {
            Root => false,
            Partition(_) => true,
            Holder => dir.join("start").exists(),
        };
        let parent = match via {
            Partition(p) => Some(p),
            Root | Holder => None,
        };
        let dm_name = if kname.starts_with("dm-") {
            read_trimmed(&dir.join("dm/name"))
        } else {
            None
        };

        // partitions inherit rm from the whole disk -- lsblk is_removable_device
        let mut rm = read_bool(&dir.join("removable"));
        if !rm {
            if let Some(p) = parent.filter(|p| p.is_wholedisk) {
                rm = p.rm;
            }
        }

        let device_type = device_type(kname, dir, is_part);
        let name = dm_name.unwrap_or_else(|| kname.replace('!', "/"));
        // dm devices can show up in mountinfo/swaps as /dev/dm-N or as
        // /dev/mapper/<name> -- lsblk canonicalizes, we just try both
        let mapper_path = dm_name_path(kname, dir);
        let mountpoints = self
            .mounts
            .lookup(maj_min, &name, kname, mapper_path.as_deref());

        let mut children = Vec::new();
        if depth < MAX_DEPTH {
            let me = Parent {
                is_wholedisk: !is_part,
                rm,
            };
            if !is_part {
                self.partitions(kname, dir, &me, depth, &mut children);
            }
            self.holders(dir, depth, &mut children);
        }
        children.sort_by_key(|d| d.maj_min);

        BlockDevice {
            name,
            maj_min,
            rm,
            size,
            ro: read_bool(&dir.join("ro")),
            device_type,
            mountpoints,
            children,
        }
    }

    /// partitions live as subdirs of the whole disk dir
    fn partitions(
        &self,
        disk: &str,
        dir: &Path,
        me: &Parent,
        depth: usize,
        out: &mut Vec<BlockDevice>,
    ) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let fname = entry.file_name();
            let Some(fname) = fname.to_str() else {
                continue;
            };
            if !is_partition_name(disk, fname) {
                continue;
            }
            let pdir = entry.path();
            let Some(maj_min) = read_majmin(&pdir) else {
                continue;
            };
            let size = read_size(&pdir);
            out.push(self.build(fname, &pdir, maj_min, size, Partition(me), depth + 1));
        }
    }

    /// `holders/` names whatever is stacked on this device -- resolve each via
    /// class/block since holders are always whole devices from sysfs' view
    fn holders(&self, dir: &Path, depth: usize, out: &mut Vec<BlockDevice>) {
        let Ok(entries) = fs::read_dir(dir.join("holders")) else {
            return;
        };
        for entry in entries.flatten() {
            let hname = entry.file_name();
            let Some(hname) = hname.to_str() else {
                continue;
            };
            let hdir = self.class_block.join(hname);
            let Some(maj_min) = read_majmin(&hdir) else {
                continue;
            };
            let size = read_size(&hdir);
            out.push(self.build(hname, &hdir, maj_min, size, Holder, depth + 1));
        }
    }
}

/// how we got to a device -- decides the partition check and rm fallback
#[derive(Clone, Copy)]
enum Via<'a> {
    /// listed in /sys/block
    Root,
    /// subdir of this whole disk
    Partition(&'a Parent),
    /// named in some device's holders/
    Holder,
}
use Via::{Holder, Partition, Root};

/// what a partition needs to know about its whole disk
struct Parent {
    is_wholedisk: bool,
    rm: bool,
}

/// lsblk `sysfs_blkdev_is_partition_dirent` -- `<disk>N` or `<disk>pN`
fn is_partition_name(disk: &str, name: &str) -> bool {
    let Some(rest) = name.strip_prefix(disk) else {
        return false;
    };
    let rest = rest.strip_prefix('p').unwrap_or(rest);
    !rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit())
}

/// lsblk `get_type`, minus the tolower since everything here is already lower
fn device_type(kname: &str, dir: &Path, is_part: bool) -> DeviceType {
    if is_part {
        return DeviceType::Part;
    }
    let s: String = if kname.starts_with("dm-") {
        // DM_UUID prefix names the owner -- LVM-, CRYPT-, mpath-, part1-...
        match read_trimmed(&dir.join("dm/uuid")) {
            Some(uuid) => {
                let prefix = uuid.split('-').next().unwrap_or("");
                // kpartx hack lsblk does too: partN-<rest> -> part
                if prefix.len() >= 4 && prefix[..4].eq_ignore_ascii_case("part") {
                    "part".to_owned()
                } else if prefix.is_empty() {
                    "dm".to_owned()
                } else {
                    prefix.to_ascii_lowercase()
                }
            }
            None => "dm".to_owned(),
        }
    } else if kname.starts_with("loop") {
        "loop".to_owned()
    } else if kname.starts_with("md") {
        read_trimmed(&dir.join("md/level"))
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| "md".to_owned())
    } else {
        // scsi device type -- only rom matters for our enum, rest fold to Other
        // via the unknown path and disk is the default like lsblk
        match read_trimmed(&dir.join("device/type")).and_then(|t| t.parse::<i32>().ok()) {
            Some(0) | None => "disk".to_owned(),
            Some(5) => "rom".to_owned(),
            Some(_) => "other".to_owned(),
        }
    };
    s.parse().unwrap_or(DeviceType::Other)
}

/// the /dev path lsblk would use as `filename` -- mountinfo source strings
/// and /proc/swaps are matched against it. dm devices go through mapper
fn dm_name_path(kname: &str, dir: &Path) -> Option<String> {
    if kname.starts_with("dm-") {
        read_trimmed(&dir.join("dm/name")).map(|n| format!("/dev/mapper/{n}"))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// sysfs attribute readers -- missing files are defaults, never errors, since
// a device can disappear between readdir and open
// ---------------------------------------------------------------------------

/// sysfs attributes are one short line. `fs::read_to_string` costs
/// open+statx+fstat+read+read+close because it sizes the buffer first --
/// a fixed stack buffer and one read is open+read+close, and the walk is
/// pure syscall overhead so that is a real cut
fn read_trimmed(p: &Path) -> Option<String> {
    use std::io::Read as _;
    let mut f = fs::File::open(p).ok()?;
    let mut buf = [0u8; 256];
    let n = f.read(&mut buf).ok()?;
    // dm/name and mountpoints could in theory exceed 256 -- fall back to
    // the slow path rather than truncate silently
    if n == buf.len() {
        return fs::read_to_string(p).ok().map(|s| s.trim().to_owned());
    }
    Some(String::from_utf8_lossy(&buf[..n]).trim().to_owned())
}

fn read_majmin(dir: &Path) -> Option<MajMin> {
    read_trimmed(&dir.join("dev"))?.parse().ok()
}

/// `size` is in 512 byte sectors regardless of the device's logical block size
fn read_size(dir: &Path) -> u64 {
    read_trimmed(&dir.join("size"))
        .and_then(|s| s.parse::<u64>().ok())
        .map_or(0, |s| s << 9)
}

fn read_bool(p: &Path) -> bool {
    read_trimmed(p).is_some_and(|s| s == "1")
}

fn dir_has_entries(p: &Path) -> bool {
    fs::read_dir(p).is_ok_and(|mut d| d.next().is_some())
}

// ---------------------------------------------------------------------------
// mount table -- one parse of mountinfo + swaps, then lookups per device
// ---------------------------------------------------------------------------

struct MountEntry {
    devno: MajMin,
    target: String,
    source: String,
}

struct MountTable {
    /// file order -- lookups walk it backwards so the newest mount is first,
    /// which is what lsblk reports as the primary mountpoint
    mounts: Vec<MountEntry>,
    /// active swap device paths from /proc/swaps
    swaps: Vec<String>,
}

impl MountTable {
    fn load(root: &Path) -> Result<Self, BlockDevError> {
        let path = root.join("proc/self/mountinfo");
        let mountinfo =
            fs::read_to_string(&path).map_err(|source| BlockDevError::Io { path, source })?;
        let mounts = mountinfo.lines().filter_map(parse_mountinfo_line).collect();
        // swaps is optional -- no swap configured, or no procfs swap support
        let swaps = fs::read_to_string(root.join("proc/swaps"))
            .map(|s| {
                s.lines()
                    .skip(1)
                    .filter_map(|l| l.split_whitespace().next().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        Ok(Self { mounts, swaps })
    }

    /// lsblk `lsblk_device_get_filesystems` then the MOUNTPOINTS column:
    /// every mountinfo entry matching by devno or source path, newest first.
    /// nothing mounted -> empty
    fn lookup(
        &self,
        devno: MajMin,
        name: &str,
        kname: &str,
        mapper_path: Option<&str>,
    ) -> Vec<String> {
        let dev_path = format!("/dev/{name}");
        let kdev_path = format!("/dev/{kname}");
        let matches_src = |s: &str| s == dev_path || s == kdev_path || mapper_path == Some(s);

        let mut out: Vec<String> = self
            .mounts
            .iter()
            .rev()
            .filter(|m| m.devno == devno || matches_src(&m.source))
            .map(|m| m.target.clone())
            .collect();

        if out.is_empty() && self.swaps.iter().any(|s| matches_src(s)) {
            out.push("[SWAP]".to_owned());
        }
        out
    }
}

/// mountinfo line: `id parent maj:min root target opts [optional...] - fstype source superopts`
fn parse_mountinfo_line(line: &str) -> Option<MountEntry> {
    let mut it = line.split(' ');
    let _id = it.next()?;
    let _parent = it.next()?;
    let devno: MajMin = it.next()?.parse().ok()?;
    let _root = it.next()?;
    let target = unescape_octal(it.next()?);
    // optional fields run until the "-" separator
    let mut rest = it.skip_while(|f| *f != "-");
    let _sep = rest.next()?;
    let _fstype = rest.next()?;
    let source = unescape_octal(rest.next()?);
    Some(MountEntry {
        devno,
        target,
        source,
    })
}

/// mountinfo escapes space/tab/newline/backslash as `\ooo`
fn unescape_octal(s: &str) -> String {
    if !s.contains('\\') {
        return s.to_owned();
    }
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\'
            && i + 3 < b.len()
            && b[i + 1..i + 4].iter().all(|c| (b'0'..=b'7').contains(c))
        {
            let v = (b[i + 1] - b'0') * 64 + (b[i + 2] - b'0') * 8 + (b[i + 3] - b'0');
            out.push(v);
            i += 4;
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    //! fake sysroot in a tempdir -- covers the stacked layouts a ci box does
    //! not have. only file names and contents matter to the walker so no
    //! symlinks needed, /sys/block entries can be empty files

    use std::fs;
    use std::path::{Path, PathBuf};

    use super::*;

    struct FakeRoot(PathBuf);

    impl FakeRoot {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!(
                "blockdev-sysfs-{tag}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            let _ = fs::remove_dir_all(&p);
            fs::create_dir_all(&p).unwrap();
            Self(p)
        }

        fn file(&self, rel: &str, content: &str) -> &Self {
            let p = self.0.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, content).unwrap();
            self
        }

        fn dir(&self, rel: &str) -> &Self {
            fs::create_dir_all(self.0.join(rel)).unwrap();
            self
        }

        /// a whole device: sys/block entry + class/block dir w/ the basics
        fn device(&self, name: &str, majmin: &str, sectors: u64) -> &Self {
            self.file(&format!("sys/block/{name}"), "")
                .file(&format!("sys/class/block/{name}/dev"), majmin)
                .file(
                    &format!("sys/class/block/{name}/size"),
                    &sectors.to_string(),
                )
                .file(&format!("sys/class/block/{name}/ro"), "0")
                .file(&format!("sys/class/block/{name}/removable"), "0")
                .dir(&format!("sys/class/block/{name}/holders"))
                .dir(&format!("sys/class/block/{name}/slaves"))
        }

        /// partition under a whole device
        fn part(&self, disk: &str, name: &str, majmin: &str, sectors: u64) -> &Self {
            let base = format!("sys/class/block/{disk}/{name}");
            self.file(&format!("{base}/dev"), majmin)
                .file(&format!("{base}/size"), &sectors.to_string())
                .file(&format!("{base}/ro"), "0")
                .file(&format!("{base}/start"), "2048")
                .dir(&format!("{base}/holders"))
        }

        fn mounts(&self, mountinfo: &str, swaps: &str) -> &Self {
            self.file("proc/self/mountinfo", mountinfo).file(
                "proc/swaps",
                &format!("Filename\t\tType\t\tSize\tUsed\tPriority\n{swaps}"),
            )
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for FakeRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn names(devs: &[BlockDevice]) -> Vec<&str> {
        devs.iter().map(|d| d.name.as_str()).collect()
    }

    #[test]
    fn raid1_over_partitions_with_swap_and_boot() {
        // sda1 + sdb1 -> md0 (/), sda2 swap. the classic backup box layout.
        // md0 must appear under both member partitions like lsblk prints it
        let r = FakeRoot::new("raid");
        r.device("sda", "8:0", 1_000_000)
            .part("sda", "sda1", "8:1", 500_000)
            .part("sda", "sda2", "8:2", 400_000)
            .file("sys/class/block/sda/sda1/holders/md0", "")
            .device("sdb", "8:16", 1_000_000)
            .part("sdb", "sdb1", "8:17", 500_000)
            .file("sys/class/block/sdb/sdb1/holders/md0", "")
            .device("md0", "9:0", 499_000)
            .file("sys/class/block/md0/md/level", "raid1\n")
            .file("sys/class/block/md0/slaves/sda1", "")
            .file("sys/class/block/md0/slaves/sdb1", "")
            .mounts(
                "22 1 9:0 / / rw,relatime - ext4 /dev/md0 rw\n\
                 23 22 0:21 / /proc rw - proc proc rw\n",
                "/dev/sda2 partition 400000 0 -2\n",
            );
        let devs = walk(r.path()).unwrap();

        // md0 is in-middle (has slaves) so it is not a root
        assert_eq!(names(&devs.devices), ["sda", "sdb"]);

        let sda = &devs.devices[0];
        assert_eq!(sda.maj_min, MajMin::new(8, 0));
        assert_eq!(sda.size, 1_000_000 * 512);
        assert_eq!(sda.device_type, DeviceType::Disk);
        assert!(sda.mountpoints.is_empty());

        let kids = &sda.children;
        assert_eq!(names(kids), ["sda1", "sda2"]);
        assert_eq!(kids[0].device_type, DeviceType::Part);
        assert_eq!(kids[1].mountpoints, ["[SWAP]"]);

        let md0 = kids[0].find_child("md0").unwrap();
        assert_eq!(md0.device_type, DeviceType::Raid1);
        assert_eq!(md0.maj_min, MajMin::new(9, 0));
        assert_eq!(md0.mountpoints, ["/"]);
        assert!(md0.children.is_empty());

        // same md0 under sdb1 too
        assert!(devs.devices[1].find_descendant("md0").is_some());
        assert!(sda.is_system());
        assert!(devs.devices[1].is_system());
    }

    #[test]
    fn luks_then_lvm_stack_uses_dm_names_and_uuid_prefix() {
        // nvme0n1p2 -> dm-0 (CRYPT-) -> dm-1 (LVM-) mounted at /
        let r = FakeRoot::new("dm");
        r.device("nvme0n1", "259:0", 2_000_000)
            .part("nvme0n1", "nvme0n1p1", "259:1", 1_000)
            .part("nvme0n1", "nvme0n1p2", "259:2", 1_990_000)
            .file("sys/class/block/nvme0n1/nvme0n1p2/holders/dm-0", "")
            .device("dm-0", "253:0", 1_980_000)
            .file("sys/class/block/dm-0/dm/name", "cryptroot\n")
            .file(
                "sys/class/block/dm-0/dm/uuid",
                "CRYPT-LUKS2-abc-cryptroot\n",
            )
            .file("sys/class/block/dm-0/slaves/nvme0n1p2", "")
            .file("sys/class/block/dm-0/holders/dm-1", "")
            .device("dm-1", "253:1", 1_970_000)
            .file("sys/class/block/dm-1/dm/name", "vg0-root\n")
            .file("sys/class/block/dm-1/dm/uuid", "LVM-xyz\n")
            .file("sys/class/block/dm-1/slaves/dm-0", "")
            .mounts("30 1 253:1 / / rw - ext4 /dev/mapper/vg0-root rw\n", "");
        let devs = walk(r.path()).unwrap();
        assert_eq!(names(&devs.devices), ["nvme0n1"]);

        let p2 = devs.devices[0].find_child("nvme0n1p2").unwrap();
        let crypt = p2.find_child("cryptroot").expect("dm name not dm-0");
        assert_eq!(crypt.device_type, DeviceType::Crypt);
        assert_eq!(crypt.maj_min, MajMin::new(253, 0));
        let lv = crypt.find_child("vg0-root").unwrap();
        assert_eq!(lv.device_type, DeviceType::Lvm);
        assert_eq!(lv.mountpoints, ["/"]);
        assert!(devs.devices[0].is_system());
    }

    #[test]
    fn hides_ram_disks_and_empty_loops_but_keeps_backed_loop() {
        let r = FakeRoot::new("loop");
        r.device("ram0", "1:0", 8192)
            .device("loop0", "7:0", 0)
            .device("loop1", "7:1", 4096)
            .device("loop2", "7:2", 0)
            .file("sys/class/block/loop2/loop/backing_file", "/x.img")
            .device("zram0", "253:0", 0)
            .mounts("", "");
        let devs = walk(r.path()).unwrap();
        // sorted by maj:min: loop1 (7:1), loop2 (7:2), zram0 (253:0)
        assert_eq!(names(&devs.devices), ["loop1", "loop2", "zram0"]);
        assert_eq!(devs.devices[0].device_type, DeviceType::Loop);
        assert_eq!(devs.devices[2].device_type, DeviceType::Disk);
        assert_eq!(devs.devices[2].size, 0);
    }

    #[test]
    fn partition_inherits_removable_and_rom_type_from_scsi() {
        let r = FakeRoot::new("rm");
        r.device("sdc", "8:32", 100)
            .file("sys/class/block/sdc/removable", "1")
            .part("sdc", "sdc1", "8:33", 90)
            .device("sr0", "11:0", 100)
            .file("sys/class/block/sr0/device/type", "5\n")
            .file("sys/class/block/sr0/ro", "1")
            .mounts("", "");
        let devs = walk(r.path()).unwrap();
        let sdc = devs.find_by_name("sdc").unwrap();
        assert!(sdc.rm);
        assert!(
            sdc.find_child("sdc1").unwrap().rm,
            "partition should inherit rm"
        );
        let sr0 = devs.find_by_name("sr0").unwrap();
        assert_eq!(sr0.device_type, DeviceType::Rom);
        assert!(sr0.ro);
        assert!(!sr0.rm);
    }

    #[test]
    fn mountpoints_newest_first_with_octal_escapes_and_bind_by_source() {
        let r = FakeRoot::new("mnt");
        r.device("vda", "254:0", 100)
            .device("cciss!c0d0", "104:0", 100)
            .mounts(
                "20 1 254:0 / / rw - ext4 /dev/vda rw\n\
                 21 20 254:0 / /mnt/my\\040data rw - ext4 /dev/vda rw\n\
                 22 20 0:50 / /by/source rw - ext4 /dev/cciss/c0d0 rw\n",
                "",
            );
        let devs = walk(r.path()).unwrap();
        let vda = devs.find_by_name("vda").unwrap();
        assert_eq!(vda.mountpoints, ["/mnt/my data", "/"]);
        // ! in sysfs names is / in /dev, and source path matching still works
        let cciss = devs.find_by_name("cciss/c0d0").unwrap();
        assert_eq!(cciss.mountpoints, ["/by/source"]);
    }

    #[test]
    fn missing_sys_block_is_an_error() {
        let r = FakeRoot::new("empty");
        r.mounts("", "");
        let e = walk(r.path()).unwrap_err();
        assert!(e.to_string().contains("sys/block"), "{e}");
    }

    #[test]
    fn missing_mountinfo_is_an_error() {
        let r = FakeRoot::new("nomnt");
        r.device("vda", "254:0", 100);
        let e = walk(r.path()).unwrap_err();
        assert!(e.to_string().contains("mountinfo"), "{e}");
    }

    #[test]
    fn missing_swaps_is_fine() {
        let r = FakeRoot::new("noswap");
        r.device("vda", "254:0", 100)
            .file("proc/self/mountinfo", "");
        assert_eq!(walk(r.path()).unwrap().len(), 1);
    }

    #[test]
    fn partition_name_rule() {
        assert!(is_partition_name("sda", "sda1"));
        assert!(is_partition_name("sda", "sda12"));
        assert!(is_partition_name("nvme0n1", "nvme0n1p3"));
        assert!(is_partition_name("mmcblk0", "mmcblk0p1"));
        assert!(!is_partition_name("sda", "sda"));
        assert!(!is_partition_name("sda", "sdb1"));
        assert!(!is_partition_name("sda", "sdap"));
        assert!(!is_partition_name("sda", "sda1x"));
        assert!(!is_partition_name("sda", "holders"));
    }

    #[test]
    fn octal_unescape() {
        assert_eq!(unescape_octal("/a\\040b"), "/a b");
        assert_eq!(unescape_octal("/tab\\011x"), "/tab\tx");
        assert_eq!(unescape_octal("plain"), "plain");
        assert_eq!(unescape_octal("trail\\04"), "trail\\04");
        assert_eq!(unescape_octal("bs\\134"), "bs\\");
    }
}
