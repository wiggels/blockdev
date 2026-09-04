# blockdev

[![CI](https://github.com/wiggels/blockdev/actions/workflows/ci.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/ci.yml)
[![Audit](https://github.com/wiggels/blockdev/actions/workflows/audit.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/audit.yml)
[![Benchmarks](https://github.com/wiggels/blockdev/actions/workflows/bench.yml/badge.svg)](https://github.com/wiggels/blockdev/actions/workflows/bench.yml)
[![Coverage](https://codecov.io/gh/wiggels/blockdev/branch/main/graph/badge.svg)](https://codecov.io/gh/wiggels/blockdev)
[![Crates.io](https://img.shields.io/crates/v/blockdev.svg)](https://crates.io/crates/blockdev)
[![Documentation](https://docs.rs/blockdev/badge.svg)](https://docs.rs/blockdev)
[![MSRV](https://img.shields.io/badge/rustc-1.85+-blue.svg)](https://blog.rust-lang.org/)
[![License](https://img.shields.io/crates/l/blockdev.svg)](https://github.com/wiggels/blockdev/blob/main/LICENSE)

A small, dependency-free Rust library for enumerating Linux block devices.
It walks `/sys` and `/proc` directly and builds a typed tree of disks,
partitions, and whatever is stacked on top (RAID, LVM, dm-crypt). No
`lsblk` process, no JSON, a full walk in the low hundreds of microseconds.

The rules are the same ones `lsblk` uses internally, and CI asserts the
result is identical to `lsblk --json --bytes` on a live machine.

## Features

- **Zero dependencies** - std only; `serde` derives behind an optional feature
- **Type-safe tree** - `BlockDevices` / `BlockDevice` / `DeviceType` / `MajMin`
- **Same answer as lsblk** - roots, partitions, holders, types, mountpoints, swap, ordering
- **System device detection** - find the disk holding `/`, through any stack depth
- **Stable identifiers** - `uuid`, `partuuid`, `fstype`, `label`, `wwn`, `serial`, `model` from the udev database, with sysfs fallbacks
- **Alternate sysroots** - point it at a bind-mounted host `/sys` from a container, or a fake tree in tests
- **Idiomatic iteration** - `IntoIterator`, pre-order `descendants()`, iterator and `Vec` flavors of every filter

## Installation

```toml
[dependencies]
blockdev = "0.5"
```

Enable `serde` if you want to serialize the tree:

```toml
[dependencies]
blockdev = { version = "0.5", features = ["serde"] }
```

## Quick Start

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    for device in &devices {
        println!("{}: {} bytes, type: {}", device.name, device.size, device.device_type);
    }

    Ok(())
}
```

## Usage Examples

### List all non-system disks

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    // disks that do not hold the root filesystem, at any depth
    for device in devices.non_system() {
        if device.is_disk() {
            println!("available disk: {} ({} bytes)", device.name, device.size);
        }
    }

    Ok(())
}
```

### Inspect a specific device

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    if let Some(device) = devices.find_by_name("sda") {
        println!("sda: {} bytes, {}, rm={} ro={}", device.size, device.maj_min, device.rm, device.ro);

        for child in &device.children {
            println!("  {} [{}] {} bytes mounted at {:?}", child.name, child.device_type, child.size, child.mountpoints);
        }
    }

    Ok(())
}
```

### Walk the whole tree

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    // every device, pre-order, including RAID / LVM / crypt layers
    for device in devices.iter_all() {
        if device.is_mounted() {
            println!("{} is mounted at {:?}", device.name, device.mountpoints);
        }
    }

    // or from one root
    if let Some(md0) = devices.find_anywhere("md0") {
        println!("md0 is a {}", md0.device_type);
    }

    Ok(())
}
```

### Find a device by UUID, not by name

```rust
use blockdev::get_devices;

fn main() -> Result<(), blockdev::BlockDevError> {
    let devices = get_devices()?;

    // sda can be sdb tomorrow -- the filesystem uuid cannot
    let target = devices
        .iter_all()
        .find(|d| d.uuid.as_deref() == Some("3f1a2b4c-0000-4000-8000-0000deadbeef"));

    if let Some(part) = target {
        println!("{} is {:?} on wwn {:?}", part.name, part.fstype, part.wwn);
    }

    Ok(())
}
```

### Another sysroot

```rust
use blockdev::get_devices_at;

fn main() -> Result<(), blockdev::BlockDevError> {
    // host /sys and /proc bind-mounted under /host inside a container
    let devices = get_devices_at("/host")?;
    println!("{} top-level devices on the host", devices.len());
    Ok(())
}
```

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `get_devices()` | Enumerate block devices from `/sys` and `/proc` |
| `get_devices_at(sysroot)` | Same, from a tree rooted elsewhere (`lsblk --sysroot`) |

### Types

#### `BlockDevices`

The top-level devices, sorted by `maj:min`.

| Method | Description |
|--------|-------------|
| `len()` / `is_empty()` | Count of top-level devices |
| `iter()` | Top-level devices |
| `iter_all()` | Every device in the tree, pre-order |
| `system()` / `system_iter()` | Devices holding `/`, directly or via a descendant |
| `non_system()` / `non_system_iter()` | Everything else |
| `find_by_name(name)` | Top-level device by name |
| `find_anywhere(name)` | Any device in the tree by name |

#### `BlockDevice`

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Device name; device-mapper devices use their mapper name |
| `maj_min` | `MajMin` | Major and minor numbers |
| `rm` | `bool` | Removable; partitions inherit from their disk |
| `size` | `u64` | Size in bytes |
| `ro` | `bool` | Read-only |
| `device_type` | `DeviceType` | Type of device |
| `mountpoints` | `Vec<String>` | Every mountpoint, newest first; `"[SWAP]"` for active swap; empty if unmounted |
| `children` | `Vec<BlockDevice>` | Partitions and holders, sorted by `maj:min` |
| `uuid` | `Option<String>` | Filesystem UUID (udev) |
| `partuuid` | `Option<String>` | Partition entry UUID (udev) |
| `fstype` | `Option<String>` | Filesystem type, e.g. `ext4`, `crypto_LUKS` (udev) |
| `label` | `Option<String>` | Filesystem label (udev) |
| `partlabel` | `Option<String>` | GPT partition name (udev) |
| `wwn` | `Option<String>` | World Wide Name (udev, then sysfs `wwid`) |
| `serial` | `Option<String>` | Disk serial; whole disks only (udev, then sysfs) |
| `model` | `Option<String>` | Disk model; whole disks only (udev, then sysfs) |

| Method | Description |
|--------|-------------|
| `has_children()` | Any partitions or holders |
| `descendants()` | `self` then every descendant, pre-order |
| `find_child(name)` / `find_descendant(name)` | Direct or recursive lookup |
| `is_mounted()` | Has at least one mountpoint |
| `is_system()` | Holds `/` somewhere in its subtree |
| `is_disk()` / `is_partition()` | Type shortcuts |

#### `DeviceType`

`Disk`, `Part`, `Loop`, `Raid0`, `Raid1`, `Raid5`, `Raid6`, `Raid10`, `Lvm`,
`Crypt`, `Rom`, and `Other` for anything else. Implements `Display` and an
infallible `FromStr` using `lsblk`'s names.

#### `MajMin`

`major` / `minor` as `u32`. Implements `Display` (`"8:0"`), `FromStr`,
`Ord`, and `Hash`.

#### `BlockDevError`

`Io { path, source }` when `<sysroot>/sys/block` or
`<sysroot>/proc/self/mountinfo` cannot be read. Missing attributes on an
individual device are treated as defaults, not errors, so a device being
torn down mid-walk does not fail the call. The enum is `#[non_exhaustive]`.

## How it works

`lsblk` is a thin layer over sysfs, so `blockdev` reads the same files:

- Roots come from `/sys/block`, minus RAM disks and anything with a
  non-empty `slaves/` (those show under their parent instead).
- Partitions are subdirectories named `<disk>N` or `<disk>pN`. Holders in
  `holders/` (RAID, LVM, crypt) become children too, recursively.
- `size` is sectors times 512. `ro` and `removable` are the attributes;
  partitions inherit `removable` from the disk.
- Type is `part`, the `dm/uuid` prefix for device-mapper (`LVM`, `CRYPT`),
  `loop`, the `md/level` for RAID, the SCSI `device/type` for optical, else
  `disk`.
- Mountpoints are every `/proc/self/mountinfo` entry matching by `maj:min`
  or source path, newest first, then `/proc/swaps`.
- Identifiers come from udev's database at `/run/udev/data/b<maj>:<min>`,
  using the same property names and priority `lsblk` uses through libudev.
  `wwn`, `serial`, and `model` fall back to sysfs attributes. Without udev
  the filesystem-level fields are `None`; the crate never reads device
  contents the way `blkid` does.

See [docs/sysfs-backend.md](docs/sysfs-backend.md) for the full mapping
and what is not covered.

## Performance

A full walk is pure syscalls: roughly 20 per device. On a laptop or normal
KVM guest that is on the order of 100 µs for a typical machine. The old
approach of spawning `lsblk` cost 3 to 4 ms per call; the benchmark keeps
an `lsblk` spawn as a reference so the gap stays visible over time.

### Regression detection

Three layers of defense against perf regressions:

1. **Committed perf-budget tests** (`tests/perf_budgets.rs`) -- wall-clock
   bounds with ~20x headroom over a slow VM, run as part of `cargo test` on
   any machine. They catch catastrophic regressions without any CI setup.

   ```sh
   cargo test --release --test perf_budgets -- --nocapture
   ```

2. **CI-side benchmark gate** (`.github/workflows/bench.yml`) -- on every
   PR, benches the merge base and then the PR head back to back on the same
   runner over a deterministic fake sysroot, and **fails the build** if
   anything is more than 25% slower. Same-runner A/B because GitHub-hosted
   runners vary 2 to 3x between machines. Pushes to main append to the
   history plotted at the repo's GitHub Pages site under `/dev/bench/` via
   [`benchmark-action/github-action-benchmark`](https://github.com/benchmark-action/github-action-benchmark).
   That page has two sections: raw nanoseconds, which mostly track which
   runner class a run landed on, and a normalized series where every bench
   is divided by a fixed-work calibration bench from the same run. Read
   the normalized one for code changes.

3. **Local criterion baselines** (`benches/devices.rs`):

   ```sh
   cargo bench --bench devices -- --save-baseline main
   # ... make changes ...
   cargo bench --bench devices -- --baseline main
   ```

## Requirements

- Linux, with sysfs mounted at `/sys` and procfs at `/proc` (or another
  sysroot via `get_devices_at`)
- Rust 1.85+ (MSRV, verified in CI)

`lsblk` itself is not required at runtime. The test suite spawns it only in
one ignored test that checks the walk against it.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for the PR checklist, commit
conventions, and the release process.

## License

MIT License - see [LICENSE](LICENSE) for details.
