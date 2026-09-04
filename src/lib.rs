//! # blockdev
//!
//! A small, dependency-free library for enumerating Linux block devices.
//!
//! `blockdev` walks `/sys` and `/proc` directly and builds a typed tree of
//! [`BlockDevice`]s: disks, their partitions, and whatever is stacked on top
//! (RAID, LVM, dm-crypt). It follows the same rules `lsblk` uses, so the
//! result matches `lsblk --json --bytes` on the same machine, but no process
//! is spawned and nothing is parsed. A full walk costs on the order of a
//! hundred microseconds.
//!
//! ## Quick start
//!
//! ```no_run
//! use blockdev::get_devices;
//!
//! let devices = get_devices()?;
//! for device in devices.non_system() {
//!     if device.is_disk() {
//!         println!("available disk: {} ({} bytes)", device.name, device.size);
//!     }
//! }
//! # Ok::<(), blockdev::BlockDevError>(())
//! ```
//!
//! ## Other roots
//!
//! [`get_devices_at`] takes the directory that holds `sys/` and `proc/`. Point
//! it at a bind-mounted host tree from inside a container, or at a fake tree
//! in tests. [`get_devices`] is `get_devices_at("/")`.
//!
//! ## Serde
//!
//! With the `serde` feature enabled every type derives `Serialize` and
//! `Deserialize`. [`MajMin`] is represented as the `"major:minor"` string.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::convert::Infallible;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;

mod sysfs;

/// Major and minor device numbers, as in `/sys/class/block/<name>/dev`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(try_from = "String", into = "String"))]
pub struct MajMin {
    /// The major device number.
    pub major: u32,
    /// The minor device number.
    pub minor: u32,
}

impl MajMin {
    /// Constructs a [`MajMin`] from its parts.
    #[must_use]
    pub const fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }
}

/// Error returned when [`MajMin::from_str`] is given a string that is not of the
/// form `"<major>:<minor>"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseMajMinError(String);

impl fmt::Display for ParseMajMinError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid maj:min format '{}': expected '<major>:<minor>'",
            self.0
        )
    }
}

impl std::error::Error for ParseMajMinError {}

/// ascii decimal to u32 -- no sign, no whitespace, no empty. std's parse takes
/// a leading `+` which sysfs never emits, and this skips the radix machinery
#[inline]
fn parse_u32_ascii(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    let mut n: u32 = 0;
    for &b in bytes {
        let d = b.wrapping_sub(b'0');
        if d > 9 {
            return None;
        }
        n = n.checked_mul(10)?.checked_add(u32::from(d))?;
    }
    Some(n)
}

impl FromStr for MajMin {
    type Err = ParseMajMinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // one split then digit-parse both halves -- a second ':' in the minor
        // half fails the digit check so no separate contains() scan
        let parsed = s.split_once(':').and_then(|(major, minor)| {
            Some(MajMin {
                major: parse_u32_ascii(major.as_bytes())?,
                minor: parse_u32_ascii(minor.as_bytes())?,
            })
        });
        parsed.ok_or_else(|| ParseMajMinError(s.to_owned()))
    }
}

impl TryFrom<String> for MajMin {
    type Error = ParseMajMinError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<MajMin> for String {
    fn from(m: MajMin) -> Self {
        m.to_string()
    }
}

impl fmt::Display for MajMin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

/// The kind of a block device, using the same names `lsblk` reports in its
/// `TYPE` column.
///
/// Anything not listed here -- multipath, `dm` with no owner, SCSI tapes and
/// changers -- maps to [`DeviceType::Other`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum DeviceType {
    /// A physical disk device.
    Disk,
    /// A partition on a disk.
    Part,
    /// A loop device.
    Loop,
    /// A RAID1 (mirroring) device.
    Raid1,
    /// A RAID5 device.
    Raid5,
    /// A RAID6 device.
    Raid6,
    /// A RAID0 (striping) device.
    Raid0,
    /// A RAID10 device.
    Raid10,
    /// An LVM logical volume.
    Lvm,
    /// A device mapper crypt device.
    Crypt,
    /// A ROM device (e.g., CD/DVD drive).
    Rom,
    /// An unknown or unsupported device type.
    #[cfg_attr(feature = "serde", serde(other))]
    Other,
}

impl DeviceType {
    /// Returns the canonical lowercase name, matching `lsblk`'s `TYPE` column.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            DeviceType::Disk => "disk",
            DeviceType::Part => "part",
            DeviceType::Loop => "loop",
            DeviceType::Raid0 => "raid0",
            DeviceType::Raid1 => "raid1",
            DeviceType::Raid5 => "raid5",
            DeviceType::Raid6 => "raid6",
            DeviceType::Raid10 => "raid10",
            DeviceType::Lvm => "lvm",
            DeviceType::Crypt => "crypt",
            DeviceType::Rom => "rom",
            DeviceType::Other => "other",
        }
    }
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DeviceType {
    type Err = Infallible;

    /// Never fails: exact lowercase `lsblk` names map to their variant and
    /// anything else is [`DeviceType::Other`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "disk" => DeviceType::Disk,
            "part" => DeviceType::Part,
            "loop" => DeviceType::Loop,
            "raid0" => DeviceType::Raid0,
            "raid1" => DeviceType::Raid1,
            "raid5" => DeviceType::Raid5,
            "raid6" => DeviceType::Raid6,
            "raid10" => DeviceType::Raid10,
            "lvm" => DeviceType::Lvm,
            "crypt" => DeviceType::Crypt,
            "rom" => DeviceType::Rom,
            _ => DeviceType::Other,
        })
    }
}

/// Error type for `blockdev` operations.
///
/// Only the two files the walk cannot do without -- the `/sys/block` listing
/// and `/proc/self/mountinfo` -- produce errors. Missing attributes on an
/// individual device are treated as defaults so a device being torn down
/// mid-walk does not fail the whole call.
#[derive(Debug)]
#[non_exhaustive]
pub enum BlockDevError {
    /// A required file or directory under the sysroot could not be read.
    Io {
        /// The path that failed.
        path: PathBuf,
        /// The underlying error.
        source: io::Error,
    },
}

impl fmt::Display for BlockDevError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockDevError::Io { path, source } => {
                write!(f, "failed to read {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for BlockDevError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BlockDevError::Io { source, .. } => Some(source),
        }
    }
}

/// The top-level block devices on a system.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockDevices {
    /// Top-level devices -- whole disks and anything else with nothing
    /// underneath it -- sorted by [`MajMin`].
    pub devices: Vec<BlockDevice>,
}

/// One block device and everything stacked on it.
///
/// Partitions and holders (RAID members, LVM, dm-crypt) are in `children`.
/// Walk the whole subtree with [`BlockDevice::descendants`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockDevice {
    /// The device name (e.g. `"sda"`, `"nvme0n1p1"`). Device mapper devices
    /// use their mapper name (`"vg0-root"`) rather than `dm-N`.
    pub name: String,
    /// The major and minor device numbers.
    pub maj_min: MajMin,
    /// Whether the device is removable. Partitions inherit this from their disk.
    pub rm: bool,
    /// The size in bytes.
    pub size: u64,
    /// Whether the device is read-only.
    pub ro: bool,
    /// The type of the block device.
    pub device_type: DeviceType,
    /// Every path this device is mounted at, newest mount first. Empty if not
    /// mounted. Active swap shows as `"[SWAP]"`, matching `lsblk`.
    pub mountpoints: Vec<String>,
    /// Partitions and holders, sorted by [`MajMin`]. Empty for leaves.
    pub children: Vec<BlockDevice>,
}

impl BlockDevice {
    /// Returns `true` if this device has any children.
    #[must_use]
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Returns an iterator over `self` and all descendants in pre-order.
    ///
    /// Iterative, so pathologically deep stacks cannot overflow the stack.
    #[must_use]
    pub fn descendants(&self) -> Descendants<'_> {
        Descendants { stack: vec![self] }
    }

    /// Finds a direct child by name. For a recursive search use
    /// [`BlockDevice::find_descendant`].
    #[must_use]
    pub fn find_child(&self, name: &str) -> Option<&BlockDevice> {
        self.children.iter().find(|c| c.name == name)
    }

    /// Recursively finds a descendant device (including `self`) by name.
    #[must_use]
    pub fn find_descendant(&self, name: &str) -> Option<&BlockDevice> {
        self.descendants().find(|d| d.name == name)
    }

    /// Returns `true` if this device has at least one mountpoint.
    #[must_use]
    pub fn is_mounted(&self) -> bool {
        !self.mountpoints.is_empty()
    }

    /// Returns `true` if this device or any descendant is mounted at `/`.
    #[must_use]
    pub fn is_system(&self) -> bool {
        self.descendants()
            .any(|d| d.mountpoints.iter().any(|m| m == "/"))
    }

    /// Returns `true` if this device is a [`DeviceType::Disk`].
    #[must_use]
    pub fn is_disk(&self) -> bool {
        self.device_type == DeviceType::Disk
    }

    /// Returns `true` if this device is a [`DeviceType::Part`].
    #[must_use]
    pub fn is_partition(&self) -> bool {
        self.device_type == DeviceType::Part
    }
}

/// Iterator returned by [`BlockDevice::descendants`].
///
/// Yields the originating device first, then every descendant in pre-order.
#[derive(Debug, Clone)]
pub struct Descendants<'a> {
    stack: Vec<&'a BlockDevice>,
}

impl<'a> Iterator for Descendants<'a> {
    type Item = &'a BlockDevice;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.stack.pop()?;
        // push reversed so natural order comes back out when popping
        self.stack.extend(next.children.iter().rev());
        Some(next)
    }
}

impl BlockDevices {
    /// Returns the number of top-level block devices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.devices.len()
    }

    /// Returns `true` if there are no block devices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.devices.is_empty()
    }

    /// Returns an iterator over the top-level block devices.
    pub fn iter(&self) -> Iter<'_, BlockDevice> {
        self.devices.iter()
    }

    /// Returns an iterator over every device in the tree, in pre-order.
    ///
    /// A device stacked on several parents -- RAID over two partitions -- is
    /// yielded once per parent, the same way `lsblk` prints it.
    pub fn iter_all(&self) -> impl Iterator<Item = &BlockDevice> {
        self.devices.iter().flat_map(|d| d.descendants())
    }

    /// Returns the top-level devices that contain the root filesystem (`/`),
    /// directly or via a descendant.
    #[must_use]
    pub fn system(&self) -> Vec<&BlockDevice> {
        self.system_iter().collect()
    }

    /// Iterator-flavored version of [`BlockDevices::system`].
    pub fn system_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.devices.iter().filter(|d| d.is_system())
    }

    /// Returns the top-level devices that do **not** contain the root
    /// filesystem.
    #[must_use]
    pub fn non_system(&self) -> Vec<&BlockDevice> {
        self.non_system_iter().collect()
    }

    /// Iterator-flavored version of [`BlockDevices::non_system`].
    pub fn non_system_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.devices.iter().filter(|d| !d.is_system())
    }

    /// Finds a top-level block device by name. For a recursive search use
    /// [`BlockDevices::find_anywhere`].
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&BlockDevice> {
        self.devices.iter().find(|d| d.name == name)
    }

    /// Recursively searches every device in the tree for one matching `name`.
    #[must_use]
    pub fn find_anywhere(&self, name: &str) -> Option<&BlockDevice> {
        self.iter_all().find(|d| d.name == name)
    }
}

impl IntoIterator for BlockDevices {
    type Item = BlockDevice;
    type IntoIter = IntoIter<BlockDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.into_iter()
    }
}

impl<'a> IntoIterator for &'a BlockDevices {
    type Item = &'a BlockDevice;
    type IntoIter = Iter<'a, BlockDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.devices.iter()
    }
}

/// Enumerates the block devices on this system.
///
/// Reads `/sys/block`, `/sys/class/block`, `/proc/self/mountinfo`, and
/// `/proc/swaps`. Equivalent to [`get_devices_at`] with `"/"`.
///
/// # Errors
///
/// Returns [`BlockDevError::Io`] if `/sys/block` or `/proc/self/mountinfo`
/// cannot be read -- on anything other than Linux, or in a container that
/// masks sysfs.
///
/// # Examples
///
/// ```no_run
/// let devices = blockdev::get_devices()?;
/// for device in &devices {
///     println!("{} {} bytes", device.name, device.size);
/// }
/// # Ok::<(), blockdev::BlockDevError>(())
/// ```
pub fn get_devices() -> Result<BlockDevices, BlockDevError> {
    get_devices_at(Path::new("/"))
}

/// Enumerates block devices from a tree rooted somewhere other than `/`.
///
/// `sysroot` must contain `sys/` and `proc/` in the usual layout. This is
/// `lsblk --sysroot`: use it from a container with the host's `/sys` and
/// `/proc` bind-mounted elsewhere, or with a hand-built tree in tests. Only
/// file names and contents are read, so a fake tree needs no symlinks.
///
/// # Errors
///
/// Returns [`BlockDevError::Io`] if `<sysroot>/sys/block` or
/// `<sysroot>/proc/self/mountinfo` cannot be read.
pub fn get_devices_at(sysroot: impl AsRef<Path>) -> Result<BlockDevices, BlockDevError> {
    sysfs::walk(sysroot.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(name: &str, maj_min: MajMin, ty: DeviceType) -> BlockDevice {
        BlockDevice {
            name: name.to_owned(),
            maj_min,
            rm: false,
            size: 0,
            ro: false,
            device_type: ty,
            mountpoints: Vec::new(),
            children: Vec::new(),
        }
    }

    fn mounted(mut d: BlockDevice, at: &str) -> BlockDevice {
        d.mountpoints.push(at.to_owned());
        d
    }

    fn with_children(mut d: BlockDevice, kids: Vec<BlockDevice>) -> BlockDevice {
        d.children = kids;
        d
    }

    #[test]
    fn majmin_from_str() {
        assert_eq!("8:0".parse::<MajMin>().unwrap(), MajMin::new(8, 0));
        assert_eq!("259:31".parse::<MajMin>().unwrap(), MajMin::new(259, 31));
        assert_eq!(
            "4294967295:0".parse::<MajMin>().unwrap(),
            MajMin::new(u32::MAX, 0)
        );
        for bad in [
            "8",
            "8:0:0",
            "a:0",
            "8:b",
            "-1:0",
            "4294967296:0",
            "+8:0",
            " 8:0",
            "8:",
            ":0",
        ] {
            assert!(bad.parse::<MajMin>().is_err(), "{bad:?} should fail");
        }
    }

    #[test]
    fn majmin_display_and_ord() {
        assert_eq!(MajMin::new(259, 1).to_string(), "259:1");
        assert!(MajMin::new(8, 16) < MajMin::new(9, 0));
        assert!(MajMin::new(8, 1) < MajMin::new(8, 16));
        let e = "junk".parse::<MajMin>().unwrap_err();
        assert!(e.to_string().contains("junk"));
    }

    #[test]
    fn device_type_roundtrip() {
        for t in [
            DeviceType::Disk,
            DeviceType::Part,
            DeviceType::Loop,
            DeviceType::Raid0,
            DeviceType::Raid1,
            DeviceType::Raid5,
            DeviceType::Raid6,
            DeviceType::Raid10,
            DeviceType::Lvm,
            DeviceType::Crypt,
            DeviceType::Rom,
        ] {
            assert_eq!(t.as_str().parse::<DeviceType>().unwrap(), t);
        }
        assert_eq!(
            "newfangled".parse::<DeviceType>().unwrap(),
            DeviceType::Other
        );
        assert_eq!(DeviceType::Raid10.to_string(), "raid10");
    }

    #[test]
    fn descendants_preorder_and_find() {
        let root = with_children(
            dev("root", MajMin::new(8, 0), DeviceType::Disk),
            vec![
                with_children(
                    dev("a", MajMin::new(8, 1), DeviceType::Part),
                    vec![dev("a1", MajMin::new(9, 0), DeviceType::Raid1)],
                ),
                dev("b", MajMin::new(8, 2), DeviceType::Part),
            ],
        );
        let names: Vec<_> = root.descendants().map(|d| d.name.as_str()).collect();
        assert_eq!(names, ["root", "a", "a1", "b"]);
        assert!(root.has_children());
        assert!(root.find_child("a").is_some());
        assert!(root.find_child("a1").is_none());
        assert!(root.find_descendant("a1").is_some());
        assert_eq!(root.find_descendant("root").unwrap().name, "root");
        assert!(root.find_child("a").unwrap().is_partition());
        assert!(!root.find_child("b").unwrap().has_children());
    }

    #[test]
    fn deep_stack_does_not_overflow() {
        let mut deepest = mounted(dev("leaf", MajMin::new(9, 0), DeviceType::Part), "/");
        for i in 0..5000 {
            deepest = with_children(
                dev(&format!("n{i}"), MajMin::new(9, i), DeviceType::Part),
                vec![deepest],
            );
        }
        assert!(deepest.is_system());
        assert_eq!(deepest.descendants().count(), 5001);
    }

    #[test]
    fn system_and_non_system_split() {
        let devices = BlockDevices {
            devices: vec![
                with_children(
                    dev("sda", MajMin::new(8, 0), DeviceType::Disk),
                    vec![with_children(
                        dev("sda2", MajMin::new(8, 2), DeviceType::Part),
                        vec![mounted(
                            dev("md0", MajMin::new(9, 0), DeviceType::Raid1),
                            "/",
                        )],
                    )],
                ),
                dev("nvme0n1", MajMin::new(259, 0), DeviceType::Disk),
                mounted(dev("sdc", MajMin::new(8, 32), DeviceType::Disk), "/home"),
            ],
        };
        let sys: Vec<_> = devices.system_iter().map(|d| d.name.as_str()).collect();
        let non: Vec<_> = devices.non_system_iter().map(|d| d.name.as_str()).collect();
        assert_eq!(sys, ["sda"]);
        assert_eq!(non, ["nvme0n1", "sdc"]);
        assert!(devices.find_by_name("md0").is_none());
        assert!(devices.find_anywhere("md0").is_some());
        assert_eq!(devices.iter_all().count(), 5);
        assert!(devices.find_by_name("sdc").unwrap().is_mounted());
        assert!(!devices.find_by_name("nvme0n1").unwrap().is_mounted());
    }

    #[test]
    fn empty_and_default() {
        let d = BlockDevices::default();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
        assert!(d.system().is_empty());
        assert!(d.non_system().is_empty());
        assert!(d.find_by_name("sda").is_none());
    }

    #[test]
    fn iteration_flavors() {
        let devices = BlockDevices {
            devices: vec![
                dev("sda", MajMin::new(8, 0), DeviceType::Disk),
                dev("sdb", MajMin::new(8, 16), DeviceType::Disk),
            ],
        };
        let by_ref: Vec<_> = (&devices).into_iter().map(|d| d.name.as_str()).collect();
        assert_eq!(by_ref, ["sda", "sdb"]);
        let by_iter: Vec<_> = devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(by_iter, ["sda", "sdb"]);
        let cloned = devices.clone();
        assert_eq!(cloned, devices);
        let owned: Vec<String> = devices.into_iter().map(|d| d.name).collect();
        assert_eq!(owned, ["sda", "sdb"]);
    }

    #[test]
    fn error_display_and_source() {
        let e = get_devices_at("/nonexistent/sysroot").unwrap_err();
        let msg = e.to_string();
        assert!(msg.contains("sys/block"), "{msg}");
        assert!(std::error::Error::source(&e).is_some());
    }

    /// needs a real /sys -- ci runs this on the linux job
    #[test]
    #[ignore = "requires a real /sys"]
    fn live_walk_has_a_root_device() {
        let devices = get_devices().expect("walk /sys");
        assert!(!devices.is_empty());
        assert_eq!(devices.system().len(), 1, "exactly one device holds /");
    }
}
