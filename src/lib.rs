//! # blockdev
//!
//! A lightweight, type-safe library for parsing `lsblk --json` output on Linux.
//!
//! `blockdev` turns the JSON produced by `util-linux`'s `lsblk` into strongly
//! typed Rust structs ([`BlockDevices`], [`BlockDevice`], [`DeviceType`],
//! [`MajMin`]) and exposes ergonomic helpers for inspecting device hierarchies,
//! separating the root-filesystem disk from the rest, and locating devices by
//! name.
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
//! ## Parsing pre-captured JSON
//!
//! [`parse_lsblk`] accepts any string produced by `lsblk --json` (with or without
//! `--bytes`). Both numeric byte values and human-readable size strings such as
//! `"3.5T"` are accepted for the `size` field, and both single-value
//! `"mountpoint": null` and array-form `"mountpoints": [...]` representations
//! are supported transparently.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::process::Command;
use std::slice::Iter;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::vec::IntoIter;
use thiserror::Error;

/// Represents the major and minor device numbers (`maj:min` in `lsblk` output).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
#[derive(Debug, Error)]
#[error("invalid maj:min format '{0}': expected '<major>:<minor>'")]
pub struct ParseMajMinError(String);

impl FromStr for MajMin {
    type Err = ParseMajMinError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (major_str, minor_str) = s
            .split_once(':')
            .ok_or_else(|| ParseMajMinError(s.to_owned()))?;
        if minor_str.contains(':') {
            return Err(ParseMajMinError(s.to_owned()));
        }
        let major = major_str
            .parse()
            .map_err(|_| ParseMajMinError(s.to_owned()))?;
        let minor = minor_str
            .parse()
            .map_err(|_| ParseMajMinError(s.to_owned()))?;
        Ok(MajMin { major, minor })
    }
}

impl Serialize for MajMin {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format_args!("{}:{}", self.major, self.minor))
    }
}

impl<'de> Deserialize<'de> for MajMin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct V;
        impl serde::de::Visitor<'_> for V {
            type Value = MajMin;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a string of the form '<major>:<minor>'")
            }

            fn visit_str<E: DeError>(self, s: &str) -> Result<Self::Value, E> {
                MajMin::from_str(s).map_err(E::custom)
            }
        }
        deserializer.deserialize_str(V)
    }
}

impl std::fmt::Display for MajMin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.major, self.minor)
    }
}

/// Represents the type of a block device, as reported by `lsblk` in the
/// `"type"` field.
///
/// Unknown values are mapped to [`DeviceType::Other`] so that newer
/// `util-linux` versions cannot cause deserialization failures.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
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
    #[serde(other)]
    Other,
}

impl DeviceType {
    /// Returns the canonical lowercase string representation, matching the value
    /// `lsblk` emits in its `"type"` field.
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

impl std::fmt::Display for DeviceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error type for `blockdev` operations.
#[derive(Debug, Error)]
pub enum BlockDevError {
    /// The `lsblk` command failed to execute (e.g. it is not installed).
    #[error("failed to execute lsblk: {0}")]
    CommandFailed(#[from] std::io::Error),

    /// The `lsblk` command returned a non-zero exit status.
    #[error("lsblk returned error: {0}")]
    LsblkError(String),

    /// The output from `lsblk` was not valid UTF-8.
    #[error("invalid UTF-8 in lsblk output: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    /// Failed to parse the JSON output from `lsblk`.
    #[error("failed to parse lsblk JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
}

/// Represents the entire JSON object produced by `lsblk --json`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockDevices {
    /// The list of top-level block devices.
    pub blockdevices: Vec<BlockDevice>,
}

/// Parses a human-readable size string (e.g. `"500G"`, `"3.5T"`) into bytes.
///
/// Suffixes are matched case-insensitively. Both SI-style (`K`, `M`, `G`, ...)
/// and IEC-style (`KiB`, `MiB`, ...) are accepted; in both cases the multiplier
/// is the IEC binary value (1024^n), which matches how `lsblk` reports sizes
/// when not given `--bytes`.
///
/// Returns `None` if the input is empty, contains a value that cannot be parsed
/// as a non-negative number, has an unknown suffix, or overflows `u64`.
fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    let bytes = s.as_bytes();
    let split_at = bytes
        .iter()
        .position(|b| !b.is_ascii_digit() && *b != b'.')
        .unwrap_or(bytes.len());
    let (num_part, suffix) = s.split_at(split_at);
    let suffix = suffix.trim_start();

    let multiplier: u64 = match suffix.as_bytes() {
        b"" => 1,
        // Single-letter suffixes (case-insensitive). `| 0x20` lower-cases ASCII letters.
        [c] => match *c | 0x20 {
            b'b' => 1,
            b'k' => 1 << 10,
            b'm' => 1 << 20,
            b'g' => 1 << 30,
            b't' => 1 << 40,
            b'p' => 1 << 50,
            _ => return None,
        },
        // Multi-letter suffixes: `KB`, `KiB`, `MB`, `MiB`, ...
        s if s.eq_ignore_ascii_case(b"kb") || s.eq_ignore_ascii_case(b"kib") => 1 << 10,
        s if s.eq_ignore_ascii_case(b"mb") || s.eq_ignore_ascii_case(b"mib") => 1 << 20,
        s if s.eq_ignore_ascii_case(b"gb") || s.eq_ignore_ascii_case(b"gib") => 1 << 30,
        s if s.eq_ignore_ascii_case(b"tb") || s.eq_ignore_ascii_case(b"tib") => 1 << 40,
        s if s.eq_ignore_ascii_case(b"pb") || s.eq_ignore_ascii_case(b"pib") => 1 << 50,
        _ => return None,
    };

    // Integer fast path avoids float conversion when the value has no fractional part.
    if memchr_dot(num_part.as_bytes()).is_none() {
        let n: u64 = num_part.parse().ok()?;
        return n.checked_mul(multiplier);
    }

    let n: f64 = num_part.parse().ok()?;
    if n < 0.0 || !n.is_finite() {
        return None;
    }
    // Multiplier values are exact powers of two (≤ 2^50), so the cast to f64
    // is lossless. The product may exceed u64::MAX, in which case `as u64`
    // saturates to u64::MAX — guard against that.
    #[allow(clippy::cast_precision_loss)]
    let multiplier_f = multiplier as f64;
    let product = n * multiplier_f;
    #[allow(clippy::cast_precision_loss)]
    let u64_max_f = u64::MAX as f64;
    if product < 0.0 || product >= u64_max_f {
        return None;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let bytes = product as u64;
    Some(bytes)
}

/// Returns the position of the first `.` in `bytes`, if any. Avoids a generic
/// `str::contains` closure overhead in the hot path.
#[inline]
fn memchr_dot(bytes: &[u8]) -> Option<usize> {
    bytes.iter().position(|&b| b == b'.')
}

/// Custom deserializer that accepts either a numeric byte value (as emitted
/// when `lsblk --bytes` is used) or a human-readable size string (the default).
fn deserialize_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match &value {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| {
                n.as_f64().and_then(|f| {
                    #[allow(clippy::cast_precision_loss)]
                    let u64_max_f = u64::MAX as f64;
                    if f >= 0.0 && f.is_finite() && f < u64_max_f {
                        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                        let v = f as u64;
                        Some(v)
                    } else {
                        None
                    }
                })
            })
            .ok_or_else(|| DeError::custom("invalid numeric size")),
        Value::String(s) => {
            parse_size_string(s).ok_or_else(|| DeError::custom(format!("invalid size string: {s}")))
        }
        _ => Err(DeError::custom("size must be a number or string")),
    }
}

/// Custom deserializer that supports both a single mountpoint (which may be
/// `null`) and an array of mountpoints.
///
/// `lsblk` versions prior to 2.37 emitted a singular `"mountpoint"` field;
/// newer versions emit `"mountpoints"` as an array. This deserializer makes the
/// in-memory representation uniform: always a `Vec<Option<String>>`.
fn deserialize_mountpoints<'de, D>(deserializer: D) -> Result<Vec<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_array() {
        serde_json::from_value(value).map_err(DeError::custom)
    } else {
        let single: Option<String> = serde_json::from_value(value).map_err(DeError::custom)?;
        Ok(vec![single])
    }
}

/// Represents a single block device as reported by `lsblk`.
///
/// Children (partitions, RAID/LVM/crypt mappings layered on top) are stored in
/// the `children` field. To walk the device tree, use [`BlockDevice::descendants`]
/// or iterate manually via [`BlockDevice::children_iter`].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockDevice {
    /// The name of the block device (e.g. `"sda"`, `"nvme0n1"`).
    pub name: String,
    /// The major and minor numbers of the block device.
    ///
    /// Corresponds to the JSON field `"maj:min"`.
    #[serde(rename = "maj:min")]
    pub maj_min: MajMin,
    /// Whether the device is removable.
    pub rm: bool,
    /// The size of the block device in bytes.
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u64,
    /// Whether the device is read-only.
    pub ro: bool,
    /// The type of the block device.
    ///
    /// Corresponds to the JSON field `"type"` (a reserved keyword in Rust).
    #[serde(rename = "type")]
    pub device_type: DeviceType,
    /// The mountpoints of the device.
    ///
    /// Accepts both a single mountpoint (possibly `null`) and an array of
    /// mountpoints in the input JSON; always stored as a vector.
    #[serde(
        default,
        alias = "mountpoint",
        deserialize_with = "deserialize_mountpoints"
    )]
    pub mountpoints: Vec<Option<String>>,
    /// Optional nested children (partitions, RAID/LVM/crypt mappings).
    #[serde(default)]
    pub children: Option<Vec<BlockDevice>>,
}

impl BlockDevice {
    /// Returns `true` if this device has any children.
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.children.as_ref().is_some_and(|c| !c.is_empty())
    }

    /// Returns an iterator over the direct children of this device.
    ///
    /// Returns an empty iterator if the device has no children.
    pub fn children_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.children.iter().flat_map(|c| c.iter())
    }

    /// Returns an iterator over `self` and all descendants in pre-order.
    ///
    /// The traversal is iterative, so it will not stack-overflow on
    /// pathologically deep device trees.
    #[must_use]
    pub fn descendants(&self) -> Descendants<'_> {
        Descendants { stack: vec![self] }
    }

    /// Finds a direct child device by name.
    ///
    /// Returns `None` if no direct child with the given name exists. For a
    /// recursive search, use [`BlockDevice::find_descendant`].
    #[must_use]
    pub fn find_child(&self, name: &str) -> Option<&BlockDevice> {
        self.children.as_ref()?.iter().find(|c| c.name == name)
    }

    /// Recursively finds a descendant device (including `self`) by name.
    #[must_use]
    pub fn find_descendant(&self, name: &str) -> Option<&BlockDevice> {
        self.descendants().find(|d| d.name == name)
    }

    /// Returns all non-null mountpoints for this device as an allocated `Vec`.
    ///
    /// For zero-allocation iteration, prefer [`BlockDevice::active_mountpoints_iter`].
    #[must_use]
    pub fn active_mountpoints(&self) -> Vec<&str> {
        self.active_mountpoints_iter().collect()
    }

    /// Returns an iterator over the non-null mountpoints for this device.
    pub fn active_mountpoints_iter(&self) -> impl Iterator<Item = &str> {
        self.mountpoints.iter().filter_map(|m| m.as_deref())
    }

    /// Returns `true` if this device has at least one mountpoint.
    #[must_use]
    pub fn is_mounted(&self) -> bool {
        self.mountpoints.iter().any(Option::is_some)
    }

    /// Returns `true` if this device or any of its descendants is mounted at
    /// `/` (i.e. holds the root filesystem).
    ///
    /// Recursion depth is bounded by the depth of the device tree; for trees
    /// constructed via [`parse_lsblk`], that is in turn bounded by
    /// `serde_json`'s recursion limit.
    #[must_use]
    pub fn is_system(&self) -> bool {
        if self.mountpoints.iter().any(|m| m.as_deref() == Some("/")) {
            return true;
        }
        match &self.children {
            Some(children) => children.iter().any(BlockDevice::is_system),
            None => false,
        }
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
        if let Some(children) = &next.children {
            // Push in reverse so that the natural order is preserved when popping.
            for child in children.iter().rev() {
                self.stack.push(child);
            }
        }
        Some(next)
    }
}

impl BlockDevices {
    /// Returns the number of top-level block devices.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blockdevices.len()
    }

    /// Returns `true` if there are no block devices.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blockdevices.is_empty()
    }

    /// Returns an iterator over references to the top-level block devices.
    pub fn iter(&self) -> Iter<'_, BlockDevice> {
        self.blockdevices.iter()
    }

    /// Returns an iterator over every device in the tree, in pre-order.
    pub fn iter_all(&self) -> impl Iterator<Item = &BlockDevice> {
        self.blockdevices.iter().flat_map(|d| d.descendants())
    }

    /// Returns the top-level devices that contain the root filesystem (`/`),
    /// either directly or via a descendant.
    #[must_use]
    pub fn system(&self) -> Vec<&BlockDevice> {
        self.system_iter().collect()
    }

    /// Iterator-flavored version of [`BlockDevices::system`].
    pub fn system_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.blockdevices.iter().filter(|d| d.is_system())
    }

    /// Returns the top-level devices that do **not** contain the root
    /// filesystem.
    #[must_use]
    pub fn non_system(&self) -> Vec<&BlockDevice> {
        self.non_system_iter().collect()
    }

    /// Iterator-flavored version of [`BlockDevices::non_system`].
    pub fn non_system_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.blockdevices.iter().filter(|d| !d.is_system())
    }

    /// Finds a top-level block device by name.
    ///
    /// Returns `None` if no device with the given name exists. For a recursive
    /// search through children, use [`BlockDevices::find_anywhere`].
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&BlockDevice> {
        self.blockdevices.iter().find(|d| d.name == name)
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
        self.blockdevices.into_iter()
    }
}

impl<'a> IntoIterator for &'a BlockDevices {
    type Item = &'a BlockDevice;
    type IntoIter = Iter<'a, BlockDevice>;

    fn into_iter(self) -> Self::IntoIter {
        self.blockdevices.iter()
    }
}

/// Parses a JSON string (produced by `lsblk --json`) into a [`BlockDevices`]
/// struct.
///
/// Useful when you already have JSON data from `lsblk` and want to parse it
/// without running the command again.
///
/// # Errors
///
/// Returns a [`serde_json::Error`] if the JSON cannot be parsed or does not
/// conform to the expected schema.
///
/// # Examples
///
/// ```
/// use blockdev::parse_lsblk;
///
/// let json = r#"{"blockdevices": [{"name": "sda", "maj:min": "8:0", "rm": false, "size": "500G", "ro": false, "type": "disk", "mountpoints": [null]}]}"#;
/// let devices = parse_lsblk(json).expect("Failed to parse JSON");
/// assert_eq!(devices.len(), 1);
/// ```
pub fn parse_lsblk(json_data: &str) -> Result<BlockDevices, serde_json::Error> {
    serde_json::from_str(json_data)
}

/// Runs `lsblk --json --bytes`, captures its output, and parses it into a
/// [`BlockDevices`] struct.
///
/// # Errors
///
/// Returns [`BlockDevError`] if the `lsblk` command fails to start, exits with
/// a non-zero status, produces non-UTF-8 output, or its JSON cannot be parsed.
///
/// # Examples
///
/// ```no_run
/// # use blockdev::get_devices;
/// let devices = get_devices().expect("Failed to get block devices");
/// ```
pub fn get_devices() -> Result<BlockDevices, BlockDevError> {
    let output = Command::new("lsblk")
        .arg("--json")
        .arg("--bytes")
        .output()?;

    if !output.status.success() {
        return Err(BlockDevError::LsblkError(
            String::from_utf8_lossy(&output.stderr).into_owned(),
        ));
    }

    let json_output = String::from_utf8(output.stdout)?;
    parse_lsblk(&json_output).map_err(BlockDevError::from)
}

#[cfg(test)]
#[allow(
    clippy::similar_names,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = include_str!("../benches/fixtures/small.json");

    #[test]
    fn test_parse_lsblk() {
        let lsblk = parse_lsblk(SAMPLE_JSON).expect("Failed to parse JSON");
        assert_eq!(lsblk.blockdevices.len(), 10);

        for device in &lsblk.blockdevices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
        }

        let nvme3n1 = lsblk
            .blockdevices
            .iter()
            .find(|d| d.name == "nvme3n1")
            .expect("Expected to find device nvme3n1");

        assert!(
            nvme3n1
                .mountpoints
                .first()
                .and_then(|opt| opt.as_deref())
                .is_none(),
        );

        let children = nvme3n1.children.as_ref().expect("nvme3n1 should have children");
        assert_eq!(children.len(), 6);

        let nvme3n1p2 = children.iter().find(|c| c.name == "nvme3n1p2").unwrap();
        assert_eq!(
            nvme3n1p2.mountpoints.first().and_then(|opt| opt.as_deref()),
            Some("/boot/efi"),
        );

        let nvme3n1p3 = children.iter().find(|c| c.name == "nvme3n1p3").unwrap();
        let nested_children = nvme3n1p3.children.as_ref().unwrap();
        let md0 = nested_children.iter().find(|d| d.name == "md0").unwrap();
        assert_eq!(
            md0.mountpoints.first().and_then(|opt| opt.as_deref()),
            Some("/boot"),
        );

        let non_system = lsblk.non_system();
        assert_eq!(non_system.len(), 8);
        assert!(!non_system.iter().any(|d| d.name == "nvme3n1"));
    }

    #[test]
    fn test_majmin_from_str() {
        assert_eq!("8:0".parse::<MajMin>().unwrap(), MajMin::new(8, 0));
        assert_eq!("259:31".parse::<MajMin>().unwrap(), MajMin::new(259, 31));
        assert!("8".parse::<MajMin>().is_err());
        assert!("8:0:0".parse::<MajMin>().is_err());
        assert!("a:0".parse::<MajMin>().is_err());
        assert!("8:b".parse::<MajMin>().is_err());
        assert!("-1:0".parse::<MajMin>().is_err());
        // u32 overflow
        assert!("4294967296:0".parse::<MajMin>().is_err());
    }

    #[test]
    fn test_majmin_display() {
        assert_eq!(format!("{}", MajMin::new(259, 1)), "259:1");
    }

    #[test]
    fn test_device_type_display() {
        assert_eq!(DeviceType::Disk.to_string(), "disk");
        assert_eq!(DeviceType::Raid10.to_string(), "raid10");
        assert_eq!(DeviceType::Other.to_string(), "other");
    }

    #[test]
    fn test_parse_size_string() {
        assert_eq!(parse_size_string("0"), Some(0));
        assert_eq!(parse_size_string("0B"), Some(0));
        assert_eq!(parse_size_string("500"), Some(500));
        assert_eq!(parse_size_string("1K"), Some(1024));
        assert_eq!(parse_size_string("1KiB"), Some(1024));
        assert_eq!(parse_size_string("1KB"), Some(1024));
        assert_eq!(parse_size_string("1M"), Some(1024 * 1024));
        assert_eq!(parse_size_string("1G"), Some(1 << 30));
        assert_eq!(parse_size_string("3.5T"), Some((3.5_f64 * (1u64 << 40) as f64) as u64));
        // case-insensitive
        assert_eq!(parse_size_string("1g"), Some(1 << 30));
        assert_eq!(parse_size_string("1gib"), Some(1 << 30));
        // whitespace-tolerant
        assert_eq!(parse_size_string("  1 G  "), Some(1 << 30));
        // junk
        assert_eq!(parse_size_string(""), None);
        assert_eq!(parse_size_string("abc"), None);
        assert_eq!(parse_size_string("1X"), None);
        assert_eq!(parse_size_string("-1G"), None);
        // overflow guard
        assert_eq!(parse_size_string("999999999999P"), None);
    }

    #[test]
    fn test_iterative_is_system_deep() {
        // Bound matches what serde_json will accept by default, ensuring that
        // any input we successfully parse, we can also traverse without stack
        // overflow. (Auto-Drop is still recursive, so we keep depth modest.)
        const DEPTH: u32 = 200;
        let mut deepest = BlockDevice {
            name: "leaf".to_string(),
            maj_min: MajMin::new(9, 0),
            rm: false,
            size: 0,
            ro: false,
            device_type: DeviceType::Part,
            mountpoints: vec![Some("/".to_string())],
            children: None,
        };
        for i in 0..DEPTH {
            deepest = BlockDevice {
                name: format!("n{i}"),
                maj_min: MajMin::new(9, i),
                rm: false,
                size: 0,
                ro: false,
                device_type: DeviceType::Part,
                mountpoints: vec![None],
                children: Some(vec![deepest]),
            };
        }
        assert!(deepest.is_system());
    }

    #[test]
    fn test_descendants_preorder() {
        let device = BlockDevice {
            name: "root".into(),
            maj_min: MajMin::new(8, 0),
            rm: false,
            size: 0,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: Some(vec![
                BlockDevice {
                    name: "a".into(),
                    maj_min: MajMin::new(8, 1),
                    rm: false,
                    size: 0,
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: Some(vec![BlockDevice {
                        name: "a1".into(),
                        maj_min: MajMin::new(8, 2),
                        rm: false,
                        size: 0,
                        ro: false,
                        device_type: DeviceType::Part,
                        mountpoints: vec![None],
                        children: None,
                    }]),
                },
                BlockDevice {
                    name: "b".into(),
                    maj_min: MajMin::new(8, 3),
                    rm: false,
                    size: 0,
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: None,
                },
            ]),
        };
        let names: Vec<_> = device.descendants().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["root", "a", "a1", "b"]);
    }

    #[test]
    fn test_find_descendant_and_find_anywhere() {
        let lsblk = parse_lsblk(SAMPLE_JSON).unwrap();
        assert!(lsblk.find_by_name("md0").is_none());
        assert!(lsblk.find_anywhere("md0").is_some());
        let nvme3n1 = lsblk.find_by_name("nvme3n1").unwrap();
        assert!(nvme3n1.find_descendant("md2").is_some());
        assert_eq!(nvme3n1.find_descendant("nvme3n1").unwrap().name, "nvme3n1");
    }

    #[test]
    fn test_iter_all_count() {
        let lsblk = parse_lsblk(SAMPLE_JSON).unwrap();
        // 10 disks + their partitions + nested raid; just sanity-check > 10.
        assert!(lsblk.iter_all().count() > 10);
    }

    #[test]
    fn test_non_system() {
        let test_json = r#"
        {
            "blockdevices": [
                {"name":"sda","maj:min":"8:0","rm":false,"size":"447.1G","ro":false,"type":"disk","mountpoints":[null],
                 "children":[
                    {"name":"sda1","maj:min":"8:1","rm":false,"size":"512M","ro":false,"type":"part","mountpoints":[null]},
                    {"name":"sda2","maj:min":"8:2","rm":false,"size":"446.6G","ro":false,"type":"part","mountpoints":[null],
                     "children":[{"name":"md0","maj:min":"9:0","rm":false,"size":"446.6G","ro":false,"type":"raid1","mountpoints":["/"]}]}]},
                {"name":"nvme0n1","maj:min":"259:2","rm":false,"size":"1.7T","ro":false,"type":"disk","mountpoints":[null]},
                {"name":"nvme1n1","maj:min":"259:3","rm":false,"size":"1.7T","ro":false,"type":"disk","mountpoints":[null]}
            ]
        }"#;
        let disks = parse_lsblk(test_json).unwrap();
        let names: Vec<&str> = disks.non_system_iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["nvme0n1", "nvme1n1"]);
    }

    /// Warning: this test will attempt to run the `lsblk` command on your system.
    #[test]
    #[ignore = "requires lsblk command to be available on the system"]
    fn test_get_devices() {
        let dev = get_devices().expect("Failed to get block devices");
        assert!(!dev.blockdevices.is_empty());
    }

    #[test]
    fn test_into_iterator() {
        let device1 = BlockDevice {
            name: "sda".to_string(),
            maj_min: MajMin::new(8, 0),
            rm: false,
            size: 536_870_912_000,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: None,
        };
        let device2 = BlockDevice {
            name: "sdb".to_string(),
            maj_min: MajMin::new(8, 16),
            rm: false,
            size: 536_870_912_000,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: None,
        };
        let devices = BlockDevices {
            blockdevices: vec![device1, device2],
        };
        let names: Vec<String> = devices.into_iter().map(|dev| dev.name).collect();
        assert_eq!(names, vec!["sda".to_string(), "sdb".to_string()]);
    }

    #[test]
    fn test_empty_blockdevices() {
        let json = r#"{"blockdevices": []}"#;
        let devices = parse_lsblk(json).expect("Failed to parse empty JSON");
        assert!(devices.is_empty());
        assert_eq!(devices.len(), 0);
        assert!(devices.non_system().is_empty());
        assert!(devices.system().is_empty());
        assert!(devices.find_by_name("sda").is_none());
    }

    #[test]
    fn test_default_trait() {
        let devices = BlockDevices::default();
        assert!(devices.is_empty());
        assert_eq!(devices.len(), 0);
    }

    #[test]
    fn test_clone_trait() {
        let json = r#"{"blockdevices":[{"name":"sda","maj:min":"8:0","rm":false,"size":"500G","ro":false,"type":"disk","mountpoints":[null]}]}"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let cloned = devices.clone();
        assert_eq!(devices, cloned);
        assert_eq!(cloned.len(), 1);
    }

    #[test]
    fn test_serialization_roundtrip() {
        let json = r#"{"blockdevices":[{"name":"sda","maj:min":"8:0","rm":false,"size":"500G","ro":false,"type":"disk","mountpoints":[null],"children":null}]}"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let serialized = serde_json::to_string(&devices).expect("Failed to serialize");
        let deserialized: BlockDevices =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(devices, deserialized);
    }

    #[test]
    fn test_device_with_direct_root_mount() {
        let json = r#"{
            "blockdevices": [{
                "name":"sda","maj:min":"8:0","rm":false,"size":"500G","ro":false,
                "type":"disk","mountpoints":["/"]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let device = devices.find_by_name("sda").unwrap();
        assert!(device.is_system());
        assert!(device.is_mounted());
        assert_eq!(device.active_mountpoints(), vec!["/"]);
        assert_eq!(devices.system().len(), 1);
        assert!(devices.non_system().is_empty());
    }

    #[test]
    fn test_block_device_methods() {
        let device = BlockDevice {
            name: "sda".to_string(),
            maj_min: MajMin::new(8, 0),
            rm: false,
            size: 536_870_912_000,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![Some("/mnt/data".to_string()), None],
            children: Some(vec![BlockDevice {
                name: "sda1".to_string(),
                maj_min: MajMin::new(8, 1),
                rm: false,
                size: 268_435_456_000,
                ro: false,
                device_type: DeviceType::Part,
                mountpoints: vec![Some("/home".to_string())],
                children: None,
            }]),
        };
        assert!(device.is_disk());
        assert!(!device.is_partition());
        assert!(device.has_children());
        assert!(device.is_mounted());
        assert_eq!(device.active_mountpoints(), vec!["/mnt/data"]);
        let child = device.find_child("sda1").unwrap();
        assert!(!child.is_disk());
        assert!(child.is_partition());
        assert!(!child.has_children());
        assert!(device.find_child("nonexistent").is_none());
    }

    #[test]
    fn test_children_iter() {
        let device = BlockDevice {
            name: "sda".to_string(),
            maj_min: MajMin::new(8, 0),
            rm: false,
            size: 536_870_912_000,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: Some(vec![
                BlockDevice {
                    name: "sda1".to_string(),
                    maj_min: MajMin::new(8, 1),
                    rm: false,
                    size: 268_435_456_000,
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "sda2".to_string(),
                    maj_min: MajMin::new(8, 2),
                    rm: false,
                    size: 268_435_456_000,
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: None,
                },
            ]),
        };
        let names: Vec<&str> = device.children_iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["sda1", "sda2"]);

        let device_no_children = BlockDevice {
            name: "sdb".to_string(),
            maj_min: MajMin::new(8, 16),
            rm: false,
            size: 536_870_912_000,
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: None,
        };
        assert_eq!(device_no_children.children_iter().count(), 0);
    }

    #[test]
    fn test_borrowing_iterator() {
        let devices = BlockDevices {
            blockdevices: vec![
                BlockDevice {
                    name: "sda".to_string(),
                    maj_min: MajMin::new(8, 0),
                    rm: false,
                    size: 536_870_912_000,
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "sdb".to_string(),
                    maj_min: MajMin::new(8, 16),
                    rm: false,
                    size: 536_870_912_000,
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
            ],
        };
        let names: Vec<&str> = (&devices).into_iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["sda", "sdb"]);
        assert_eq!(devices.len(), 2);
        let names2: Vec<&str> = devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names2, vec!["sda", "sdb"]);
    }

    #[test]
    fn test_find_by_name() {
        let devices = BlockDevices {
            blockdevices: vec![
                BlockDevice {
                    name: "sda".to_string(),
                    maj_min: MajMin::new(8, 0),
                    rm: false,
                    size: 536_870_912_000,
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "nvme0n1".to_string(),
                    maj_min: MajMin::new(259, 0),
                    rm: false,
                    size: 1_099_511_627_776,
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
            ],
        };
        assert!(devices.find_by_name("sda").is_some());
        assert_eq!(devices.find_by_name("sda").unwrap().size, 536_870_912_000);
        assert!(devices.find_by_name("nvme0n1").is_some());
        assert!(devices.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_system_method() {
        let json = r#"{
            "blockdevices": [
                {"name":"sda","maj:min":"8:0","rm":false,"size":"500G","ro":false,"type":"disk","mountpoints":["/"]},
                {"name":"sdb","maj:min":"8:16","rm":false,"size":"500G","ro":false,"type":"disk","mountpoints":[null]},
                {"name":"sdc","maj:min":"8:32","rm":false,"size":"500G","ro":false,"type":"disk","mountpoints":["/home"]}
            ]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let system = devices.system();
        assert_eq!(system.len(), 1);
        assert_eq!(system[0].name, "sda");
    }

    #[test]
    fn test_multiple_mountpoints() {
        let json = r#"{
            "blockdevices": [{
                "name":"sda","maj:min":"8:0","rm":false,"size":"500G","ro":false,"type":"disk",
                "mountpoints":["/mnt/data","/mnt/backup",null]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let device = devices.find_by_name("sda").unwrap();
        assert!(device.is_mounted());
        assert_eq!(device.active_mountpoints(), vec!["/mnt/data", "/mnt/backup"]);
    }

    #[test]
    fn test_removable_and_readonly() {
        let json = r#"{
            "blockdevices": [{
                "name":"sr0","maj:min":"11:0","rm":true,"size":"4.7G","ro":true,
                "type":"rom","mountpoints":[null]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let device = devices.find_by_name("sr0").unwrap();
        assert!(device.rm);
        assert!(device.ro);
        assert_eq!(device.device_type, DeviceType::Rom);
    }

    #[test]
    fn test_unknown_device_type() {
        let json = r#"{
            "blockdevices": [{
                "name":"weird","maj:min":"8:0","rm":false,"size":0,"ro":false,
                "type":"newfangled","mountpoints":[null]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        assert_eq!(devices.find_by_name("weird").unwrap().device_type, DeviceType::Other);
    }

    #[test]
    fn test_size_as_number_and_string() {
        let json = r#"{
            "blockdevices": [
                {"name":"a","maj:min":"8:0","rm":false,"size":1073741824,"ro":false,"type":"disk","mountpoints":[null]},
                {"name":"b","maj:min":"8:1","rm":false,"size":"1G","ro":false,"type":"disk","mountpoints":[null]}
            ]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        assert_eq!(devices.find_by_name("a").unwrap().size, 1 << 30);
        assert_eq!(devices.find_by_name("b").unwrap().size, 1 << 30);
    }
}
