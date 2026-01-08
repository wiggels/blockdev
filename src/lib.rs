use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::process::Command;
use std::slice::Iter;
use std::string::FromUtf8Error;
use std::vec::IntoIter;
use thiserror::Error;

/// Represents the type of a block device.
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

/// Error type for blockdev operations.
#[derive(Debug, Error)]
pub enum BlockDevError {
    /// The lsblk command failed to execute.
    #[error("failed to execute lsblk: {0}")]
    CommandFailed(#[from] std::io::Error),

    /// The lsblk command returned a non-zero exit status.
    #[error("lsblk returned error: {0}")]
    LsblkError(String),

    /// The output from lsblk was not valid UTF-8.
    #[error("invalid UTF-8 in lsblk output: {0}")]
    InvalidUtf8(#[from] FromUtf8Error),

    /// Failed to parse the JSON output from lsblk.
    #[error("failed to parse lsblk JSON: {0}")]
    JsonParse(#[from] serde_json::Error),
}

/// Represents the entire JSON output produced by `lsblk --json`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct BlockDevices {
    /// A vector of block devices.
    pub blockdevices: Vec<BlockDevice>,
}

/// Parses a human-readable size string (e.g., "500G", "3.5T") into bytes.
fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    // Find where the numeric part ends and the suffix begins
    let (num_part, suffix) = {
        let idx = s
            .find(|c: char| !c.is_ascii_digit() && c != '.')
            .unwrap_or(s.len());
        (&s[..idx], s[idx..].trim())
    };

    let num: f64 = num_part.parse().ok()?;
    let multiplier: u64 = match suffix.to_uppercase().as_str() {
        "" | "B" => 1,
        "K" | "KB" | "KIB" => 1024,
        "M" | "MB" | "MIB" => 1024 * 1024,
        "G" | "GB" | "GIB" => 1024 * 1024 * 1024,
        "T" | "TB" | "TIB" => 1024 * 1024 * 1024 * 1024,
        "P" | "PB" | "PIB" => 1024 * 1024 * 1024 * 1024 * 1024,
        _ => return None,
    };

    Some((num * multiplier as f64) as u64)
}

/// Custom deserializer that handles both numeric byte values and human-readable size strings.
fn deserialize_size<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    match &value {
        Value::Number(n) => n
            .as_u64()
            .or_else(|| n.as_f64().map(|f| f as u64))
            .ok_or_else(|| DeError::custom("invalid numeric size")),
        Value::String(s) => {
            parse_size_string(s).ok_or_else(|| DeError::custom(format!("invalid size string: {s}")))
        }
        _ => Err(DeError::custom("size must be a number or string")),
    }
}

/// Custom deserializer that supports both a single mountpoint (which may be null)
/// and an array of mountpoints.
///
/// # Arguments
///
/// * `deserializer` - The deserializer instance.
///
/// # Returns
///
/// A vector of optional strings representing mountpoints.
///
/// # Errors
///
/// Returns an error if the value cannot be deserialized either as a single value or as an array.
///
/// This function is used internally by Serde when deserializing block devices.
/// For example, if the JSON value is `null`, it will be converted to `vec![None]`.
fn deserialize_mountpoints<'de, D>(deserializer: D) -> Result<Vec<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    if value.is_array() {
        // Deserialize as an array of optional strings.
        serde_json::from_value(value).map_err(DeError::custom)
    } else {
        // Otherwise, deserialize as a single Option<String> and wrap it in a vector.
        let single: Option<String> = serde_json::from_value(value).map_err(DeError::custom)?;
        Ok(vec![single])
    }
}

/// Represents a block device as output by `lsblk`.
///
/// Note that the `children` field is optional, as some devices might not have any nested children.
///
/// # Field Details
///
/// - `name`: The device name.
/// - `maj_min`: The device's major and minor numbers. (Renamed from the JSON field "maj:min")
/// - `rm`: Whether the device is removable.
/// - `size`: The device size.
/// - `ro`: Whether the device is read-only.
/// - `device_type`: The device type (renamed from the reserved keyword "type").
/// - `mountpoints`: A vector of mountpoints for the device. Uses a custom deserializer to support both single and multiple mountpoints.
/// - `children`: Optional nested block devices.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockDevice {
    /// The name of the block device.
    pub name: String,
    /// The major and minor numbers of the block device.
    ///
    /// This field corresponds to the JSON field `"maj:min"`.
    #[serde(rename = "maj:min")]
    pub maj_min: String,
    /// Indicates if the device is removable.
    pub rm: bool,
    /// The size of the block device in bytes.
    #[serde(deserialize_with = "deserialize_size")]
    pub size: u64,
    /// Indicates if the device is read-only.
    pub ro: bool,
    /// The type of the block device.
    ///
    /// The JSON field is `"type"`, which is a reserved keyword in Rust. It is renamed to `device_type`.
    #[serde(rename = "type")]
    pub device_type: DeviceType,
    /// The mountpoints of the device.
    ///
    /// Uses a custom deserializer to handle both a single mountpoint (possibly null) and an array of mountpoints.
    #[serde(
        default,
        alias = "mountpoint",
        deserialize_with = "deserialize_mountpoints"
    )]
    pub mountpoints: Vec<Option<String>>,
    /// Optional nested children block devices.
    #[serde(default)]
    pub children: Option<Vec<BlockDevice>>,
}

impl BlockDevice {
    /// Returns `true` if this device has any children.
    #[must_use]
    pub fn has_children(&self) -> bool {
        self.children.as_ref().is_some_and(|c| !c.is_empty())
    }

    /// Returns an iterator over the children of this device.
    ///
    /// Returns an empty iterator if the device has no children.
    pub fn children_iter(&self) -> impl Iterator<Item = &BlockDevice> {
        self.children.iter().flat_map(|c| c.iter())
    }

    /// Finds a direct child device by name.
    ///
    /// Returns `None` if no child with the given name exists.
    #[must_use]
    pub fn find_child(&self, name: &str) -> Option<&BlockDevice> {
        self.children.as_ref()?.iter().find(|c| c.name == name)
    }

    /// Returns all non-null mountpoints for this device.
    #[must_use]
    pub fn active_mountpoints(&self) -> Vec<&str> {
        self.mountpoints
            .iter()
            .filter_map(|m| m.as_deref())
            .collect()
    }

    /// Returns `true` if this device has at least one mountpoint.
    #[must_use]
    pub fn is_mounted(&self) -> bool {
        self.mountpoints.iter().any(|m| m.is_some())
    }

    /// Determines if this block device or any of its recursive children has a mountpoint of `/`,
    /// indicating a system mount.
    #[must_use]
    pub fn is_system(&self) -> bool {
        if self.mountpoints.iter().any(|m| m.as_deref() == Some("/")) {
            return true;
        }
        if let Some(children) = &self.children {
            for child in children {
                if child.is_system() {
                    return true;
                }
            }
        }
        false
    }

    /// Returns `true` if this device is a disk.
    #[must_use]
    pub fn is_disk(&self) -> bool {
        self.device_type == DeviceType::Disk
    }

    /// Returns `true` if this device is a partition.
    #[must_use]
    pub fn is_partition(&self) -> bool {
        self.device_type == DeviceType::Part
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

    /// Returns an iterator over references to the block devices.
    pub fn iter(&self) -> Iter<'_, BlockDevice> {
        self.blockdevices.iter()
    }

    /// Returns a vector of references to `BlockDevice` entries that have a mountpoint
    /// of `/` on them or on any of their recursive children.
    #[must_use]
    pub fn system(&self) -> Vec<&BlockDevice> {
        self.blockdevices
            .iter()
            .filter(|device| device.is_system())
            .collect()
    }

    /// Returns a vector of references to `BlockDevice` entries that do not have a mountpoint
    /// of `/` on them or on any of their recursive children.
    #[must_use]
    pub fn non_system(&self) -> Vec<&BlockDevice> {
        self.blockdevices
            .iter()
            .filter(|device| !device.is_system())
            .collect()
    }

    /// Finds a top-level block device by name.
    ///
    /// Returns `None` if no device with the given name exists.
    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&BlockDevice> {
        self.blockdevices.iter().find(|d| d.name == name)
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

/// Parses a JSON string (produced by `lsblk --json`)
/// into a `BlockDevices` struct.
///
/// This function is useful when you already have JSON data from `lsblk`
/// and want to parse it without running the command again.
///
/// # Arguments
///
/// * `json_data` - A string slice containing the JSON data.
///
/// # Errors
///
/// Returns a `serde_json::Error` if the JSON cannot be parsed.
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

/// Runs the `lsblk --json` command, captures its output, and parses it
/// into a `BlockDevices` struct. If the command fails or the output cannot be parsed,
/// an error is returned.
///
/// # Errors
///
/// Returns an error if the `lsblk` command fails or if the output cannot be parsed as valid JSON.
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
    let lsblk = parse_lsblk(&json_output)?;
    Ok(lsblk)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_JSON: &str = r#"
    {
        "blockdevices": [
            {"name":"nvme1n1", "maj:min":"259:0", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme1n1p1", "maj:min":"259:1", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme1n1p9", "maj:min":"259:2", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme7n1", "maj:min":"259:3", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme7n1p1", "maj:min":"259:7", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme7n1p9", "maj:min":"259:8", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme5n1", "maj:min":"259:4", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme5n1p1", "maj:min":"259:5", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme5n1p9", "maj:min":"259:6", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme9n1", "maj:min":"259:9", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme9n1p1", "maj:min":"259:13", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme9n1p9", "maj:min":"259:14", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme4n1", "maj:min":"259:10", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme4n1p1", "maj:min":"259:11", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme4n1p9", "maj:min":"259:12", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme8n1", "maj:min":"259:15", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme8n1p1", "maj:min":"259:20", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme8n1p9", "maj:min":"259:21", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme6n1", "maj:min":"259:16", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme6n1p1", "maj:min":"259:17", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme6n1p9", "maj:min":"259:18", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme3n1", "maj:min":"259:19", "rm":false, "size":"894.3G", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme3n1p1", "maj:min":"259:23", "rm":false, "size":"1M", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme3n1p2", "maj:min":"259:24", "rm":false, "size":"244M", "ro":false, "type":"part", "mountpoint":"/boot/efi"},
                    {"name":"nvme3n1p3", "maj:min":"259:25", "rm":false, "size":"488M", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md0", "maj:min":"9:0", "rm":false, "size":"487M", "ro":false, "type":"raid1", "mountpoint":"/boot"}
                    ]
                    },
                    {"name":"nvme3n1p4", "maj:min":"259:26", "rm":false, "size":"7.6G", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md1", "maj:min":"9:1", "rm":false, "size":"7.6G", "ro":false, "type":"raid1", "mountpoint":"[SWAP]"}
                    ]
                    },
                    {"name":"nvme3n1p5", "maj:min":"259:27", "rm":false, "size":"19.1G", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md2", "maj:min":"9:2", "rm":false, "size":"19.1G", "ro":false, "type":"raid1", "mountpoint":"/"}
                    ]
                    },
                    {"name":"nvme3n1p6", "maj:min":"259:28", "rm":false, "size":"866.8G", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme0n1", "maj:min":"259:22", "rm":false, "size":"3.5T", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme0n1p1", "maj:min":"259:29", "rm":false, "size":"3.5T", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme0n1p9", "maj:min":"259:30", "rm":false, "size":"8M", "ro":false, "type":"part", "mountpoint":null}
                ]
            },
            {"name":"nvme2n1", "maj:min":"259:31", "rm":false, "size":"894.3G", "ro":false, "type":"disk", "mountpoint":null,
                "children": [
                    {"name":"nvme2n1p1", "maj:min":"259:32", "rm":false, "size":"1M", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme2n1p2", "maj:min":"259:33", "rm":false, "size":"244M", "ro":false, "type":"part", "mountpoint":null},
                    {"name":"nvme2n1p3", "maj:min":"259:34", "rm":false, "size":"488M", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md0", "maj:min":"9:0", "rm":false, "size":"487M", "ro":false, "type":"raid1", "mountpoint":"/boot"}
                    ]
                    },
                    {"name":"nvme2n1p4", "maj:min":"259:35", "rm":false, "size":"7.6G", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md1", "maj:min":"9:1", "rm":false, "size":"7.6G", "ro":false, "type":"raid1", "mountpoint":"[SWAP]"}
                    ]
                    },
                    {"name":"nvme2n1p5", "maj:min":"259:36", "rm":false, "size":"19.1G", "ro":false, "type":"part", "mountpoint":null,
                    "children": [
                        {"name":"md2", "maj:min":"9:2", "rm":false, "size":"19.1G", "ro":false, "type":"raid1", "mountpoint":"/"}
                    ]
                    },
                    {"name":"nvme2n1p6", "maj:min":"259:37", "rm":false, "size":"866.8G", "ro":false, "type":"part", "mountpoint":null}
                ]
            }
        ]
    }
    "#;

    #[test]
    fn test_parse_lsblk() {
        let lsblk = parse_lsblk(SAMPLE_JSON).expect("Failed to parse JSON");

        // Assert the expected number of top-level block devices.
        assert_eq!(
            lsblk.blockdevices.len(),
            10,
            "Expected 10 top-level block devices"
        );

        // Verify that required fields are non-empty.
        for device in &lsblk.blockdevices {
            assert!(!device.name.is_empty(), "Device name should not be empty");
            assert!(
                !device.maj_min.is_empty(),
                "Device maj:min should not be empty"
            );
        }

        // Pick a device with nested children and validate details.
        let nvme3n1 = lsblk
            .blockdevices
            .iter()
            .find(|d| d.name == "nvme3n1")
            .expect("Expected to find device nvme3n1");

        // Its first mountpoint should be None.
        assert!(
            nvme3n1
                .mountpoints
                .first()
                .and_then(|opt| opt.as_deref())
                .is_none(),
            "nvme3n1 effective mountpoint should be None"
        );

        // Verify that nvme3n1 has exactly 6 children.
        let children = nvme3n1
            .children
            .as_ref()
            .expect("nvme3n1 should have children");
        assert_eq!(children.len(), 6, "nvme3n1 should have 6 children");

        // Validate that child nvme3n1p2 has first mountpoint of "/boot/efi".
        let nvme3n1p2 = children
            .iter()
            .find(|c| c.name == "nvme3n1p2")
            .expect("Expected to find nvme3n1p2");
        assert_eq!(
            nvme3n1p2.mountpoints.first().and_then(|opt| opt.as_deref()),
            Some("/boot/efi"),
            "nvme3n1p2 first mountpoint should be '/boot/efi'"
        );

        // In nvme3n1p3, verify that its nested child md0 has an effective mountpoint of "/boot".
        let nvme3n1p3 = children
            .iter()
            .find(|c| c.name == "nvme3n1p3")
            .expect("Expected to find nvme3n1p3");
        let nested_children = nvme3n1p3
            .children
            .as_ref()
            .expect("nvme3n1p3 should have children");
        let md0 = nested_children
            .iter()
            .find(|d| d.name == "md0")
            .expect("Expected to find md0 under nvme3n1p3");
        assert_eq!(
            md0.mountpoints.first().and_then(|opt| opt.as_deref()),
            Some("/boot"),
            "md0 effective mountpoint should be '/boot'"
        );

        // Test the non_system method.
        // Since nvme3n1 has a descendant (md2) with effective mountpoint "/" it should be excluded.
        let non_system = lsblk.non_system();
        assert_eq!(
            non_system.len(),
            8,
            "Expected 8 non-system top-level devices, since nvme3n1/nvme2n1 is system"
        );
        assert!(
            !non_system.iter().any(|d| d.name == "nvme3n1"),
            "nvme3n1 should be excluded from non-system devices"
        );
    }

    #[test]
    fn test_non_system() {
        // Create a JSON where one device is system (has "/" mountpoint in a child)
        // and one is non-system.
        let test_json = r#"
        {
            "blockdevices": [
                {
                    "name": "sda",
                    "maj:min": "8:0",
                    "rm": false,
                    "size": "447.1G",
                    "ro": false,
                    "type": "disk",
                    "mountpoints": [
                        null
                    ],
                    "children": [
                        {
                        "name": "sda1",
                        "maj:min": "8:1",
                        "rm": false,
                        "size": "512M",
                        "ro": false,
                        "type": "part",
                        "mountpoints": [
                            null
                        ]
                        },{
                        "name": "sda2",
                        "maj:min": "8:2",
                        "rm": false,
                        "size": "446.6G",
                        "ro": false,
                        "type": "part",
                        "mountpoints": [
                            null
                        ],
                        "children": [
                            {
                                "name": "md0",
                                "maj:min": "9:0",
                                "rm": false,
                                "size": "446.6G",
                                "ro": false,
                                "type": "raid1",
                                "mountpoints": [
                                    "/"
                                ]
                            }
                        ]
                        }
                    ]
                },{
                    "name": "sdb",
                    "maj:min": "8:16",
                    "rm": false,
                    "size": "447.1G",
                    "ro": false,
                    "type": "disk",
                    "mountpoints": [
                        null
                    ],
                    "children": [
                        {
                        "name": "sdb1",
                        "maj:min": "8:17",
                        "rm": false,
                        "size": "512M",
                        "ro": false,
                        "type": "part",
                        "mountpoints": [
                            "/boot/efi"
                        ]
                        },{
                        "name": "sdb2",
                        "maj:min": "8:18",
                        "rm": false,
                        "size": "446.6G",
                        "ro": false,
                        "type": "part",
                        "mountpoints": [
                            null
                        ],
                        "children": [
                            {
                                "name": "md0",
                                "maj:min": "9:0",
                                "rm": false,
                                "size": "446.6G",
                                "ro": false,
                                "type": "raid1",
                                "mountpoints": [
                                    "/"
                                ]
                            }
                        ]
                        }
                    ]
                },{
                    "name": "nvme0n1",
                    "maj:min": "259:2",
                    "rm": false,
                    "size": "1.7T",
                    "ro": false,
                    "type": "disk",
                    "mountpoints": [
                        null
                    ]
                },{
                    "name": "nvme1n1",
                    "maj:min": "259:3",
                    "rm": false,
                    "size": "1.7T",
                    "ro": false,
                    "type": "disk",
                    "mountpoints": [
                        null
                    ]
                }
            ]
        }
        "#;
        let disks = parse_lsblk(test_json).unwrap();
        let non_system = disks.non_system();
        assert_eq!(non_system.len(), 2);
        let names: Vec<&str> = non_system.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["nvme0n1", "nvme1n1"]);
    }

    /// Warning: This test will attempt to run the `lsblk` command on your system.
    /// It may fail if `lsblk` is not available or if the test environment does not permit running commands.
    #[test]
    #[ignore = "requires lsblk command to be available on the system"]
    fn test_get_devices() {
        let dev = get_devices().expect("Failed to get block devices");
        // This assertion is simplistic; adjust according to your environment's expected output.
        assert!(!dev.blockdevices.is_empty());
    }
    #[test]
    fn test_into_iterator() {
        // Create dummy BlockDevice instances.
        let device1 = BlockDevice {
            name: "sda".to_string(),
            maj_min: "8:0".to_string(),
            rm: false,
            size: 536_870_912_000, // 500G in bytes
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: None,
        };

        let device2 = BlockDevice {
            name: "sdb".to_string(),
            maj_min: "8:16".to_string(),
            rm: false,
            size: 536_870_912_000, // 500G in bytes
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: None,
        };

        // Create a BlockDevices instance containing the two devices.
        let devices = BlockDevices {
            blockdevices: vec![device1, device2],
        };

        // Use the IntoIterator implementation to iterate over the devices.
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
        let json = r#"{"blockdevices": [{"name": "sda", "maj:min": "8:0", "rm": false, "size": "500G", "ro": false, "type": "disk", "mountpoints": [null]}]}"#;
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
                "name": "sda",
                "maj:min": "8:0",
                "rm": false,
                "size": "500G",
                "ro": false,
                "type": "disk",
                "mountpoints": ["/"]
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
            maj_min: "8:0".to_string(),
            rm: false,
            size: 536_870_912_000, // 500G in bytes
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![Some("/mnt/data".to_string()), None],
            children: Some(vec![BlockDevice {
                name: "sda1".to_string(),
                maj_min: "8:1".to_string(),
                rm: false,
                size: 268_435_456_000, // 250G in bytes
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
            maj_min: "8:0".to_string(),
            rm: false,
            size: 536_870_912_000, // 500G in bytes
            ro: false,
            device_type: DeviceType::Disk,
            mountpoints: vec![None],
            children: Some(vec![
                BlockDevice {
                    name: "sda1".to_string(),
                    maj_min: "8:1".to_string(),
                    rm: false,
                    size: 268_435_456_000, // 250G in bytes
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "sda2".to_string(),
                    maj_min: "8:2".to_string(),
                    rm: false,
                    size: 268_435_456_000, // 250G in bytes
                    ro: false,
                    device_type: DeviceType::Part,
                    mountpoints: vec![None],
                    children: None,
                },
            ]),
        };

        let names: Vec<&str> = device.children_iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["sda1", "sda2"]);

        // Test empty children iterator
        let device_no_children = BlockDevice {
            name: "sdb".to_string(),
            maj_min: "8:16".to_string(),
            rm: false,
            size: 536_870_912_000, // 500G in bytes
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
                    maj_min: "8:0".to_string(),
                    rm: false,
                    size: 536_870_912_000, // 500G in bytes
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "sdb".to_string(),
                    maj_min: "8:16".to_string(),
                    rm: false,
                    size: 536_870_912_000, // 500G in bytes
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
            ],
        };

        // Test borrowing iterator (doesn't consume)
        let names: Vec<&str> = (&devices).into_iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["sda", "sdb"]);

        // devices is still available
        assert_eq!(devices.len(), 2);

        // Test iter() method
        let names2: Vec<&str> = devices.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names2, vec!["sda", "sdb"]);
    }

    #[test]
    fn test_find_by_name() {
        let devices = BlockDevices {
            blockdevices: vec![
                BlockDevice {
                    name: "sda".to_string(),
                    maj_min: "8:0".to_string(),
                    rm: false,
                    size: 536_870_912_000, // 500G in bytes
                    ro: false,
                    device_type: DeviceType::Disk,
                    mountpoints: vec![None],
                    children: None,
                },
                BlockDevice {
                    name: "nvme0n1".to_string(),
                    maj_min: "259:0".to_string(),
                    rm: false,
                    size: 1_099_511_627_776, // 1T in bytes
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
                {"name": "sda", "maj:min": "8:0", "rm": false, "size": "500G", "ro": false, "type": "disk", "mountpoints": ["/"]},
                {"name": "sdb", "maj:min": "8:16", "rm": false, "size": "500G", "ro": false, "type": "disk", "mountpoints": [null]},
                {"name": "sdc", "maj:min": "8:32", "rm": false, "size": "500G", "ro": false, "type": "disk", "mountpoints": ["/home"]}
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
                "name": "sda",
                "maj:min": "8:0",
                "rm": false,
                "size": "500G",
                "ro": false,
                "type": "disk",
                "mountpoints": ["/mnt/data", "/mnt/backup", null]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let device = devices.find_by_name("sda").unwrap();
        assert!(device.is_mounted());
        assert_eq!(
            device.active_mountpoints(),
            vec!["/mnt/data", "/mnt/backup"]
        );
    }

    #[test]
    fn test_removable_and_readonly() {
        let json = r#"{
            "blockdevices": [{
                "name": "sr0",
                "maj:min": "11:0",
                "rm": true,
                "size": "4.7G",
                "ro": true,
                "type": "rom",
                "mountpoints": [null]
            }]
        }"#;
        let devices = parse_lsblk(json).expect("Failed to parse JSON");
        let device = devices.find_by_name("sr0").unwrap();
        assert!(device.rm);
        assert!(device.ro);
        assert_eq!(device.device_type, DeviceType::Rom);
    }
}
