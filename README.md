# blockdev

[![CI](https://github.com/wiggels/blockdev/actions/workflows/ci.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/ci.yml)
[![Audit](https://github.com/wiggels/blockdev/actions/workflows/audit.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/audit.yml)
[![Benchmarks](https://github.com/wiggels/blockdev/actions/workflows/bench.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/bench.yml)
[![Coverage](https://codecov.io/gh/wiggels/blockdev/branch/main/graph/badge.svg)](https://codecov.io/gh/wiggels/blockdev)
[![Crates.io](https://img.shields.io/crates/v/blockdev.svg)](https://crates.io/crates/blockdev)
[![Documentation](https://docs.rs/blockdev/badge.svg)](https://docs.rs/blockdev)
[![MSRV](https://img.shields.io/badge/rustc-1.85+-blue.svg)](https://blog.rust-lang.org/)
[![License](https://img.shields.io/crates/l/blockdev.svg)](https://github.com/wiggels/blockdev/blob/main/LICENSE)

A lightweight Rust library for parsing and working with `lsblk --json` output on Linux. Provides type-safe block device representation with utilities for inspecting device properties, filtering system devices, and iterating over device hierarchies.

## Features

- **Type-safe parsing** - Strongly typed structs and enums for all block device properties
- **Flexible mountpoint handling** - Supports both single and multiple mountpoints per device
- **System device detection** - Identify devices mounted at `/` (including nested children)
- **Device filtering** - Helper methods to separate system and non-system devices
- **Size parsing** - Handles both byte values and human-readable sizes (e.g., `500G`, `3.5T`)
- **Full Serde support** - Serialize and deserialize block device data
- **Idiomatic iteration** - Implements `IntoIterator` for ergonomic Rust patterns

## Installation

Add `blockdev` to your `Cargo.toml`:

```toml
[dependencies]
blockdev = "0.4"
```

## Quick Start

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    // Get all block devices from the system
    let devices = get_devices()?;

    // Iterate over devices
    for device in &devices {
        println!("{}: {} bytes, type: {:?}",
            device.name,
            device.size,
            device.device_type
        );
    }

    Ok(())
}
```

## Usage Examples

### List All Non-System Disks

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    // Get disks that don't contain the root filesystem
    for device in devices.non_system() {
        if device.is_disk() {
            println!("Available disk: {} ({} bytes)", device.name, device.size);
        }
    }

    Ok(())
}
```

### Find a Specific Device

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    if let Some(device) = devices.find_by_name("sda") {
        println!("Found sda:");
        println!("  Size: {} bytes", device.size);
        println!("  Type: {:?}", device.device_type);
        println!("  Major:Minor: {}", device.maj_min);
        println!("  Removable: {}", device.rm);
        println!("  Read-only: {}", device.ro);

        // Check partitions
        if device.has_children() {
            println!("  Partitions:");
            for child in device.children_iter() {
                println!("    - {}: {} bytes", child.name, child.size);
            }
        }
    }

    Ok(())
}
```

### Parse Pre-Existing JSON

```rust
use blockdev::parse_lsblk;

fn main() -> Result<(), serde_json::Error> {
    let json = r#"{
        "blockdevices": [{
            "name": "sda",
            "maj:min": "8:0",
            "rm": false,
            "size": 500107862016,
            "ro": false,
            "type": "disk",
            "mountpoints": [null],
            "children": [{
                "name": "sda1",
                "maj:min": "8:1",
                "rm": false,
                "size": 500106813440,
                "ro": false,
                "type": "part",
                "mountpoints": ["/data"]
            }]
        }]
    }"#;

    let devices = parse_lsblk(json)?;
    println!("Parsed {} devices", devices.len());

    Ok(())
}
```

### Check Mountpoints

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    for device in &devices {
        if device.is_mounted() {
            println!("{} is mounted at: {:?}",
                device.name,
                device.active_mountpoints()
            );
        }

        // Check children too
        for child in device.children_iter() {
            if child.is_mounted() {
                println!("  {} is mounted at: {:?}",
                    child.name,
                    child.active_mountpoints()
                );
            }
        }
    }

    Ok(())
}
```

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `get_devices()` | Execute `lsblk --json --bytes` and parse the output |
| `parse_lsblk(json)` | Parse a JSON string from `lsblk` without executing the command |

### Types

#### `BlockDevices`

Container for the parsed `lsblk` output.

| Method | Description |
|--------|-------------|
| `len()` | Number of top-level devices |
| `is_empty()` | Check if no devices exist |
| `iter()` | Iterate over device references |
| `system()` | Get devices containing the root mountpoint |
| `non_system()` | Get devices not containing the root mountpoint |
| `find_by_name(name)` | Find a device by name |

#### `BlockDevice`

Represents a single block device.

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Device name (e.g., `sda`, `nvme0n1`) |
| `maj_min` | `MajMin` | Major and minor device numbers |
| `rm` | `bool` | Whether the device is removable |
| `size` | `u64` | Size in bytes |
| `ro` | `bool` | Whether the device is read-only |
| `device_type` | `DeviceType` | Type of device |
| `mountpoints` | `Vec<Option<String>>` | Mountpoint(s) for the device |
| `children` | `Option<Vec<BlockDevice>>` | Nested devices (e.g., partitions) |

| Method | Description |
|--------|-------------|
| `has_children()` | Check if the device has child devices |
| `children_iter()` | Iterate over child devices |
| `find_child(name)` | Find a child device by name |
| `active_mountpoints()` | Get all non-null mountpoints |
| `is_mounted()` | Check if the device has any mountpoint |
| `is_system()` | Check if the device or children contain `/` |
| `is_disk()` | Check if device type is `Disk` |
| `is_partition()` | Check if device type is `Part` |

#### `DeviceType`

Enum representing block device types:

- `Disk` - Physical disk
- `Part` - Partition
- `Loop` - Loop device
- `Raid0`, `Raid1`, `Raid5`, `Raid6`, `Raid10` - RAID arrays
- `Lvm` - LVM logical volume
- `Crypt` - dm-crypt device
- `Rom` - CD/DVD drive
- `Other` - Unknown device type

#### `MajMin`

Represents major and minor device numbers.

| Field | Type | Description |
|-------|------|-------------|
| `major` | `u32` | Major device number |
| `minor` | `u32` | Minor device number |

Implements `Display` to format as `major:minor`.

#### `BlockDevError`

Error type for library operations:

- `CommandFailed` - Failed to execute `lsblk`
- `LsblkError` - `lsblk` returned non-zero exit status
- `InvalidUtf8` - Output contained invalid UTF-8
- `JsonParse` - Failed to parse JSON output

## Requirements

- Linux operating system (for `lsblk` command)
- `lsblk` with JSON output support (util-linux 2.27+)
- Rust 1.85+ (MSRV, verified in CI)

`parse_lsblk` and every type/filter in the crate are pure and work on any
platform. Only `get_devices()` needs a real `lsblk`.

## Performance

Parsing is a single pass through `serde_json` straight into the typed structs.
On a 2024 laptop a realistic 10-disk `lsblk` dump parses in tens of
microseconds. Benchmark history is plotted at the repo's GitHub Pages site.

### Regression detection

Three layers of defense against perf regressions:

1. **Committed perf-budget tests** (`tests/perf_budgets.rs`) -- wall-clock
   bounds with 20-50x headroom, run as part of `cargo test` on any machine.
   They catch catastrophic regressions without any CI setup.

   ```sh
   cargo test --release --test perf_budgets -- --nocapture
   ```

2. **CI-side benchmark comparison** (`.github/workflows/bench.yml`) -- runs
   `cargo bench` on every PR, compares against the `main` baseline stored on
   the `gh-pages` branch, and **fails the build** if anything regresses by
   more than 25%. Posts a comment on the PR with the diff. History is
   plotted at the repo's GitHub Pages site under `/dev/bench/`. Powered by
   [`benchmark-action/github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark).

3. **Local criterion baselines** (`benches/parse.rs`) -- for fine-grained
   work on a single machine:

   ```sh
   cargo bench --bench parse -- --save-baseline main
   # ... make changes ...
   cargo bench --bench parse -- --baseline main
   ```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the PR checklist, commit
conventions, and the release process.

## License

MIT License - see [LICENSE](LICENSE) for details.
