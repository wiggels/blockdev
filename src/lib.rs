use serde::de::Error as DeError;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::error::Error;
use std::process::Command;

/// Represents the entire JSON output produced by `lsblk --json`.
#[derive(Serialize, Deserialize, Debug)]
pub struct BlockDevices {
    /// A vector of block devices.
    pub blockdevices: Vec<BlockDevice>,
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
/// - `maj_min`: The device’s major and minor numbers. (Renamed from the JSON field "maj:min")
/// - `rm`: Whether the device is removable.
/// - `size`: The device size.
/// - `ro`: Whether the device is read-only.
/// - `device_type`: The device type (renamed from the reserved keyword "type").
/// - `mountpoints`: A vector of mountpoints for the device. Uses a custom deserializer to support both single and multiple mountpoints.
/// - `children`: Optional nested block devices.
#[derive(Serialize, Deserialize, Debug)]
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
    /// The size of the block device.
    pub size: String,
    /// Indicates if the device is read-only.
    pub ro: bool,
    /// The type of the block device.
    ///
    /// The JSON field is `"type"`, which is a reserved keyword in Rust. It is renamed to `device_type`.
    #[serde(rename = "type")]
    pub device_type: String,
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
    /// Determines if this block device or any of its recursive children has a mountpoint of `/`,
    /// indicating a system mount.
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
}

impl BlockDevices {
    /// Returns a vector of references to `BlockDevice` entries that do not have a mountpoint
    /// of `/` on them or on any of their recursive children.
    pub fn non_system(&self) -> Vec<&BlockDevice> {
        self.blockdevices
            .iter()
            .filter(|device| !device.is_system())
            .collect()
    }
}

/// Parses a JSON string (produced by `lsblk --json`)
/// into a `BlockDevices` struct.
///
/// # Arguments
///
/// * `json_data` - A string slice containing the JSON data.
///
/// # Errors
///
/// Returns a `serde_json::Error` if the JSON cannot be parsed.
fn parse_lsblk(json_data: &str) -> Result<BlockDevices, serde_json::Error> {
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
/// # use blovkdev::get_devices;
/// let devices = get_devices().expect("Failed to get block devices");
/// ```
pub fn get_devices() -> Result<BlockDevices, Box<dyn Error>> {
    let output = Command::new("lsblk").arg("--json").output()?;

    if !output.status.success() {
        return Err(format!("lsblk failed: {}", String::from_utf8_lossy(&output.stderr)).into());
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
    fn test_get_devices() {
        let dev = get_devices().expect("Failed to get block devices");
        // This assertion is simplistic; adjust according to your environment's expected output.
        assert!(!dev.blockdevices.is_empty());
    }
}
