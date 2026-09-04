# sysfs backend

Why `blockdev` reads `/sys` itself instead of running `lsblk`, what it
reads, and what it does not cover.

## Background

Through 0.4 the crate spawned `lsblk --json --bytes` and deserialized the
output with serde. The obvious next step looked like calling lsblk's C code
directly to skip the JSON. That does not work, for three reasons:

- **There is no library.** `lsblk-cmd/` in util-linux is the lsblk command
  relocated from `misc-utils/`. Its build files define `bin_PROGRAMS +=
  lsblk` and nothing else. `lsblk.h` is an internal header with a global
  `struct lsblk *lsblk` handler that `main()` owns.
- **License.** lsblk is GPL-2.0-or-later. Linking it into an MIT crate would
  relicense every downstream binary.
- **Weight.** lsblk links libblkid, libmount, libsmartcols, libtcolors, and
  optionally libudev.

And the JSON was never the cost. Measured on the 0.4 code:

| step | time |
|---|---|
| fork+exec lsblk, wait, read stdout | ~3.8 ms |
| parse that output | ~4 µs |

Deserialization was 0.1% of a request. The only lever is not starting a
process. lsblk is itself a thin layer over sysfs, so 0.5 reads the same
files in Rust.

## What is read

From `lsblk-cmd/lsblk.c`, `devtree.c`, `mnt.c`, and `lib/sysfs.c`:

- **Roots**: `readdir(/sys/block)`. Major 1 (RAM disks) is excluded, as
  lsblk does by default. Anything with a non-empty `slaves/` directory is
  in the middle of a stack and is shown under its parent instead. Zero-size
  loop devices with no `loop/backing_file` are hidden.
- **Attributes**: `dev` for `maj:min`, `size` in 512-byte sectors, `ro`,
  `removable`. A partition inherits `removable` from its disk. Each is one
  open/read/close on a stack buffer rather than `read_to_string`, because
  the walk is pure syscall overhead.
- **Partitions**: subdirectories of the disk named `<disk>N` or
  `<disk>pN`. Roots skip the check entirely since `/sys/block` never lists
  partitions; holders still check for a `start` file (kpartx).
- **Holders**: names in `holders/`, resolved through
  `/sys/class/block/<name>`, recursively. Depth is capped at 32.
- **Type**: `part` for partitions; the `dm/uuid` prefix lowercased for
  `dm-*` (LVM, CRYPT, mpath; the kpartx `partN-` prefix folds to `part`;
  no uuid means `dm`); `loop`; `md/level` for `md*`; SCSI `device/type`
  otherwise (5 is `rom`), defaulting to `disk`.
- **Names**: `!` becomes `/` (`cciss!c0d0` is `/dev/cciss/c0d0`), and
  `dm-N` shows as its `dm/name`.
- **Mountpoints**: every `/proc/self/mountinfo` line matching by `maj:min`
  or by source path (`/dev/<name>` or `/dev/mapper/<dm name>`), newest
  first, with octal escapes decoded. If none, `/proc/swaps` for `[SWAP]`.
- **Order**: everything sorted by `maj:min`, because `/sys` stopped being
  sorted in Linux 4.8 and lsblk sorts by default since.

## How it is checked

- `tests/lsblk_equivalence.rs` diffs `get_devices()` against
  `lsblk --json --bytes` on the running machine. Ignored by default; the
  Linux CI job runs it.
- `tests/layouts.rs` and the unit tests in `src/sysfs.rs` build fake
  sysroots for layouts a CI box does not have: RAID1 over partitions with
  swap, LUKS then LVM, hidden RAM and loop devices, removable inheritance,
  SCSI optical, escaped and bind mountpoints.

## Numbers

Same Firecracker VM, 7 real devices plus 8 empty loops:

| path | time |
|---|---|
| spawn `lsblk --json --bytes` (reference) | ~3.3 ms |
| `get_devices()` | ~0.57 ms |

The walk is roughly 300 syscalls here at 3 to 4 µs each. On bare metal or
a normal KVM guest the same walk lands near 100 to 150 µs. Cutting further
means `openat` relative to a directory fd, which needs `libc` or `rustix`;
not worth a dependency for the current numbers.

## Not covered

- Multi-device filesystems where only one member appears in mountinfo
  (btrfs RAID, ZFS pools). lsblk has special-case grouping for these.
- The `BLKROGET` ioctl fallback when the `ro` attribute is missing. Every
  kernel this crate can realistically run on has the attribute.
- lsblk's udev and blkid property probing. None of those columns are in
  `BlockDevice`.
