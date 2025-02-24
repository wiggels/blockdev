# blockdev

[![Crates.io](https://img.shields.io/crates/v/blockdev.svg)](https://crates.io/crates/blockdev)
[![Documentation](https://docs.rs/blockdev/badge.svg)](https://docs.rs/blockdev)
[![License](https://img.shields.io/crates/l/blockdev.svg)](https://github.com/wiggels/blockdev/blob/main/LICENSE)

blockdev is a lightweight Rust library for parsing and working with the output of the `lsblk --json` command on Linux. It leverages Serde for JSON deserialization, providing a type-safe representation of block devices and a set of utilities to inspect their properties.

## Features

- **JSON Parsing:** Easily parse the JSON output of `lsblk --json` into Rust data structures.
- **Flexible Mountpoint Support:** Handles both single mountpoint values (which may be null) and arrays of mountpoints.
- **System Device Detection:** Determine whether a block device (or any of its nested children) is a system device (i.e. has a mountpoint of `/`).
- **Filtering Utilities:** Provides helper functions to filter out non-system devices.

---

## Installation

Add `blockdev` to your `Cargo.toml`:

```toml
[dependencies]
blockdev = "0.1.0"
```

Then run:

```shell
cargo build
```

---

## Usage

### Get devices

```rust
use blockdev::get_devices;

fn example_devices() -> Result<BlockDevices, Box<dyn Error>> {
    let devices = get_devices();
    devices
}
```
